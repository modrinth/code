use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use futures::{StreamExt, stream};
use reqwest::{
    Client, Method, StatusCode,
    dns::{Addrs, Name, Resolve, Resolving},
    header,
};
use serde_json::Value;
use tokio::sync::{Mutex, OnceCell, Semaphore};
use url::Url;

use super::{LinkTarget, ProjectNag, ProjectNagSeverity, SOURCE_DOMAINS};

const CACHE_TTL: Duration = Duration::from_secs(600);
const HTTP_FAILURE_CACHE_TTL: Duration = Duration::from_secs(60);
const UNVERIFIABLE_CACHE_TTL: Duration = Duration::from_secs(5);
const MAX_CACHE_ENTRIES: usize = 4096;
const CHECK_TIMEOUT: Duration = Duration::from_secs(25);
const RETRY_DELAY: Duration = Duration::from_millis(200);
const MAX_REDIRECTS: usize = 5;
const MAX_JSON_BYTES: usize = 65536;

type CacheEntry = (Instant, Arc<OnceCell<(Instant, Probe)>>);
static CACHE: LazyLock<Mutex<HashMap<String, CacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static REQUESTS: Semaphore = Semaphore::const_new(16);
static CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .user_agent(concat!(
            "Labrinth link validation/",
            env!("CARGO_PKG_VERSION")
        ))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(Arc::new(PublicResolver))
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(5))
        .build()
        .expect("link validation HTTP client")
});

struct PublicResolver;

impl Resolve for PublicResolver {
    fn resolve(&self, name: Name) -> Resolving {
        Box::pin(async move {
            let addresses = tokio::net::lookup_host((name.as_str(), 0))
                .await?
                .collect::<Vec<_>>();
            if addresses.is_empty()
                || addresses.iter().any(|address| !public_ip(address.ip()))
            {
                return Err(std::io::Error::other(
					"link hostname does not resolve exclusively to public addresses",
				)
				.into());
            }
            Ok(Box::new(addresses.into_iter()) as Addrs)
        })
    }
}

/// Restricts user-supplied link checks to public IPs to prevent SSRF against
/// local services, private networks, and cloud metadata endpoints.
fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_documentation()
                && !ip.is_broadcast()
                && !ip.is_unspecified()
                && a != 0
                && a < 224
                && !(a == 100 && (64..128).contains(&b))
                && !(a == 192 && b == 0 && c == 0)
                && !(a == 198 && matches!(b, 18 | 19))
        }
        IpAddr::V6(ip) => {
            if let Some(ip) = ip.to_ipv4_mapped() {
                return public_ip(IpAddr::V4(ip));
            }
            let segments = ip.segments();
            (segments[0] & 0xe000) == 0x2000
                && !(segments[0] == 0x2001
                    && (segments[1] < 0x0200 || segments[1] == 0x0db8))
                && segments[0] != 0x2002
        }
    }
}

fn fetchable(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http")
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(url.port_or_known_default(), Some(80 | 443))
        && match url.host() {
            Some(url::Host::Ipv4(ip)) => public_ip(IpAddr::V4(ip)),
            Some(url::Host::Ipv6(ip)) => public_ip(IpAddr::V6(ip)),
            Some(url::Host::Domain(host)) => {
                host.contains('.')
                    && !["localhost", "local", "internal", "test", "invalid"]
                        .iter()
                        .any(|suffix| {
                            super::domain_matches(
                                host.trim_end_matches('.'),
                                suffix,
                            )
                        })
            }
            None => false,
        }
}

#[derive(Clone, Debug, Default)]
struct Probe {
    hops: Vec<Url>,
    status: Option<StatusCode>,
    content_type: String,
    disposition: String,
    json: Option<Value>,
}

impl Probe {
    fn accessible(&self) -> bool {
        self.status.is_some_and(|status| status.is_success())
    }

    fn cache_ttl(&self, json: bool) -> Duration {
        match self.status {
            Some(status)
                if status.is_client_error() || status.is_server_error() =>
            {
                HTTP_FAILURE_CACHE_TTL
            }
            Some(status)
                if (status.is_success() || status.is_redirection())
                    && (!json || self.json.is_some()) =>
            {
                CACHE_TTL
            }
            _ => UNVERIFIABLE_CACHE_TTL,
        }
    }
}

