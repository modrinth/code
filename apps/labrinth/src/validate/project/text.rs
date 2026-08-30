use std::collections::BTreeSet;
use std::sync::LazyLock;

use censor::Censor;
use linkify::{LinkFinder, LinkKind};
use regex::Regex;
use url::Url;
use whatlang::{Detector, Lang};

static PROFANITY_CENSOR: LazyLock<Censor> =
	LazyLock::new(|| Censor::Standard + Censor::Sex);

static WORD: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"[\p{L}\p{M}\p{N}]+").unwrap());
static SUMMARY_LINK_FINDER: LazyLock<LinkFinder> = LazyLock::new(|| {
	let mut finder = LinkFinder::new();
	finder.kinds(&[LinkKind::Url]).url_must_have_scheme(false);
	finder
});
static LANGUAGE_DETECTOR: LazyLock<Detector> = LazyLock::new(Detector::new);
static MARKDOWN_LINK: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"!?\[[^\]]*\]\([^)]+\)").unwrap());
static HTML_TAG: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?is)<[a-z][^>]*>").unwrap());
static CODE_BLOCK: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?s)```.*?```").unwrap());
static INLINE_CODE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"`[^`]*`").unwrap());
static MARKDOWN_IMAGE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"!\[([^\]]*)\]\([^)]+\)").unwrap());
static HTML_IMAGE: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?is)<img\b[^>]*>").unwrap());
static ALT_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r#"(?is)\balt\s*=\s*(?:"([^"]*)"|'([^']*)')"#).unwrap()
});
static DESCRIPTION_LINK: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?i)(?:https?://|www\.)[^\s<>()\]]+").unwrap()
});
static HEADER: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"(?m)^#{1,3}[\t ]+(.+?)\s*#*\s*$").unwrap());
static HTML_HEADER: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"(?is)<h[1-3]\b[^>]*>(.*?)</h[1-3]>").unwrap()
});
static INLINE_MARKDOWN: LazyLock<Regex> = LazyLock::new(|| {
	Regex::new(r"[*_~`]|!?\[([^\]]*)\]\([^)]+\)|<[^>]+>").unwrap()
});

const URL_SHORTENERS: &[&str] =
	&["bit.ly", "adf.ly", "tinyurl.com", "short.io", "is.gd"];

pub(super) fn contains_profanity(text: &str) -> bool {
	PROFANITY_CENSOR.check(text)
}

pub(super) fn profanity_count(text: &str) -> usize {
	PROFANITY_CENSOR.count(text)
}

pub(super) fn has_non_standard_text(text: &str) -> bool {
	non_standard_character_count(text) > 0
}

pub(super) fn non_standard_text_ratio(text: &str) -> f64 {
	let character_count = text.chars().count();
	if character_count == 0 {
		return 0.0;
	}

	non_standard_character_count(text) as f64 / character_count as f64
}

fn non_standard_character_count(text: &str) -> usize {
	let mut count = 0;
	let mut combining_marks = 0;
	let mut has_base = false;

	for character in text.chars() {
		let code = character as u32;
		let is_combining = is_in_ranges(
			code,
			&[
				(0x0300, 0x036f),
				(0x1ab0, 0x1aff),
				(0x1dc0, 0x1dff),
				(0x20d0, 0x20ff),
				(0xfe20, 0xfe2f),
			],
		);
		if is_combining {
			combining_marks += 1;
			if !has_base || combining_marks > 2 {
				count += 1;
			}
			continue;
		}

		combining_marks = 0;
		has_base = !character.is_whitespace();
		let allowed_control = matches!(character, '\n' | '\r' | '\t');
		let control = character.is_control() && !allowed_control;
		let private_use = is_in_ranges(
			code,
			&[(0xe000, 0xf8ff), (0xf0000, 0xffffd), (0x100000, 0x10fffd)],
		);
		let invisible = is_in_ranges(
			code,
			&[
				(0x00ad, 0x00ad),
				(0x061c, 0x061c),
				(0x200e, 0x200f),
				(0x202a, 0x202e),
				(0x2060, 0x206f),
				(0xfeff, 0xfeff),
			],
		);
		let fancy = is_in_ranges(
			code,
			&[
				(0x02b0, 0x02ff),
				(0x1d400, 0x1d7ff),
				(0x2460, 0x24ff),
				(0x2070, 0x209f),
				(0x2100, 0x214f),
				(0xfb00, 0xfb06),
				(0xff10, 0xff19),
				(0xff21, 0xff3a),
				(0xff41, 0xff5a),
				(0x1f100, 0x1f1ad),
			],
		) && !matches!(
			code,
			0x02d6
				| 0x02d7 | 0x02d8
				| 0x02d9 | 0x02da
				| 0x02db | 0x02dc
				| 0x02dd | 0x207a
				| 0x207b | 0x208a
				| 0x208b | 0x2120
				| 0x2122 | 0x2139
		);

		if control || private_use || invisible || fancy {
			count += 1;
		}
	}

	count
}

