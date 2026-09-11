pub(super) fn contains(host: &str) -> bool {
	let normalized = host.trim_end_matches('.').to_ascii_lowercase();
	let mut suffix = normalized.as_str();
	loop {
		if blocklist::is_porn(suffix) {
			return true;
		}
		let Some((_, rest)) = suffix.split_once('.') else {
			return false;
		};
		suffix = rest;
	}
}