async fn get_or_probe(url: &Url, json: bool) -> Probe {
    let key = format!("{json}:{url}");
    // Remove expired cache entries and get or create the shared slot for this URL.
    let cell = {
        let mut cache = CACHE.lock().await;
        cache.retain(|_, (created, cell)| {
            cell.get().map_or_else(
                || created.elapsed() < CACHE_TTL,
                |(expires, _)| Instant::now() < *expires,
            )
        });
        if !cache.contains_key(&key)
            && cache.len() >= MAX_CACHE_ENTRIES
            && let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, (created, _))| *created)
                .map(|(key, _)| key.clone())
        {
            cache.remove(&oldest);
        }
        cache
            .entry(key)
            .or_insert_with(|| (Instant::now(), Arc::new(OnceCell::new())))
            .1
            .clone()
    };

    // Reuse the cached result or probe url
    cell.get_or_init(|| async {
        let result = tokio::time::timeout(
            CHECK_TIMEOUT,
            probe_with_client(&CLIENT, url.clone(), json),
        )
        .await
        .unwrap_or_default();
        (Instant::now() + result.cache_ttl(json), result)
    })
    .await
    .1
    .clone()
}

// only allow redirect if
// - hostname stays the same, i.e. github.com → api.github.com is rejected
// - https is not downgraded to http
fn redirect_allowed(current: &Url, next: &Url) -> bool {
    super::host(current).eq_ignore_ascii_case(super::host(next))
        && !(current.scheme() == "https" && next.scheme() != "https")
}

fn login_destination(url: &Url) -> bool {
    matches!(
        url.path().trim_end_matches('/'),
        "/login"
            | "/signin"
            | "/sign-in"
            | "/users/sign_in"
            | "/user/login"
            | "/session/new"
    )
}

async fn probe_with_client(client: &Client, mut url: Url, json: bool) -> Probe {
    let mut result = Probe::default();
    for _ in 0..=MAX_REDIRECTS {
        result.hops.push(url.clone());
        if super::globally_blocked(&url) || !fetchable(&url) {
            return result;
        }

        let Some(response) = request_with_retries(client, &url, json).await
        else {
            return result;
        };

        // probe again for redirect, if allowed
        if response.status().is_redirection() {
            let Some(next) = response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|location| url.join(location).ok())
            else {
                return result;
            };
            if result.hops.contains(&next) || !redirect_allowed(&url, &next) {
                return result;
            }
            url = next;
        } else {
            read_probe_response(&mut result, response, json).await;
            return result;
        }
    }
    result
}

async fn request_with_retries(
    client: &Client,
    url: &Url,
    json: bool,
) -> Option<reqwest::Response> {
    for attempt in 0..4 {
        if attempt > 0 {
            tokio::time::sleep(RETRY_DELAY).await;
        }
        let Ok(_permit) = REQUESTS.acquire().await else {
            return None;
        };
        let method = if json { Method::GET } else { Method::HEAD };
        let mut request = client.request(method, url.clone());
        if json {
            request = request.header(header::ACCEPT, "application/json");
        }
        let mut received = request.send().await;
        if !json
            && received.as_ref().is_ok_and(|response| {
                matches!(
                    response.status(),
                    StatusCode::METHOD_NOT_ALLOWED
                        | StatusCode::NOT_IMPLEMENTED
                )
            })
        {
            received = client.get(url.clone()).send().await;
        }

        let should_retry = match &received {
            Ok(response) => response.status().is_server_error(),
            Err(_) => true,
        };
        if !should_retry || attempt == 3 {
            return received.ok();
        }
    }
    None
}

async fn read_probe_response(
    result: &mut Probe,
    mut response: reqwest::Response,
    json: bool,
) {
    result.status = Some(response.status());
    result.content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .into();
    result.disposition = response
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .into();
    if json && result.accessible() {
        let mut bytes = Vec::new();
        while let Ok(Some(chunk)) = response.chunk().await {
            if bytes.len() + chunk.len() > MAX_JSON_BYTES {
                return;
            }
            bytes.extend_from_slice(&chunk);
        }
        result.json = serde_json::from_slice(&bytes).ok();
    }
}