fn is_in_ranges(code: u32, ranges: &[(u32, u32)]) -> bool {
	ranges
		.iter()
		.any(|(start, end)| code >= *start && code <= *end)
}

pub(super) fn contains_spam(text: &str) -> bool {
	let normalized = text.to_lowercase();
	let mut previous = None;
	let mut repeated = 0;
	for character in normalized.chars() {
		if !character.is_whitespace() && previous == Some(character) {
			repeated += 1;
		} else {
			repeated = usize::from(!character.is_whitespace());
			previous = Some(character);
		}
		if repeated >= 8 {
			return true;
		}
	}

	let words = WORD
		.find_iter(&normalized)
		.map(|word| word.as_str())
		.collect::<Vec<_>>();
	let mut repeated_words = 1;
	for pair in words.windows(2) {
		repeated_words = if pair[0] == pair[1] {
			repeated_words + 1
		} else {
			1
		};
		if repeated_words >= 4 {
			return true;
		}
	}

	let max_phrase_words = 8.min(words.len() / 3);
	for phrase_words in (2..=max_phrase_words).rev() {
		for phrases in words.windows(phrase_words * 3) {
			if phrases[..phrase_words]
				== phrases[phrase_words..phrase_words * 2]
				&& phrases[..phrase_words]
					== phrases[phrase_words * 2..phrase_words * 3]
			{
				return true;
			}
		}
	}

	false
}

pub(super) fn contains_link_or_ip(text: &str) -> bool {
	SUMMARY_LINK_FINDER.links(text).any(|link| {
		let raw = link.as_str();
		if raw.contains("://") {
			return true;
		}

		Url::parse(&format!("https://{raw}")).is_ok_and(|url| {
			url.host_str().is_some_and(|hostname| {
				psl::domain(hostname.as_bytes())
					.is_some_and(|domain| domain.suffix().typ().is_some())
			})
		})
	})
}

pub(super) fn has_summary_formatting(summary: &str) -> bool {
	HTML_TAG.is_match(summary)
		|| MARKDOWN_LINK.is_match(summary)
		|| summary.lines().any(|line| {
			let line = line.trim_start();
			line.starts_with('#')
				|| line.starts_with('>')
				|| line.starts_with("- ")
				|| line.starts_with("* ")
				|| line.starts_with("+ ")
				|| line.starts_with("```")
				|| line.contains("**")
				|| line.contains("__")
				|| line.contains("~~")
				|| INLINE_CODE.is_match(line)
		})
}

pub(super) fn extract_description_text(markdown: &str) -> String {
	let without_code = CODE_BLOCK.replace_all(markdown, " ");
	let without_code = INLINE_CODE.replace_all(&without_code, " ");
	let with_image_alt = MARKDOWN_IMAGE.replace_all(&without_code, "$1");
	let without_links = MARKDOWN_LINK.replace_all(&with_image_alt, " ");
	let without_html = HTML_TAG.replace_all(&without_links, " ");
	without_html
		.replace(['*', '_', '~', '`', '>', '-', '|'], " ")
		.split_whitespace()
		.collect::<Vec<_>>()
		.join(" ")
}

pub(super) fn extract_description_blocks(markdown: &str) -> Vec<String> {
	CODE_BLOCK
		.replace_all(markdown, "")
		.split("\n\n")
		.map(extract_description_text)
		.filter(|block| !block.is_empty())
		.collect()
}

