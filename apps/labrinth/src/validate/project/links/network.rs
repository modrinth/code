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
const MAX_CACHE_ENTRIES: usize = 4096;
const CHECK_TIMEOUT: Duration = Duration::from_secs(25);
const RETRY_DELAY: Duration = Duration::from_millis(200);
const MAX_REDIRECTS: usize = 5;
const MAX_JSON_BYTES: usize = 65536;

type CacheEntry = (Instant, Arc<OnceCell<Probe>>);
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
}

async fn probe(url: &Url, json: bool, follow_redirects: bool) -> Probe {
	let key = format!("{json}:{follow_redirects}:{url}");
    let cell = {
        let mut cache = CACHE.lock().await;
        cache.retain(|_, (created, _)| created.elapsed() < CACHE_TTL);
        if cache.len() >= MAX_CACHE_ENTRIES
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
    cell.get_or_init(|| async {
        tokio::time::timeout(
			CHECK_TIMEOUT,
			probe_with_client(&CLIENT, url.clone(), json, follow_redirects),
		)
		.await
		.unwrap_or_default()
    })
    .await
    .clone()
}

async fn probe_with_client(
	client: &Client,
	mut url: Url,
	json: bool,
	follow_redirects: bool,
) -> Probe {
    let mut result = Probe::default();
    for _ in 0..=MAX_REDIRECTS {
        result.hops.push(url.clone());
        if super::globally_blocked(&url) || !fetchable(&url) {
            return result;
        }
        let mut response = None;
        for attempt in 0..4 {
            if attempt > 0 {
                tokio::time::sleep(RETRY_DELAY).await;
            }
            let Ok(_permit) = REQUESTS.acquire().await else {
                return result;
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
            match received {
                Ok(received) => {
                    let done = received.status().is_success()
                        || received.status().is_redirection();
                    response = Some(received);
                    if done {
                        break;
                    }
                }
                Err(_) => response = None,
            }
        }
        let Some(mut response) = response else {
            return result;
        };
        if follow_redirects && response.status().is_redirection() {
            let Some(next) = response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|location| url.join(location).ok())
            else {
                return result;
            };
            if result.hops.contains(&next) {
                return result;
            }
            url = next;
            continue;
        }
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
                    return result;
                }
                bytes.extend_from_slice(&chunk);
            }
            result.json = serde_json::from_slice(&bytes).ok();
        }
        return result;
    }
    result
}

pub(super) async fn validate(targets: Vec<LinkTarget>) -> Vec<ProjectNag> {
    let targets = targets
        .into_iter()
        .filter(|target| {
            super::validate_target(target)
                .is_none_or(|nag| nag.severity != ProjectNagSeverity::Required)
        })
        .collect::<Vec<_>>();
    let mut pending =
        stream::iter(targets.iter().enumerate().map(
            |(index, target)| async move { (index, check(target).await) },
        ))
        .buffer_unordered(8);
    let deadline = tokio::time::Instant::now() + CHECK_TIMEOUT;
    let mut completed = std::collections::HashSet::new();
    let mut nags = Vec::new();
    while let Ok(Some((index, found))) =
        tokio::time::timeout_at(deadline, pending.next()).await
    {
        completed.insert(index);
        nags.extend(found);
    }
    for (index, target) in targets.iter().enumerate() {
        if !completed.contains(&index) {
            nags.push(target.warning("unverifiable"));
        }
    }
    nags.sort_by(|a, b| a.details.to_string().cmp(&b.details.to_string()));
    nags.dedup();
    nags
}

