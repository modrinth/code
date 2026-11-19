use crate::prelude::Credentials;
use crate::state::ModrinthCredentials;
use regex::bytes::{Captures, Regex};
use serde::Serialize;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{ConnectOptions, Connection, SqlitePool};
use std::net::Ipv6Addr;
use std::path::Path;
use std::sync::LazyLock;

#[derive(Serialize, Debug, Default)] // Not deserialize
#[serde(transparent)]
pub struct CensoredString(String);
impl CensoredString {
    pub fn censor(mut s: String, credentials_list: &[Credentials]) -> Self {
        let Ok(username) = whoami::username_os() else {
            return Self(
                "[Log withheld: unable to determine computer username]".into(),
            );
        };
        let username = username.to_string_lossy();
        if username.is_empty() {
            return Self(
                "[Log withheld: unable to determine computer username]".into(),
            );
        }
        s = s
            .replace(&format!("/{username}/"), "/{COMPUTER_USERNAME}/")
            .replace(&format!("\\{username}\\"), "\\{COMPUTER_USERNAME}\\");
        for credentials in credentials_list {
            // Use the offline profile to guarantee that this function does not cause
            // Mojang API request, and is never delayed by a network request. The offline
            // profile is optimistically updated on upsert from time to time anyway
            s = s
                .replace(&credentials.access_token, "{MINECRAFT_ACCESS_TOKEN}")
                .replace(
                    &credentials.offline_profile.name,
                    "{MINECRAFT_USERNAME}",
                )
                .replace(
                    &credentials.offline_profile.id.as_simple().to_string(),
                    "{MINECRAFT_UUID}",
                )
                .replace(
                    &credentials.offline_profile.id.as_hyphenated().to_string(),
                    "{MINECRAFT_UUID}",
                );
        }

        Self(censor_log_text(s))
    }
}

pub(crate) struct Redactor(Vec<(Vec<u8>, Vec<u8>)>);

impl Redactor {
    pub(crate) async fn load(pool: &SqlitePool) -> crate::Result<Self> {
        let mut replacements = Vec::new();
        for credentials in ModrinthCredentials::get_all(pool).await? {
            replacements.push((
                credentials.session.into_bytes(),
                b"{MODRINTH_ACCESS_TOKEN}".to_vec(),
            ));
        }
        for token in minecraft_tokens(pool).await? {
            replacements
                .push((token.into_bytes(), b"{MINECRAFT_TOKEN}".to_vec()));
        }
        if let Ok(username) = whoami::username_os() {
            let username = username.to_string_lossy();
            if !username.is_empty() {
                replacements.push((
                    format!("/{username}/").into_bytes(),
                    b"/{COMPUTER_USERNAME}/".to_vec(),
                ));
                replacements.push((
                    format!("\\{username}\\").into_bytes(),
                    b"\\{COMPUTER_USERNAME}\\".to_vec(),
                ));
            }
        }
        replacements.retain(|(secret, _)| !secret.is_empty());
        Ok(Self(replacements))
    }

    pub(crate) fn apply(&self, mut data: Vec<u8>) -> Vec<u8> {
        for (secret, replacement) in &self.0 {
            data = replace_bytes(data, secret, replacement);
        }
        censor_log_patterns(data)
    }

    pub(crate) fn apply_config(&self, data: Vec<u8>) -> Option<Vec<u8>> {
        censor_config_patterns(self.apply(data))
    }

    pub(crate) fn apply_path(&self, path: &Path) -> String {
        let path = path.to_string_lossy().into_owned().into_bytes();
        String::from_utf8_lossy(&self.apply(path)).into_owned()
    }
}

fn replace_bytes(data: Vec<u8>, from: &[u8], to: &[u8]) -> Vec<u8> {
    if !data.windows(from.len()).any(|window| window == from) {
        return data;
    }
    let mut output = Vec::with_capacity(data.len());
    let mut index = 0;
    while index < data.len() {
        if data[index..].starts_with(from) {
            output.extend_from_slice(to);
            index += from.len();
        } else {
            output.push(data[index]);
            index += 1;
        }
    }
    output
}