pub(super) async fn validate(targets: Vec<LinkTarget>) -> Vec<ProjectNag> {
	let targets = targets.into_iter().filter(needs_network_check);
	let deadline = tokio::time::Instant::now() + CHECK_TIMEOUT;
	let mut pending = stream::iter(targets)
		.map(|target| check_with_deadline(target, deadline))
		.buffer_unordered(8);
	let mut nags = Vec::new();
	while let Some(found) = pending.next().await {
		nags.extend(found);
	}
	nags.sort_by(|a, b| a.details.to_string().cmp(&b.details.to_string()));
	nags.dedup();
	nags
}

fn needs_network_check(target: &LinkTarget) -> bool {
	super::validate_target(target)
		.is_none_or(|nag| nag.severity != ProjectNagSeverity::Required)
}

/// Validates a link within the batch deadline, warning if it cannot finish in time.
async fn check_with_deadline(
	target: LinkTarget,
	deadline: tokio::time::Instant,
) -> Vec<ProjectNag> {
	if tokio::time::Instant::now() >= deadline {
		return vec![target.warning("unverifiable")];
	}
	match tokio::time::timeout_at(deadline, validate_link(&target)).await {
		Ok(nags) => nags,
		Err(_) => vec![target.warning("unverifiable")],
	}
}

async fn validate_link(target: &LinkTarget) -> Vec<ProjectNag> {
	let Ok(url) = Url::parse(&target.url) else {
		return Vec::new();
	};
	let observed = get_or_probe(&url, false).await;
	if let Some(nag) = check_hops(target, &url, &observed) {
		return vec![nag];
	}

	let mut nags = Vec::new();
	if !observed.accessible() {
		nags.push(target.warning("unverifiable"));
	}
	let final_url = observed.hops.last().unwrap_or(&url);
	nags.extend(validate_link_field(target, final_url, &observed).await);
	nags
}

async fn validate_link_field(
	target: &LinkTarget,
	final_url: &Url,
	observed: &Probe,
) -> Option<ProjectNag> {
	match target.field.as_str() {
		"description" => check_description_response(target, observed),
		"discord" => check_discord_invite(target, final_url, observed).await,
		"source" | "issues" | "wiki" if is_github_repository(final_url) => {
			check_github_repository(target, final_url).await
		}
		"source" => {
			if observed.accessible()
				&& !super::from_domains(final_url, SOURCE_DOMAINS)
			{
				check_source_repository(target, final_url).await
			} else {
				None
			}
		}
		_ => {
			if observed.accessible()
				&& !super::from_domains(final_url, SOURCE_DOMAINS)
				&& !super::allowed(&target.field, final_url)
			{
				check_repository_field(target, final_url).await
			} else {
				None
			}
		}
	}
}

fn is_github_repository(url: &Url) -> bool {
	super::from_domains(url, &["github.com"]) && super::repository_path(url)
}

/// Checks visited URLs for blocked hosts, IP addresses, misplaced links,
/// downloads, and redirects to login pages.
fn check_hops(
    target: &LinkTarget,
    original_url: &Url,
    observed: &Probe,
) -> Option<ProjectNag> {
    for hop in &observed.hops {
        if super::globally_blocked(hop) {
            return Some(target.required("global_blocklist_match"));
        }
        if hop != original_url && login_destination(hop) {
            return Some(target.warning("unverifiable"));
        }
        if target.field != "description" {
            if !matches!(hop.host(), Some(url::Host::Domain(_))) {
                return Some(target.required("ip_address"));
            }
            if let Some(reason) = super::field_block(&target.field, hop) {
                return Some(target.required(reason));
            }
        } else if super::description::known_download(hop, target.image) {
            return Some(target.required("download"));
        }
    }
    None
}

/// Checks response headers for downloads and unverifiable description images.
fn check_description_response(
    target: &LinkTarget,
    observed: &Probe,
) -> Option<ProjectNag> {
    if !observed.accessible() {
        return None;
    }
    let mime = observed
        .content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim();
    if target.image
        && matches!(mime, "" | "application/octet-stream" | "text/html")
    {
        return Some(target.warning("unverifiable"));
    }
    if super::description::response_is_download(
        &observed.content_type,
        &observed.disposition,
        target.image,
    ) {
        return Some(target.required("download"));
    }
    None
}