pub(super) fn has_long_header(markdown: &str) -> bool {
	HEADER.captures_iter(markdown).any(|captures| {
		INLINE_MARKDOWN
			.replace_all(&captures[1], "$1")
			.trim()
			.chars()
			.count() > 80
	}) || HTML_HEADER.captures_iter(markdown).any(|captures| {
		HTML_TAG
			.replace_all(&captures[1], " ")
			.split_whitespace()
			.collect::<Vec<_>>()
			.join(" ")
			.chars()
			.count() > 80
	})
}

pub(super) fn has_image_without_alt_text(markdown: &str) -> bool {
	let without_code = CODE_BLOCK.replace_all(markdown, "");
	let without_code = INLINE_CODE.replace_all(&without_code, "");
	MARKDOWN_IMAGE
		.captures_iter(&without_code)
		.any(|captures| captures[1].trim().is_empty())
		|| HTML_IMAGE.find_iter(&without_code).any(|image| {
			ALT_ATTRIBUTE
				.captures(image.as_str())
				.and_then(|captures| {
					captures.get(1).or_else(|| captures.get(2))
				})
				.is_none_or(|alt| alt.as_str().trim().is_empty())
		})
}

pub(super) fn contains_banned_description_link(markdown: &str) -> bool {
	DESCRIPTION_LINK.find_iter(markdown).any(|link| {
		let raw = link.as_str();
		let normalized = if raw.to_ascii_lowercase().starts_with("www.") {
			format!("https://{raw}")
		} else {
			raw.to_owned()
		};
		Url::parse(&normalized).is_ok_and(|url| {
			url.host_str().is_some_and(|hostname| {
				URL_SHORTENERS
					.iter()
					.any(|domain| hostname_matches_domain(hostname, domain))
			})
		})
	})
}

fn hostname_matches_domain(hostname: &str, domain: &str) -> bool {
	hostname.eq_ignore_ascii_case(domain)
		|| hostname
			.to_ascii_lowercase()
			.ends_with(&format!(".{domain}"))
}

pub(super) fn is_likely_english_summary(text: &str) -> bool {
	if !has_enough_language_content(text) {
		return true;
	}

	LANGUAGE_DETECTOR
		.detect(text)
		.is_none_or(|info| info.lang() == Lang::Eng || info.confidence() < 0.5)
}

pub(super) fn has_sufficient_english_blocks(blocks: &[String]) -> bool {
	let mut english_chunks = 0;
	let mut non_english_chunks = 0;

	for block in blocks {
		for chunk in language_chunks(block) {
			let Some(info) = LANGUAGE_DETECTOR.detect(&chunk) else {
				continue;
			};

			if info.lang() == Lang::Eng {
				english_chunks += 1;
			} else if info.confidence() >= 0.8 {
				non_english_chunks += 1;
			}
		}
	}

	let classified_chunks = english_chunks + non_english_chunks;
	classified_chunks == 0 || english_chunks * 10 >= classified_chunks * 3
}

fn language_chunks(block: &str) -> Vec<String> {
	const CHUNK_WORDS: usize = 24;
	const CHUNK_STRIDE_WORDS: usize = 12;

	let words = WORD
		.find_iter(block)
		.map(|word| word.as_str())
		.collect::<Vec<_>>();
	if words.len() < 8 {
		return Vec::new();
	}
	if words.len() <= CHUNK_WORDS {
		let chunk = words.join(" ");
		return has_enough_language_content(&chunk)
			.then_some(chunk)
			.into_iter()
			.collect();
	}

	let mut starts = BTreeSet::new();
	let mut start = 0;
	while start + 8 <= words.len() {
		starts.insert(start);
		start += CHUNK_STRIDE_WORDS;
	}
	starts.insert(words.len() - CHUNK_WORDS);

	starts
		.into_iter()
		.map(|start| {
			words[start..(start + CHUNK_WORDS).min(words.len())].join(" ")
		})
		.filter(|chunk| has_enough_language_content(chunk))
		.collect()
}

fn has_enough_language_content(text: &str) -> bool {
	WORD.find_iter(text).count() >= 8 && text.trim().chars().count() >= 35
}