pub(crate) async fn scrub_database_snapshot(path: &Path) -> crate::Result<()> {
    let mut connection = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .journal_mode(SqliteJournalMode::Delete)
        .pragma("secure_delete", "ON")
        .connect()
        .await?;
    sqlx::query!(
        "UPDATE minecraft_users SET access_token = '', refresh_token = ''"
    )
    .execute(&mut connection)
    .await?;
    sqlx::query!("DELETE FROM minecraft_device_tokens")
        .execute(&mut connection)
        .await?;
    sqlx::query!("UPDATE modrinth_users SET session_id = ''")
        .execute(&mut connection)
        .await?;
    sqlx::query!("VACUUM").execute(&mut connection).await?;
    connection.close().await?;
    Ok(())
}

pub(crate) async fn censor_support_text(
    mut text: String,
    pool: &SqlitePool,
) -> crate::Result<String> {
    for credentials in ModrinthCredentials::get_all(pool).await? {
        replace_nonempty(
            &mut text,
            &credentials.session,
            "{MODRINTH_ACCESS_TOKEN}",
        );
    }

    for token in minecraft_tokens(pool).await? {
        replace_nonempty(&mut text, &token, "{MINECRAFT_TOKEN}");
    }

    text = censor_log_text(text);

    Ok(text)
}

async fn minecraft_tokens(
    pool: &sqlx::SqlitePool,
) -> crate::Result<Vec<String>> {
    let rows =
        sqlx::query!("SELECT access_token, refresh_token FROM minecraft_users")
            .fetch_all(pool)
            .await?;

    Ok(rows
        .into_iter()
        .flat_map(|row| [row.access_token, row.refresh_token])
        .collect())
}

fn replace_nonempty(text: &mut String, value: &str, replacement: &str) {
    if !value.is_empty() {
        *text = text.replace(value, replacement);
    }
}