/// Checks that the URL identifies an existing, unexpired Discord server invite.
async fn check_discord_invite(
    target: &LinkTarget,
    url: &Url,
    observed: &Probe,
) -> Option<ProjectNag> {
    let Some(code) = super::discord_code(url) else {
        return observed
            .accessible()
            .then(|| target.required("discord_invite"));
    };
    let api =
        Url::parse(&format!("https://discord.com/api/v10/invites/{code}"))
            .unwrap();
    let invite = get_or_probe(&api, true).await;
    if invite.status == Some(StatusCode::NOT_FOUND)
        || invite.json.as_ref().is_some_and(|body| {
            body.get("guild").is_none()
                || body["guild"].is_null()
                || body["expires_at"]
                    .as_str()
                    .and_then(|date| {
                        chrono::DateTime::parse_from_rfc3339(date).ok()
                    })
                    .is_some_and(|expiry| expiry < chrono::Utc::now())
        })
    {
        Some(target.required("discord_invite"))
    } else if invite.json.is_none() {
        Some(target.warning("unverifiable"))
    } else {
        None
    }
}

/// Checks GitHub repository visibility for source links, or whether issues/wiki
/// are enabled for links in those fields.
async fn check_github_repository(
    target: &LinkTarget,
    url: &Url,
) -> Option<ProjectNag> {
    let parts = super::path(url);
    let api = Url::parse(&format!(
        "https://api.github.com/repos/{}/{}",
        parts[0], parts[1]
    ))
    .unwrap();
    let repository = get_or_probe(&api, true).await;
    let Some(body) = repository.json else {
        return Some(target.warning("unverifiable"));
    };
    let enabled = match target.field.as_str() {
        "issues" => body["has_issues"].as_bool(),
        "wiki" => body["has_wiki"].as_bool(),
        _ => body["private"].as_bool().map(|private| !private),
    };
    match enabled {
        Some(false) => Some(target.required("repository_feature")),
        None => Some(target.warning("unverifiable")),
        Some(true) => None,
    }
}

/// Checks an unrecognized source host's Forgejo-style API for a public repository.
async fn check_source_repository(
    target: &LinkTarget,
    url: &Url,
) -> Option<ProjectNag> {
    let Some(api) = forgejo_api(url) else {
        return Some(target.required("source_repository"));
    };
    let repository = get_or_probe(&api, true).await;
    if repository.status == Some(StatusCode::NOT_FOUND)
        || repository.json.as_ref().is_some_and(|body| {
            body["full_name"].as_str().is_none()
                || body["clone_url"].as_str().is_none()
                || body["private"].as_bool() != Some(false)
        })
    {
        Some(target.required("source_repository"))
    } else if repository.json.is_none() {
        Some(target.warning("unverifiable"))
    } else {
        None
    }
}

/// Detects Forgejo-style repository links in the wrong field, allowing matching
/// issues and wiki sections in their respective fields.
async fn check_repository_field(
    target: &LinkTarget,
    url: &Url,
) -> Option<ProjectNag> {
    let api = forgejo_api(url)?;
    let repository = get_or_probe(&api, true).await.json?;
    if repository["full_name"].is_string()
        && repository["clone_url"].is_string()
        && !(matches!(target.field.as_str(), "wiki" | "issues")
            && super::repo_section(url, &target.field))
    {
        Some(target.required("wrong_field"))
    } else {
        None
    }
}