async fn check(target: &LinkTarget) -> Vec<ProjectNag> {
    let Ok(url) = Url::parse(&target.url) else {
        return Vec::new();
    };
    let observed = probe(&url, false, target.field == "discord").await;
    let mut nags = Vec::new();
    for hop in &observed.hops {
        if super::globally_blocked(hop) {
            return vec![target.required("global_blocklist_match")];
        }
        if target.field != "description" {
            if !matches!(hop.host(), Some(url::Host::Domain(_))) {
                return vec![target.required("ip_address")];
            }
            if let Some(reason) = super::field_block(&target.field, hop) {
                return vec![target.required(reason)];
            }
        } else if super::description::known_download(hop, target.image) {
            return vec![target.required("download")];
        }
    }
    if !observed.accessible()
		&& !observed.status.is_some_and(|status| status.is_redirection())
	{
        nags.push(target.warning("unverifiable"));
    }
    let final_url = observed.hops.last().unwrap_or(&url);
    if target.field == "description" {
        let mime = observed
            .content_type
            .split(';')
            .next()
            .unwrap_or_default()
            .trim();
        if target.image
            && observed.accessible()
            && matches!(mime, "" | "application/octet-stream" | "text/html")
        {
            nags.push(target.warning("unverifiable"));
            return nags;
        }
        if observed.accessible()
            && super::description::response_is_download(
                &observed.content_type,
                &observed.disposition,
                target.image,
            )
        {
            nags.push(target.required("download"));
        }
        return nags;
    }
    if target.field == "discord" {
        if let Some(code) = super::discord_code(final_url) {
            let api = Url::parse(&format!(
                "https://discord.com/api/v10/invites/{code}"
            ))
            .unwrap();
            let invite = probe(&api, true, false).await;
            let nag = if invite.status == Some(StatusCode::NOT_FOUND)
                || invite.json.as_ref().is_some_and(|body| {
                    body.get("guild").is_none()
                        || body["guild"].is_null()
                        || body["expires_at"]
                            .as_str()
                            .and_then(|date| {
                                chrono::DateTime::parse_from_rfc3339(date).ok()
                            })
                            .is_some_and(|expiry| expiry < chrono::Utc::now())
                }) {
                Some(target.required("discord_invite"))
            } else if invite.json.is_none() {
                Some(target.warning("unverifiable"))
            } else {
                None
            };
            if let Some(nag) = nag {
                nags.push(nag);
            }
        } else if observed.accessible() {
            nags.push(target.required("discord_invite"));
        }
    }
    if matches!(target.field.as_str(), "source" | "issues" | "wiki")
        && super::from_domains(final_url, &["github.com"])
        && super::repository_path(final_url)
    {
        let parts = super::path(final_url);
        let api = Url::parse(&format!(
            "https://api.github.com/repos/{}/{}",
            parts[0], parts[1]
        ))
        .unwrap();
        let repository = probe(&api, true, false).await;
        if let Some(body) = repository.json {
            let enabled = match target.field.as_str() {
                "issues" => body["has_issues"].as_bool(),
                "wiki" => body["has_wiki"].as_bool(),
                _ => body["private"].as_bool().map(|private| !private),
            };
            match enabled {
                Some(false) => nags.push(target.required("repository_feature")),
                None => nags.push(target.warning("unverifiable")),
                _ => {}
            }
        } else {
            nags.push(target.warning("unverifiable"));
        }
    }
    if target.field == "source"
        && !super::from_domains(final_url, SOURCE_DOMAINS)
    {
        if !observed.accessible() {
            return nags;
        }
        if let Some(api) = forgejo_api(final_url) {
            let repository = probe(&api, true, false).await;
            if repository.status == Some(StatusCode::NOT_FOUND)
                || repository.json.as_ref().is_some_and(|body| {
                    body["full_name"].as_str().is_none()
                        || body["clone_url"].as_str().is_none()
                        || body["private"].as_bool() != Some(false)
                })
            {
                nags.push(target.required("source_repository"));
            } else if repository.json.is_none() {
                nags.push(target.warning("unverifiable"));
            }
        } else {
            nags.push(target.required("source_repository"));
        }
    }
    if !matches!(target.field.as_str(), "source" | "discord")
        && observed.accessible()
        && !super::from_domains(final_url, SOURCE_DOMAINS)
        && !super::allowed(&target.field, final_url)
        && let Some(api) = forgejo_api(final_url)
        && let Some(repository) = probe(&api, true, false).await.json
        && repository["full_name"].is_string()
        && repository["clone_url"].is_string()
        && !(matches!(target.field.as_str(), "wiki" | "issues")
            && super::repo_section(final_url, &target.field))
    {
        nags.push(target.required("wrong_field"));
    }
    nags.dedup();
    nags
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
    async fn retries_three_times_and_preserves_final_404() {
        let (client, task) = mock_client(vec![
			"HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
			4
		])
		.await;
        let start = Instant::now();
        let result = probe_with_client(
            &client,
            Url::parse("http://checks.modrinth.com/page").unwrap(),
            false,
			false,
        )
        .await;
        assert_eq!(result.status, Some(StatusCode::NOT_FOUND));
        assert!(!result.accessible());
        assert_eq!(task.await.unwrap(), vec!["HEAD"; 4]);
        assert!(start.elapsed() >= RETRY_DELAY * 3);
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
			false,
        )
        .await;
        assert!(result.accessible());
        assert_eq!(result.content_type, "application/zip");
        assert_eq!(result.disposition, "attachment");
        assert_eq!(task.await.unwrap(), vec!["HEAD", "GET"]);
    }

    #[tokio::test]
    async fn prohibited_redirect_is_recorded_without_being_fetched() {
        let (client, task) = mock_client(vec!["HTTP/1.1 302 Found\r\nLocation: https://bit.ly/prohibited\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"]).await;
        let result = probe_with_client(
            &client,
            Url::parse("http://checks.modrinth.com/invite").unwrap(),
            false,
			true,
        )
        .await;
        assert_eq!(result.hops.len(), 2);
        assert!(super::super::globally_blocked(result.hops.last().unwrap()));
        assert_eq!(task.await.unwrap(), vec!["HEAD"]);
    }

	#[tokio::test]
	async fn redirects_are_not_followed_when_disabled() {
		let (client, task) = mock_client(vec!["HTTP/1.1 302 Found\r\nLocation: https://bit.ly/prohibited\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"]).await;
		let url = Url::parse("http://checks.modrinth.com/sponsors/creator").unwrap();
		let result = probe_with_client(&client, url.clone(), false, false).await;
		assert_eq!(result.status, Some(StatusCode::FOUND));
		assert_eq!(result.hops, vec![url]);
		assert_eq!(task.await.unwrap(), vec!["HEAD"]);
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