type PatternReplacement = fn(&[u8], &Captures<'_>) -> Option<Vec<u8>>;

static LOG_PATTERNS: LazyLock<Vec<(Regex, PatternReplacement)>> = LazyLock::new(
    || {
        let patterns: [(&str, PatternReplacement); 12] = [
            (r"[0-9]{1,3}(?:\.[0-9]{1,3}){3}", censor_ipv4),
            (
                r"[0-9A-Fa-f.]*:[0-9A-Fa-f:.]*[0-9A-Fa-f:](?:%[0-9A-Za-z_-]+)?",
                censor_ipv6,
            ),
            (r"(?-u)([A-Za-z]):\\Users\\[^\\\r\n]+\\", |_, captures| {
                Some([&captures[1], br":\Users\{COMPUTER_USERNAME}\"].concat())
            }),
            (
                r"(?-u)([A-Za-z]):\\\\Users\\\\[^\\\r\n]+\\\\",
                |_, captures| {
                    Some(
                        [&captures[1], br":\\Users\\{COMPUTER_USERNAME}\\"]
                            .concat(),
                    )
                },
            ),
            (r"(?-u)([A-Za-z]):/Users/[^/\r\n]+/", |_, captures| {
                Some([&captures[1], b":/Users/{COMPUTER_USERNAME}/"].concat())
            }),
            (r"(?-u)/home/[^/\r\n]+/", |data, captures| {
                (!preceded_by_word(data, captures))
                    .then(|| b"/home/{COMPUTER_USERNAME}/".to_vec())
            }),
            (r"(?-u)/Users/[^/\r\n]+/", |data, captures| {
                (!preceded_by_word(data, captures))
                    .then(|| b"/Users/{COMPUTER_USERNAME}/".to_vec())
            }),
            (r"USERNAME=\w+", |_, _| {
                Some(b"USERNAME={COMPUTER_USERNAME}".to_vec())
            }),
            (r"\(Session ID is token:[^:\r\n]+:[^)\r\n]+\)", |_, _| {
                Some(
                    b"(Session ID is token:{MINECRAFT_ACCESS_TOKEN}:{MINECRAFT_UUID})"
                        .to_vec(),
                )
            }),
            (r"--accessToken \S+", |_, _| {
                Some(b"--accessToken {MINECRAFT_ACCESS_TOKEN}".to_vec())
            }),
            (r#""authToken":"[^"]+""#, |_, _| {
                Some(br#""authToken":"{AUTH_TOKEN}""#.to_vec())
            }),
            (r#""refreshToken":"[^"]+""#, |_, _| {
                Some(br#""refreshToken":"{REFRESH_TOKEN}""#.to_vec())
            }),
        ];
        patterns
            .into_iter()
            .map(|(pattern, replacement)| {
                (
                    Regex::new(pattern).expect("valid log redaction pattern"),
                    replacement,
                )
            })
            .collect()
    },
);

static CONFIG_PATTERNS: LazyLock<Vec<(Regex, PatternReplacement)>> =
    LazyLock::new(|| {
        let patterns: [(&str, PatternReplacement); 2] = [
            (
                r"(?i)https?://(?:[a-z0-9-]+\.)?discord(?:app)?\.com/api/webhooks/[^\s\x22',]+",
                |_, _| Some(b"{DISCORD_WEBHOOK}".to_vec()),
            ),
            (
                r#"(?i)([\w.-]*(?:token|secret|password|passwd|api[_-]?key|webhook|credential)[\w.-]*["']?\s*[:=]\s*["']?)([^"'\r\n,}\]#]*[^"'\s,}\]#])"#,
                censor_config_secret,
            ),
        ];
        patterns
            .into_iter()
            .map(|(pattern, replacement)| {
                (
                    Regex::new(pattern)
                        .expect("valid config redaction pattern"),
                    replacement,
                )
            })
            .collect()
    });

static MULTILINE_CONFIG_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?im)[\w.-]*(?:token|secret|password|passwd|api[_-]?key|webhook|credential)[\w.-]*["']?[ \t]*[:=][ \t]*(?:"""|'''|[|>][-+0-9]*[ \t]*(?:#[^\n]*)?\r?$)"#,
    )
    .expect("valid multiline config secret pattern")
});

fn censor_config_secret(_: &[u8], captures: &Captures<'_>) -> Option<Vec<u8>> {
    let prefix = &captures[1];
    let value = &captures[2];
    let quoted = prefix.ends_with(b"\"") || prefix.ends_with(b"'");
    let keep = value.eq_ignore_ascii_case(b"true")
        || value.eq_ignore_ascii_case(b"false")
        || value.eq_ignore_ascii_case(b"null")
        || value.starts_with(b"{")
        || !quoted
            && value.iter().all(|byte| {
                byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'+')
            });
    (!keep).then(|| [prefix, b"{REDACTED}".as_slice()].concat())
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn preceded_by_word(data: &[u8], captures: &Captures<'_>) -> bool {
    data[..captures.get_match().start()]
        .last()
        .is_some_and(|byte| is_word_byte(*byte))
}

fn censor_ipv4(data: &[u8], captures: &Captures<'_>) -> Option<Vec<u8>> {
    let address = captures.get_match();
    let before = &data[..address.start()];
    if before
        .last()
        .is_some_and(|byte| is_word_byte(*byte) || *byte == b'-')
        || data.get(address.end()).is_some_and(u8::is_ascii_digit)
        || data[address.end()..].starts_with(b".")
            && data.get(address.end() + 1).is_some_and(u8::is_ascii_digit)
        || before.ends_with(b"version ")
        || before.ends_with(b"version: ")
    {
        return None;
    }
    let octets = address
        .as_bytes()
        .split(|byte| *byte == b'.')
        .collect::<Vec<_>>();
    let exempt = matches!(
        octets.as_slice(),
        [b"127", ..]
            | [b"0", b"0", b"0", b"0"]
            | [b"1", b"0" | b"1", b"0" | b"1", b"1"]
            | [b"8", b"8", b"8" | b"4", b"8" | b"4"]
    );
    (!exempt).then(|| b"{IPV4_ADDRESS}".to_vec())
}

fn censor_ipv6(data: &[u8], captures: &Captures<'_>) -> Option<Vec<u8>> {
    let candidate = captures.get_match();
    if preceded_by_word(data, captures)
        || data
            .get(candidate.end())
            .is_some_and(|byte| is_word_byte(*byte))
    {
        return None;
    }
    let candidate = std::str::from_utf8(candidate.as_bytes()).ok()?;
    let address = candidate.split('%').next()?;
    address.parse::<Ipv6Addr>().ok()?;
    let loopback_or_unspecified = address
        .strip_suffix('1')
        .unwrap_or(address)
        .bytes()
        .all(|byte| byte == b'0' || byte == b':');
    (!loopback_or_unspecified).then(|| b"{IPV6_ADDRESS}".to_vec())
}

/// Censors the same kinds of data that mclogs filters out of uploaded logs: IPv4 and IPv6
/// addresses (other than loopback and well-known public DNS resolvers), usernames in home
/// directory paths and the `USERNAME` environment variable, and Minecraft session tokens.
///
/// Adapted from the log filters of mclogs (<https://github.com/aternosorg/mclogs>),
/// copyright (c) 2018-2024 Aternos GmbH, MIT license.
///
/// See `THIRD_PARTY.md`.
pub(crate) fn censor_log_patterns(data: Vec<u8>) -> Vec<u8> {
    apply_patterns(data, &LOG_PATTERNS)
}

/// Censors values that look like credentials in a mod config file: Discord webhook URLs and
/// values of keys whose names mention tokens, secrets, passwords, API keys, webhooks or
/// credentials. Booleans, empty values and unquoted numbers are kept since they carry no secret.
///
/// Returns `None` when a secret-named key has a multiline value (YAML block scalars, TOML
/// multiline strings), which can't be censored reliably, so the file should be left out.
fn censor_config_patterns(data: Vec<u8>) -> Option<Vec<u8>> {
    if MULTILINE_CONFIG_SECRET.is_match(&data) {
        return None;
    }
    Some(apply_patterns(data, &CONFIG_PATTERNS))
}

fn apply_patterns(
    mut data: Vec<u8>,
    patterns: &[(Regex, PatternReplacement)],
) -> Vec<u8> {
    for (pattern, replacement) in patterns {
        let mut output = Vec::new();
        let mut copied = 0;
        for captures in pattern.captures_iter(&data) {
            if let Some(replacement) = replacement(&data, &captures) {
                let range = captures.get_match().range();
                output.extend_from_slice(&data[copied..range.start]);
                output.extend_from_slice(&replacement);
                copied = range.end;
            }
        }
        if copied > 0 {
            output.extend_from_slice(&data[copied..]);
            data = output;
        }
    }
    data
}

fn censor_log_text(text: String) -> String {
    match String::from_utf8(censor_log_patterns(text.into_bytes())) {
        Ok(text) => text,
        Err(error) => String::from_utf8_lossy(error.as_bytes()).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{censor_config_patterns, censor_log_text};

    fn censor_config(text: &str) -> Option<String> {
        censor_config_patterns(text.as_bytes().to_vec())
            .map(|data| String::from_utf8(data).unwrap())
    }

    #[test]
    fn censors_config_secrets() {
        for (text, censored) in [
            ("botToken = \"abc.def.ghi\"", "botToken = \"{REDACTED}\""),
            (
                r#"{"apiKey": "sk-123", "enabled": true}"#,
                r#"{"apiKey": "{REDACTED}", "enabled": true}"#,
            ),
            (
                "database.password: hunter2",
                "database.password: {REDACTED}",
            ),
            (
                "url=https://discord.com/api/webhooks/123/abc-DEF_ghi",
                "url={DISCORD_WEBHOOK}",
            ),
            ("password = \"1234567890\"", "password = \"{REDACTED}\""),
            ("rcon.password='1234'", "rcon.password='{REDACTED}'"),
        ] {
            assert_eq!(censor_config(text).as_deref(), Some(censored));
        }
    }

    #[test]
    fn leaves_out_multiline_config_secrets() {
        for text in [
            "auth:\n  token: |\n    abc\n    def\n",
            "auth:\n  password: >- # comment\r\n    abc\r\n",
            "secret = \"\"\"\nabc\ndef\"\"\"",
            "apiKey = '''\nabc'''",
        ] {
            assert_eq!(censor_config(text), None);
        }
    }

    #[test]
    fn keeps_non_secret_config_values() {
        for text in [
            "requireToken = true",
            "tokenExpiry = 3600",
            "passwordLength: 12",
            "apiKey = \"\"",
            "renderDistance = 12",
            "webhookEnabled=false",
            "description = \"\"\"\nabc\"\"\"",
        ] {
            assert_eq!(censor_config(text).as_deref(), Some(text));
        }
    }

    fn censor(text: &str) -> String {
        censor_log_text(text.to_string())
    }

    #[test]
    fn censors_ipv4_addresses() {
        assert_eq!(
            censor("Connecting to 192.168.1.20, 25565"),
            "Connecting to {IPV4_ADDRESS}, 25565"
        );
        assert_eq!(
            censor("Joined 203.0.113.7:25565"),
            "Joined {IPV4_ADDRESS}:25565"
        );
    }

    #[test]
    fn keeps_exempt_ipv4_addresses_and_versions() {
        for text in [
            "Bound to 127.0.0.1:25565",
            "Listening on 0.0.0.0",
            "Resolver 1.1.1.1 and 8.8.4.4",
            "Loaded version 1.2.3.4",
            "Loaded version: 1.2.3.4",
            "Loaded mod-1.20.1.2",
            "Loaded v1.2.3.4",
            "Value 1.2.3.45678",
            "Loaded file/Pack Demo 1.2.3.4.6.zip",
        ] {
            assert_eq!(censor(text), text);
        }
    }

    #[test]
    fn censors_ipv6_addresses() {
        assert_eq!(
            censor("Connecting to [2001:db8::1]:25565"),
            "Connecting to [{IPV6_ADDRESS}]:25565"
        );
        assert_eq!(
            censor("Address fe80::1%eth0 is up"),
            "Address {IPV6_ADDRESS} is up"
        );
    }

    #[test]
    fn keeps_loopback_ipv6_and_timestamps() {
        for text in [
            "Bound to ::1",
            "Listening on ::",
            "[12:34:56] [Server thread/INFO]: Done",
            "Caused by: java.lang.Exception",
        ] {
            assert_eq!(censor(text), text);
        }
    }

    #[test]
    fn censors_usernames_in_paths() {
        assert_eq!(
            censor(r"Loading C:\Users\Steve\AppData\Roaming\x.jar"),
            r"Loading C:\Users\{COMPUTER_USERNAME}\AppData\Roaming\x.jar"
        );
        assert_eq!(
            censor(r#"{"path":"D:\\Users\\Steve\\x.jar"}"#),
            r#"{"path":"D:\\Users\\{COMPUTER_USERNAME}\\x.jar"}"#
        );
        assert_eq!(
            censor("Loading C:/Users/Steve/x.jar"),
            "Loading C:/Users/{COMPUTER_USERNAME}/x.jar"
        );
        assert_eq!(
            censor("Loading /home/steve/.minecraft/x.jar"),
            "Loading /home/{COMPUTER_USERNAME}/.minecraft/x.jar"
        );
        assert_eq!(
            censor("Loading /Users/steve/Library/x.jar"),
            "Loading /Users/{COMPUTER_USERNAME}/Library/x.jar"
        );
        assert_eq!(
            censor("Loading (/home/steve/x.jar)"),
            "Loading (/home/{COMPUTER_USERNAME}/x.jar)"
        );
        assert_eq!(censor("Loading a/home/steve/x"), "Loading a/home/steve/x");
        assert_eq!(
            censor("USERNAME=steve PATH=x"),
            "USERNAME={COMPUTER_USERNAME} PATH=x"
        );
    }

    #[test]
    fn censors_minecraft_tokens() {
        assert_eq!(
            censor("--username Steve --accessToken eyJ.abc.def --version 1.21"),
            "--username Steve --accessToken {MINECRAFT_ACCESS_TOKEN} --version 1.21"
        );
        assert_eq!(
            censor("Setting user: Steve (Session ID is token:eyJabc:0123abcd)"),
            "Setting user: Steve (Session ID is token:{MINECRAFT_ACCESS_TOKEN}:{MINECRAFT_UUID})"
        );
        assert_eq!(
            censor(r#"{"authToken":"abc","refreshToken":"def"}"#),
            r#"{"authToken":"{AUTH_TOKEN}","refreshToken":"{REFRESH_TOKEN}"}"#
        );
    }

    #[test]
    fn keeps_placeholders_stable() {
        let censored = censor(r"C:\Users\Steve\x --accessToken abc 10.0.0.5");
        assert_eq!(censor(&censored), censored);
    }
}