fn forgejo_api(url: &Url) -> Option<Url> {
    let parts = super::path(url);
    if parts.len() < 2 {
        return None;
    }
    let mut api = url.clone();
    api.set_query(None);
    api.set_fragment(None);
    api.set_path(&format!(
        "/api/v1/repos/{}/{}",
        parts[0],
        parts[1].trim_end_matches(".git")
    ));
    Some(api)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn mock_client(
        responses: Vec<&'static str>,
    ) -> (Client, tokio::task::JoinHandle<Vec<String>>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .resolve("checks.modrinth.com", listener.local_addr().unwrap())
            .build()
            .unwrap();
        let task = tokio::spawn(async move {
            let mut methods = Vec::new();
            for response in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                let mut bytes = [0u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let count = socket.read(&mut bytes).await.unwrap();
                    if count == 0 {
                        break;
                    }
                    request.extend_from_slice(&bytes[..count]);
                }
                methods.push(
                    String::from_utf8_lossy(&request)
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .to_owned(),
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
            methods
        });
        (client, task)
    }

    #[tokio::test]
    async fn preserves_404_without_retrying() {
        let (client, task) = mock_client(vec![
			"HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
		])
		.await;
        let result = probe_with_client(
            &client,
            Url::parse("http://checks.modrinth.com/page").unwrap(),
            false,
        )
        .await;
        assert_eq!(result.status, Some(StatusCode::NOT_FOUND));
        assert!(!result.accessible());
        assert_eq!(task.await.unwrap(), vec!["HEAD"]);
    }

    #[tokio::test]
    async fn head_falls_back_to_get_and_keeps_download_headers() {
        let (client, task) = mock_client(vec![
			"HTTP/1.1 405 Method Not Allowed\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
			"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/zip\r\nContent-Disposition: attachment\r\nContent-Length: 0\r\n\r\n",
		]).await;
        let result = probe_with_client(
            &client,
            Url::parse("http://checks.modrinth.com/file").unwrap(),
            false,
        )
        .await;
        assert!(result.accessible());
        assert_eq!(result.content_type, "application/zip");
        assert_eq!(result.disposition, "attachment");
        assert_eq!(task.await.unwrap(), vec!["HEAD", "GET"]);
    }

    #[tokio::test]
    async fn same_hostname_redirect_reaches_final_page() {
        let (client, task) = mock_client(vec![
			"HTTP/1.1 301 Moved Permanently\r\nLocation: /owner/renamed/wiki\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
			"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: 0\r\n\r\n",
		]).await;
        let url =
            Url::parse("http://checks.modrinth.com/owner/old/wiki").unwrap();
        let result = probe_with_client(&client, url, false).await;
        assert!(result.accessible());
        assert_eq!(result.hops.last().unwrap().path(), "/owner/renamed/wiki");
        assert_eq!(task.await.unwrap(), vec!["HEAD", "HEAD"]);
    }

    #[tokio::test]
    async fn cross_hostname_redirect_is_not_fetched_or_verified() {
        let (client, task) = mock_client(vec!["HTTP/1.1 302 Found\r\nLocation: https://other.modrinth.com/page\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"]).await;
        let url = Url::parse("http://checks.modrinth.com/page").unwrap();
        let result = probe_with_client(&client, url.clone(), false).await;
        assert!(!result.accessible());
        assert_eq!(result.hops, vec![url]);
        assert_eq!(task.await.unwrap(), vec!["HEAD"]);
    }

    #[test]
    fn redirect_policy_preserves_hostname_and_https() {
        let url = Url::parse("https://github.com/owner/old/wiki").unwrap();
        for destination in [
            "http://github.com/owner/new/wiki",
            "https://api.github.com/owner/new/wiki",
            "https://github.com.evil.org/owner/new/wiki",
        ] {
            assert!(!redirect_allowed(&url, &Url::parse(destination).unwrap()));
        }
        assert!(redirect_allowed(
            &url,
            &url.join("/owner/new/wiki").unwrap()
        ));
    }

    #[test]
    fn redirected_wiki_homepage_is_wrong_field() {
        let url = Url::parse("https://github.com/owner/repository").unwrap();
        assert_eq!(
            super::super::field_block("wiki", &url),
            Some("wrong_field")
        );
        assert!(login_destination(
            &url.join("/login?return_to=/owner/repository/wiki").unwrap()
        ));
        assert!(!login_destination(
            &url.join("/owner/repository/wiki/Login").unwrap()
        ));
    }

    #[test]
    fn network_requests_only_connect_to_public_addresses() {
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "169.254.169.254",
            "100.64.0.1",
            "192.0.2.1",
            "198.18.0.1",
            "224.0.0.1",
            "::1",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
            "2002:7f00:1::",
        ] {
            assert!(!public_ip(ip.parse().unwrap()), "{ip}");
        }
        for ip in ["1.1.1.1", "8.8.8.8", "2606:4700:4700::1111"] {
            assert!(public_ip(ip.parse().unwrap()), "{ip}");
        }
        for url in [
            "http://localhost/",
            "https://service.internal/",
            "http://127.0.0.1/",
            "https://public.org:8080",
            "ftp://public.org/file",
            "https://name:pass@public.org/",
        ] {
            assert!(!fetchable(&Url::parse(url).unwrap()), "{url}");
        }
    }

    #[test]
    fn forgejo_repository_request_uses_origin_and_repository_path() {
        assert_eq!(
			forgejo_api(
				&Url::parse(
					"https://git.project.dev/owner/repo/src/branch/main?x=1#readme"
				)
				.unwrap()
			)
			.unwrap()
			.as_str(),
			"https://git.project.dev/api/v1/repos/owner/repo"
		);
    }
}
