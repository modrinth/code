//! Optional Typesense-compatible query and indexing behavior.

use super::*;

use std::{
	cmp::Ordering,
	collections::BTreeMap,
};

use itertools::Itertools;
use serde_json::{Value, json};
use unicode_normalization::{
	UnicodeNormalization,
	char::is_combining_mark,
};

use crate::search::backend::{
	SearchIndex,
	typesense::Bucketing,
};

pub(super) const BUCKETED_HITS: usize = 250;
pub(super) const INTERNAL_INDEX_BATCH_SIZE: usize = 1000;
pub(super) const MAX_CANDIDATE_QUERY_LENGTH: usize = 64;
pub(super) const CANDIDATE_AGGREGATION_MULTIPLIER: usize = 128;
pub(super) const CANDIDATE_SHARD_MULTIPLIER: usize = 3;
pub(super) const SEARCH_TEXT_FIELDS: [(&str, u8); 6] = [
	("name", 15),
	("indexed_name", 15),
	("slug", 10),
	("author", 3),
	("indexed_author", 3),
	("summary", 1),
];
pub(super) const SEARCH_CANDIDATE_FIELDS: [(&str, &str); 6] = [
	("name.prefix", "_search_tokens.name"),
	("indexed_name.prefix", "indexed_name"),
	("slug.prefix", "_search_tokens.slug"),
	("author.prefix", "_search_tokens.author"),
	("indexed_author.prefix", "_search_tokens.indexed_author"),
	("summary.prefix", "_search_tokens.summary"),
];
pub(super) const CANDIDATE_SCRIPT: &str = r#"
String query = params.query;
int cost = Integer.parseInt(params.cost);
int queryLength = query.length();
for (def candidateValue : doc[params.field]) {
	String candidate = candidateValue.toString();
	int minimumLength = candidate.length() < queryLength
		? candidate.length()
		: queryLength;
	int maximumLength = queryLength + cost;
	if (maximumLength > candidate.length()) {
		maximumLength = candidate.length();
	}
	int[] previousPrevious = new int[maximumLength + 1];
	int[] previous = new int[maximumLength + 1];
	int[] current = new int[maximumLength + 1];
	for (int right = 0; right <= maximumLength; right++) {
		previous[right] = right;
	}
	for (int left = 1; left <= queryLength; left++) {
		current[0] = left;
		for (int right = 1; right <= maximumLength; right++) {
			int substitution =
				query.charAt(left - 1) == candidate.charAt(right - 1)
					? 0
					: 1;
			int distance = previous[right] + 1;
			int insertion = current[right - 1] + 1;
			if (insertion < distance) {
				distance = insertion;
			}
			int substitutionDistance =
				previous[right - 1] + substitution;
			if (substitutionDistance < distance) {
				distance = substitutionDistance;
			}
			if (
				left > 1
				&& right > 1
				&& query.charAt(left - 1) == candidate.charAt(right - 2)
				&& query.charAt(left - 2) == candidate.charAt(right - 1)
			) {
				int transposition =
					previousPrevious[right - 2] + 1;
				if (transposition < distance) {
					distance = transposition;
				}
			}
			current[right] = distance;
		}
		int[] swap = previousPrevious;
		previousPrevious = previous;
		previous = current;
		current = swap;
	}
	boolean matched = false;
	for (
		int candidateLength = minimumLength;
		candidateLength <= maximumLength && !matched;
		candidateLength++
	) {
		matched = previous[candidateLength] == cost;
	}
	if (matched) {
		emit(candidate);
	}
}
"#;

const TWO_TYPO_WEIGHT_SCALE: f64 = 16.0;

pub(super) fn project_import_batch_size() -> usize {
	ENV.TYPESENSE_IMPORT_BATCH_SIZE
}

pub(super) struct CandidateSelection {
	pub(super) terms: Vec<String>,
	pub(super) cost: usize,
	pub(super) boundary_overshoot: bool,
}

struct CandidateTerm {
	term: String,
	score: f64,
	batch_score: f64,
}

#[derive(Default)]
struct CandidateTrieNode {
	children: BTreeMap<u8, usize>,
	term: Option<usize>,
	batch_score: f64,
	path: Vec<u8>,
}

pub(super) fn text_query(
	query: &str,
	allow_typos: bool,
	two_typo_max_expansions: usize,
) -> Value {
	if query.is_empty() || query.trim() == "*" {
		return json!({"match_all": {}});
	}

	let tokens = query.split_whitespace().collect_vec();
	if tokens.is_empty() {
		return json!({"match_all": {}});
	}
	let token_queries = |token: &str, prefix: bool| {
		let mut queries = Vec::with_capacity(SEARCH_TEXT_FIELDS.len() * 4);
		let token_length = token.chars().count();
		for ((field, weight), (_, candidate_field)) in
			SEARCH_TEXT_FIELDS.into_iter().zip(SEARCH_CANDIDATE_FIELDS)
		{
			if field != "indexed_name" {
				let normalized_token =
					tokenize_candidate_text(
						token,
						field == "name" || field == "author",
					)
					.into_iter()
					.next()
					.unwrap_or_else(|| token.to_lowercase());
				queries.push(json!({
					"constant_score": {
						"filter": {
							"bool": {
								"filter": [{
									"match": {
										(field): {
											"query": token,
											"fuzziness": 0
										}
									}
								}, {
									"term": {
										(candidate_field): normalized_token
									}
								}]
							}
						},
						"boost": weight + 1
					}
				}));
			}
			if prefix {
				let prefix_query = if field == "indexed_name" {
					json!({
						"prefix": {
							(field): {"value": token.to_lowercase()}
						}
					})
				} else {
					json!({
						"match_bool_prefix": {
							(field): {"query": token}
						}
					})
				};
				queries.push(json!({
					"constant_score": {
						"filter": prefix_query,
						"boost": weight as f64 + 0.75
					}
				}));
			}
			if allow_typos {
				let max_expansions =
					if field == "indexed_name" && token_length >= 7 {
						1
					} else if field == "indexed_name" {
						4
					} else {
						2
					};
				queries.push(json!({
					"constant_score": {
						"filter": {
							"match": {
								(field): {
									"query": token,
									"fuzziness": 1,
									"prefix_length": 1,
									"max_expansions": max_expansions
								}
							}
						},
						"boost": weight as f64 + 0.5
					}
				}));
				if field == "name" {
					let transposed_prefix_query = json!({
						"match": {
							"name.prefix": {
								"query": token,
								"fuzziness": 1,
								"prefix_length": 1,
								"max_expansions": 4,
								"fuzzy_transpositions": true
							}
						}
					});
					queries.push(json!({
						"constant_score": {
							"filter": &transposed_prefix_query,
							"boost": weight as f64 + 0.25
						}
					}));
					queries.push(json!({
						"constant_score": {
							"filter": {
								"bool": {
									"must": [
										transposed_prefix_query,
										{
											"match": {
												"name": {
													"query": token,
													"fuzziness": 2,
													"prefix_length": 1,
													"max_expansions": 24
												}
											}
										}
									],
									"must_not": [{
										"match": {
											"name.prefix": {
												"query": token,
												"fuzziness": 1,
												"prefix_length": 1,
												"max_expansions": 4,
												"fuzzy_transpositions": false
											}
										}
									}]
								}
							},
							"boost": weight as f64 + 0.3
						}
					}));
				}
				queries.push(json!({
					"constant_score": {
						"filter": {
							"match": {
								(field): {
									"query": token,
									"fuzziness": "AUTO:4,7",
									"prefix_length": 1,
									"max_expansions":
										two_typo_max_expansions
								}
							}
						},
						"boost": weight as f64 / TWO_TYPO_WEIGHT_SCALE
					}
				}));
			}
		}
		if allow_typos && tokens.len() == 1 {
			let chars = token.chars().collect_vec();
			if chars.len() >= 8 {
				let split_at = chars.len().div_ceil(2);
				let left = chars[..split_at].iter().collect::<String>();
				let right = chars[split_at..].iter().collect::<String>();
				queries.push(json!({
					"constant_score": {
						"filter": {
							"intervals": {
								"name": {
									"all_of": {
										"ordered": true,
										"max_gaps": 0,
										"intervals": [
											{
												"fuzzy": {
													"term": left,
													"fuzziness": 1,
													"prefix_length": 1,
													"transpositions": true
												}
											},
											{
												"prefix": {
													"prefix": right
												}
											}
										]
									}
								}
							}
						},
						"boost": 15.25
					}
				}));
			}
		}
		queries
	};
	let queries_by_token = tokens
		.iter()
		.enumerate()
		.map(|(index, token)| {
			let is_last = index == tokens.len() - 1;
			token_queries(token, is_last)
		})
		.collect_vec();
	if tokens.len() == 1 {
		json!({
			"dis_max": {
				"queries": queries_by_token
					.into_iter()
					.flatten()
					.collect_vec(),
				"tie_breaker": 0
			}
		})
	} else {
		json!({
			"bool": {
				"must": queries_by_token
					.into_iter()
					.map(|queries| {
						json!({
							"dis_max": {
								"queries": queries,
								"tie_breaker": 0
							}
						})
					})
					.collect_vec()
			}
		})
	}
}

pub(super) fn uses_text_match_bucketing(index: SearchIndex) -> bool {
	matches!(
		index,
		SearchIndex::Relevance
			| SearchIndex::MinecraftJavaServerVerifiedPlays2w
			| SearchIndex::MinecraftJavaServerPlayersOnline
	)
}

pub(super) fn bucket_size(
	bucketing: &Bucketing,
	hit_count: usize,
) -> Option<usize> {
	if hit_count == 0 {
		return None;
	}

	match bucketing {
		Bucketing::Buckets(count) => {
			let count = usize::try_from(*count).ok()?;
			if count == 0 {
				return None;
			}
			Some(hit_count.div_ceil(count))
		}
		Bucketing::BucketSize(size) => {
			let size = usize::try_from(*size).ok()?;
			(size > 0).then_some(size)
		}
	}
}

fn compare_descending_sort_values(left: &Value, right: &Value) -> Ordering {
	match (left, right) {
		(Value::Null, Value::Null) => Ordering::Equal,
		(Value::Null, _) => Ordering::Greater,
		(_, Value::Null) => Ordering::Less,
		(Value::Number(left), Value::Number(right)) => right
			.as_f64()
			.and_then(|right| {
				left.as_f64()
					.and_then(|left| right.partial_cmp(&left))
			})
			.unwrap_or(Ordering::Equal),
		(Value::Bool(left), Value::Bool(right)) => right.cmp(left),
		(Value::String(left), Value::String(right)) => right.cmp(left),
		_ => Ordering::Equal,
	}
}

fn compare_bucket_hits(left: &Value, right: &Value) -> Ordering {
	let Some(left) = left["sort"].as_array() else {
		return Ordering::Equal;
	};
	let Some(right) = right["sort"].as_array() else {
		return Ordering::Equal;
	};

	left.iter()
		.zip(right)
		.skip(1)
		.take(left.len().saturating_sub(2))
		.map(|(left, right)| compare_descending_sort_values(left, right))
		.find(|ordering| !ordering.is_eq())
		.unwrap_or(Ordering::Equal)
}

pub(super) fn rerank_bucketed_hits(
	hits: &mut [Value],
	bucketing: &Bucketing,
	candidate_count: usize,
	used_fuzzy_query: bool,
) {
	let use_narrow_full_bucket = candidate_count >= 500;
	let candidate_count = candidate_count.min(BUCKETED_HITS);
	let Some(mut bucket_size) = bucket_size(bucketing, candidate_count) else {
		return;
	};
	if used_fuzzy_query
		&& use_narrow_full_bucket
		&& matches!(bucketing, Bucketing::Buckets(_))
	{
		bucket_size = bucket_size.saturating_sub(1).max(1);
	}
	let hit_count = hits.len().min(BUCKETED_HITS);
	for bucket in hits[..hit_count].chunks_mut(bucket_size) {
		bucket.sort_by(compare_bucket_hits);
	}
}

pub(super) fn selected_candidate_text_query(terms: &[String]) -> Value {
	json!({
		"dis_max": {
			"queries": SEARCH_TEXT_FIELDS
				.into_iter()
				.map(|(field, weight)| {
					json!({
						"constant_score": {
							"filter": {
								"terms": {(field): terms}
							},
							"boost": weight
						}
					})
				})
				.collect_vec(),
			"tie_breaker": 0
		}
	})
}

pub(super) fn stemmed_single_name_query(query: &str) -> Value {
	json!({
		"constant_score": {
			"filter": {
				"bool": {
					"must": [{
						"prefix": {
							"_search_tokens.indexed_name": {
								"value": query.to_lowercase()
							}
						}
					}, {
						"match": {
							"indexed_name": {
								"query": query,
								"fuzziness": 0
							}
						}
					}],
					"must_not": [{
						"wildcard": {
							"_search_tokens.indexed_name": {
								"value": "*-*"
							}
						}
					}]
				}
			},
			"boost": 16
		}
	})
}

pub(super) fn candidate_token_text_query(
	terms: &[String],
	cost: usize,
	query_length: usize,
) -> Value {
	let (short_terms, long_terms): (Vec<_>, Vec<_>) = terms
		.iter()
		.cloned()
		.partition(|term| term.chars().count() <= query_length + 1);
	let mut queries = Vec::new();
	for ((_, field), (_, weight)) in SEARCH_CANDIDATE_FIELDS
		.into_iter()
		.zip(SEARCH_TEXT_FIELDS)
	{
		let mut add_query = |terms: &[String], boost: f64| {
			if !terms.is_empty() {
				queries.push(json!({
					"constant_score": {
						"filter": {
							"terms": {(field): terms}
						},
						"boost": boost
					}
				}));
			}
		};
		match cost {
			0 => add_query(terms, weight as f64 + 0.75),
			1 => {
				add_query(&short_terms, weight as f64 + 0.5);
				add_query(&long_terms, weight as f64 + 0.25);
			}
			_ => {
				add_query(terms, weight as f64 / TWO_TYPO_WEIGHT_SCALE);
			}
		}
	}
	json!({
		"dis_max": {
			"queries": queries,
			"tie_breaker": 0
		}
	})
}

pub(super) fn rank_candidate_buckets(
	buckets: &[Value],
	query: &str,
	cost: usize,
	max_candidates: usize,
) -> Vec<String> {
	let candidates = buckets
		.iter()
		.filter(|bucket| {
			bucket["allowed"]["doc_count"]
				.as_u64()
				.unwrap_or_default()
				> 0
		})
		.filter_map(|bucket| {
			let term = bucket["key"].as_str()?.to_string();
			let score = bucket["max_score"]["value"]
				.as_f64()
				.unwrap_or_default() as f32 as f64;
			let batch_score = bucket["max_batch_score"]["value"]
				.as_f64()
				.unwrap_or(score) as f32 as f64;
			Some(CandidateTerm {
				term,
				score,
				batch_score,
			})
		})
		.collect_vec();
	if candidates.is_empty() {
		return Vec::new();
	}

	let query_chars = query.chars().collect_vec();
	let mut groups = BTreeMap::<String, Vec<usize>>::new();
	for (index, candidate) in candidates.iter().enumerate() {
		let candidate_chars = candidate.term.chars().collect_vec();
		let minimum_length = candidate_chars.len().min(query_chars.len());
		let maximum_length =
			(query_chars.len() + cost).min(candidate_chars.len());
		let prefix = (minimum_length..=maximum_length)
			.find(|length| {
				damerau_levenshtein_distance(
					&query_chars,
					&candidate_chars[..*length],
				) == cost
			})
			.map(|length| {
				candidate_chars[..length].iter().collect::<String>()
			})
			.unwrap_or_else(|| candidate.term.clone());
		groups.entry(prefix).or_default().push(index);
	}

	let discovery_limit = max_candidates.saturating_mul(4);
	let mut discovered = Vec::with_capacity(discovery_limit);
	for (prefix, candidate_indices) in groups.iter().rev() {
		if discovered.len() >= discovery_limit {
			break;
		}
		let mut nodes = vec![CandidateTrieNode::default()];
		for candidate_index in candidate_indices {
			let candidate = &candidates[*candidate_index];
			let mut node_index = 0;
			nodes[node_index].batch_score = nodes[node_index]
				.batch_score
				.max(candidate.batch_score);
			for byte in candidate.term.as_bytes()[prefix.len()..]
				.iter()
				.copied()
			{
				let child_index =
					if let Some(index) = nodes[node_index]
						.children
						.get(&byte)
						.copied()
					{
						index
					} else {
						let index = nodes.len();
						let mut path = nodes[node_index].path.clone();
						path.push(byte);
						nodes.push(CandidateTrieNode {
							path,
							..CandidateTrieNode::default()
						});
						nodes[node_index].children.insert(byte, index);
						index
					};
				node_index = child_index;
				nodes[node_index].batch_score = nodes[node_index]
					.batch_score
					.max(candidate.batch_score);
			}
			let mut path = nodes[node_index].path.clone();
			path.push(0);
			let terminal_index = nodes.len();
			nodes.push(CandidateTrieNode {
				term: Some(*candidate_index),
				batch_score: candidate.score,
				path,
				..CandidateTrieNode::default()
			});
			nodes[node_index].children.insert(0, terminal_index);
		}

		let compressed_node = |mut index: usize| {
			while nodes[index].term.is_none()
				&& nodes[index].children.len() == 1
			{
				index = *nodes[index]
					.children
					.values()
					.next()
					.expect("candidate trie node has one child");
			}
			index
		};
		let mut frontier = vec![compressed_node(0)];
		while !frontier.is_empty() && discovered.len() < discovery_limit {
			let best_position = frontier
				.iter()
				.enumerate()
				.max_by(|(_, left), (_, right)| {
					nodes[**left]
						.batch_score
						.total_cmp(&nodes[**right].batch_score)
						.then_with(|| {
							nodes[**right].path.cmp(&nodes[**left].path)
						})
				})
				.map(|(position, _)| position)
				.unwrap_or_default();
			let node_index = frontier.swap_remove(best_position);
			if let Some(candidate_index) = nodes[node_index].term {
				discovered.push(candidate_index);
			}
			frontier.extend(
				nodes[node_index]
					.children
					.values()
					.copied()
					.map(compressed_node),
			);
		}
	}

	discovered.sort_by(|left, right| {
		candidates[*right]
			.score
			.total_cmp(&candidates[*left].score)
			.then_with(|| {
				candidates[*left].term.cmp(&candidates[*right].term)
			})
	});
	let mut ranked = discovered
		.into_iter()
		.take(max_candidates.saturating_add(8))
		.map(|index| candidates[index].term.clone())
		.collect_vec();
	if cost == 0 && candidates.iter().any(|candidate| candidate.term == query)
	{
		ranked.retain(|term| term != query);
		ranked.insert(0, query.to_string());
		ranked.truncate(max_candidates.saturating_add(8));
	}
	ranked
}

pub(super) fn tokenize_search_name(name: &str) -> Vec<String> {
	tokenize_candidate_text(name, true)
}

pub(super) fn tokenize_candidate_text(
	text: &str,
	separate_hyphens: bool,
) -> Vec<String> {
	let mut tokens = Vec::new();
	let mut token = String::new();
	let characters = text.chars().collect_vec();
	for (index, character) in characters.iter().copied().enumerate() {
		if character == ' '
			|| character == '\n'
			|| (separate_hyphens && character == '-')
		{
			if !token.is_empty() {
				tokens.push(std::mem::take(&mut token));
			}
		} else if character.is_ascii_alphanumeric() {
			token.extend(character.to_lowercase());
		} else if !character.is_ascii() {
			let joins_word = matches!(
				character,
				'\u{2010}'..='\u{2015}' | '\u{2212}'
			) && index > 0
				&& characters[index - 1].is_alphanumeric()
				&& characters
					.get(index + 1)
					.is_some_and(|next| next.is_alphanumeric());
			if joins_word {
				continue;
			}
			if character == '\u{202f}' {
				token.push(character);
				continue;
			}
			let mut encoded = [0; 4];
			for normalized in character
				.encode_utf8(&mut encoded)
				.nfkd()
				.filter(|normalized| !is_combining_mark(*normalized))
			{
				if drops_transliterated_punctuation(normalized) {
					continue;
				}
				if normalized.is_ascii_alphanumeric()
					|| !normalized.is_ascii()
				{
					token.extend(normalized.to_lowercase());
				}
			}
		}
	}
	if !token.is_empty() {
		tokens.push(token);
	}
	tokens.into_iter().unique().collect()
}

fn drops_transliterated_punctuation(character: char) -> bool {
	matches!(
		character,
		'\u{00ab}'
			| '\u{00b7}'
			| '\u{00bb}'
			| '\u{2018}'..='\u{201f}'
			| '\u{2026}'
			| '\u{2032}'..='\u{2037}'
	)
}

pub(super) fn damerau_levenshtein_distance(
	left: &[char],
	right: &[char],
) -> usize {
	let mut distances = vec![vec![0; right.len() + 1]; left.len() + 1];
	for (index, row) in distances.iter_mut().enumerate() {
		row[0] = index;
	}
	for index in 0..=right.len() {
		distances[0][index] = index;
	}

	for left_index in 1..=left.len() {
		for right_index in 1..=right.len() {
			let substitution_cost =
				usize::from(left[left_index - 1] != right[right_index - 1]);
			let mut distance = distances[left_index - 1][right_index]
				.saturating_add(1)
				.min(
					distances[left_index][right_index - 1]
						.saturating_add(1),
				)
				.min(
					distances[left_index - 1][right_index - 1]
						.saturating_add(substitution_cost),
				);
			if left_index > 1
				&& right_index > 1
				&& left[left_index - 1] == right[right_index - 2]
				&& left[left_index - 2] == right[right_index - 1]
			{
				distance = distance.min(
					distances[left_index - 2][right_index - 2]
						.saturating_add(1),
				);
			}
			distances[left_index][right_index] = distance;
		}
	}
	distances[left.len()][right.len()]
}

impl Elasticsearch {
	pub(super) async fn search_for_project_raw_typesense_parity(
		&self,
		info: &SearchRequest,
	) -> Result<SearchResults, ApiError> {
		let parsed = parse_search_request(info)?;
		let search_sort =
			parse_search_index(parsed.index, info.new_filters.as_deref())?;
		let filter = Self::build_filter(info)?;

		let mut filters =
			vec![json!({"term": {"document_type": "project"}})];
		if let Some(filter) = &filter {
			filters.push(filter.query.clone());
		}
		let alias = self.config.alias_name();
		let strict_query = json!({
			"bool": {
				"must": [text_query(parsed.query, false, 2)],
				"filter": &filters
			}
		});
		let fuzzy_query = parsed
			.query
			.split_whitespace()
			.next()
			.map(|_| {
				json!({
					"bool": {
						"must": [text_query(parsed.query, true, 2)],
						"filter": &filters
					}
				})
			});
		let broader_fuzzy_query = parsed
			.query
			.split_whitespace()
			.next()
			.map(|_| {
				json!({
					"bool": {
						"must": [text_query(parsed.query, true, 3)],
						"filter": &filters
					}
				})
			});
		let mut query = strict_query;
		let sort = Self::sort(search_sort.index);
		let bucketed_relevance = uses_text_match_bucketing(
			search_sort.index,
		) && parsed.offset < BUCKETED_HITS
			&& bucket_size(
				&info.typesense_config.bucketing,
				BUCKETED_HITS,
			)
			.is_some();
		let requested_end =
			parsed.offset.saturating_add(parsed.hits_per_page);
		let remaining_offset =
			if bucketed_relevance { 0 } else { parsed.offset };
		let fetch_size = if bucketed_relevance {
			BUCKETED_HITS.max(requested_end)
		} else {
			parsed.hits_per_page
		};
		let mut used_fuzzy_query = false;
		let mut used_typesense_candidates = false;
		let mut typesense_candidate_count_hint = None;
		let mut typesense_candidate_token_query = None;
		let mut typesense_candidate_documents_query = None;
		let mut typesense_candidate_cost = None;
		let candidate_query_length = parsed
			.query
			.split_whitespace()
			.next()
			.filter(|_| parsed.query.split_whitespace().count() == 1)
			.map(|token| token.chars().count())
			.unwrap_or_default();
		let should_select_typesense_candidates =
			(3..=MAX_CANDIDATE_QUERY_LENGTH)
				.contains(&candidate_query_length);
		if should_select_typesense_candidates
			&& let Some(selection) = self
				.select_typesense_candidates(
					&alias,
					&filters,
					parsed.query,
					info.typesense_config.max_candidates,
				)
				.await?
		{
			let normalized_candidate_query = parsed.query.to_lowercase();
			let filter_trailing_prefix = selection.cost > 0
				&& filters.len() == 1
				&& candidate_query_length < 5;
			let candidate_terms = selection
				.terms
				.iter()
				.filter(|term| {
					!filter_trailing_prefix
						|| term.chars().count() >= candidate_query_length
						|| !normalized_candidate_query.starts_with(*term)
				})
				.take(
					info.typesense_config
						.max_candidates
						.clamp(1, 128)
						.saturating_add(usize::from(
							selection.boundary_overshoot,
						)),
				)
				.cloned()
				.collect_vec();
			debug!(
				query = parsed.query,
				cost = selection.cost,
				?candidate_terms,
				"selected Elasticsearch typo candidates"
			);
			let selected_query = json!({
				"bool": {
					"must": [
						selected_candidate_text_query(
							&candidate_terms,
						)
					],
					"filter": &filters
				}
			});
			let candidate_token_query =
				candidate_token_text_query(
					&candidate_terms,
					selection.cost,
					candidate_query_length,
				);
			let candidate_documents_query = json!({
				"bool": {
					"must": [&candidate_token_query],
					"filter": &filters
				}
			});
			let should_count_candidates = selection.cost == 0;
			let selected_candidate_count =
				if should_count_candidates {
					Some(
						self.count_matches(
							&alias,
							&candidate_documents_query,
						)
						.await?,
					)
				} else {
					None
				};
			typesense_candidate_count_hint = selected_candidate_count;
			typesense_candidate_token_query = Some(candidate_token_query);
			typesense_candidate_documents_query =
				Some(candidate_documents_query.clone());
			typesense_candidate_cost = Some(selection.cost);
			if selection.cost == 0 {
				query = json!({
					"bool": {
						"must": [{
							"dis_max": {
								"queries": [
									text_query(
										parsed.query,
										false,
										2,
									),
									stemmed_single_name_query(
										parsed.query,
									)
								],
								"tie_breaker": 0
							}
						}],
						"filter": &filters
					}
				});
			}
			let use_candidate_ranking = selection.cost > 0
				|| (selection.cost == 0
					&& candidate_query_length <= 4
					&& selected_candidate_count
						.is_some_and(|count| {
							count <= BUCKETED_HITS
						}));
			if use_candidate_ranking {
				query = if selection.cost > 0 {
					candidate_documents_query.clone()
				} else {
					selected_query
				};
				used_fuzzy_query = selection.cost > 0;
				used_typesense_candidates = true;
			}
		}
		let deep_pagination = remaining_offset
			.saturating_add(fetch_size)
			> MAX_RESULT_WINDOW;
		let mut adjusted_candidate_query = false;
		if deep_pagination
			&& !used_typesense_candidates
			&& let (
				Some(candidate_token_query),
				Some(candidate_documents_query),
				Some(candidate_cost),
				Some(candidate_count),
			) = (
				typesense_candidate_token_query.as_ref(),
				typesense_candidate_documents_query.as_ref(),
				typesense_candidate_cost,
				typesense_candidate_count_hint,
			)
		{
			let current_count = self.count_matches(&alias, &query).await?;
			if let Some(adjusted) = self
				.adjust_candidate_query(
					&alias,
					&query,
					&sort,
					&filters,
					candidate_token_query,
					candidate_documents_query,
					candidate_cost,
					candidate_count,
					current_count,
					candidate_query_length,
				)
				.await?
			{
				query = adjusted;
				adjusted_candidate_query = true;
			}
		}
		if deep_pagination
			&& let Some(fuzzy_query) = &fuzzy_query
			&& !self.has_matches(&alias, &query).await?
		{
			query = fuzzy_query.clone();
			query = self
				.add_fuzzy_name_promotions(
					&alias,
					query,
					&filters,
					parsed.query,
				)
				.await?;
			used_fuzzy_query = true;
		}

		let mut body = if deep_pagination {
			self
				.execute_deep_search(
					&alias,
					&query,
					&sort,
					remaining_offset,
					fetch_size,
				)
				.await?
		} else {
			let request_body = Self::search_body(
				&query,
				&sort,
				remaining_offset,
				fetch_size,
				true,
				None,
				!bucketed_relevance,
			);
			self
				.execute_search(
					&alias,
					&request_body,
					bucketed_relevance,
				)
				.await?
		};
		if !used_typesense_candidates
			&& !deep_pagination
			&& body["hits"]["total"]["value"].as_u64() == Some(0)
			&& let Some(fuzzy_query) = fuzzy_query
		{
			query = fuzzy_query;
			query = self
				.add_fuzzy_name_promotions(
					&alias,
					query,
					&filters,
					parsed.query,
				)
				.await?;
			used_fuzzy_query = true;
			let request_body = Self::search_body(
				&query,
				&sort,
				remaining_offset,
				fetch_size,
				true,
				None,
				!bucketed_relevance,
			);
			body = self
				.execute_search(
					&alias,
					&request_body,
					bucketed_relevance,
				)
				.await?;
		}

		if !used_typesense_candidates
			&& !adjusted_candidate_query
			&& !deep_pagination
			&& let (
				Some(candidate_token_query),
				Some(candidate_documents_query),
				Some(candidate_cost),
				Some(candidate_count),
			) = (
				typesense_candidate_token_query.as_ref(),
				typesense_candidate_documents_query.as_ref(),
				typesense_candidate_cost,
				typesense_candidate_count_hint,
			)
		{
			let current_count = body["hits"]["total"]["value"]
				.as_u64()
				.unwrap_or_default() as usize;
			if let Some(adjusted) = self
				.adjust_candidate_query(
					&alias,
					&query,
					&sort,
					&filters,
					candidate_token_query,
					candidate_documents_query,
					candidate_cost,
					candidate_count,
					current_count,
					candidate_query_length,
				)
				.await?
			{
				query = adjusted;
				let request_body = Self::search_body(
					&query,
					&sort,
					remaining_offset,
					fetch_size,
					true,
					None,
					!bucketed_relevance,
				);
				body = self
					.execute_search(
						&alias,
						&request_body,
						bucketed_relevance,
					)
					.await?;
			}
		}

		let total_hits = body["hits"]["total"]["value"]
			.as_u64()
			.unwrap_or_default() as usize;
		let bucketed_candidate_count = if let Some(candidate_count) =
			typesense_candidate_count_hint
		{
			candidate_count
		} else if bucketed_relevance
			&& used_fuzzy_query
			&& !used_typesense_candidates
			&& total_hits < BUCKETED_HITS
		{
			if let Some(broader_fuzzy_query) = &broader_fuzzy_query {
				let broader_count =
					self.count_matches(&alias, broader_fuzzy_query).await?;
				broader_count.min(
					total_hits.saturating_add(total_hits.div_ceil(5)),
				)
			} else {
				total_hits
			}
		} else {
			total_hits
		};
		if bucketed_relevance {
			let project_ids = {
				let hits = body["hits"]["hits"]
					.as_array_mut()
					.ok_or_else(|| {
						ApiError::Internal(eyre!(
							"elasticsearch search hits were not an array"
						))
					})?;
				rerank_bucketed_hits(
					hits,
					&info.typesense_config.bucketing,
					bucketed_candidate_count,
					used_fuzzy_query && !used_typesense_candidates,
				);
				hits.iter()
					.skip(parsed.offset)
					.take(parsed.hits_per_page)
					.filter_map(|hit| {
						hit["sort"]
							.as_array()
							.and_then(|sort| sort.last())
							.and_then(Value::as_str)
							.map(ToOwned::to_owned)
					})
					.collect_vec()
			};

			if project_ids.is_empty() {
				body["hits"]["hits"] = Value::Array(Vec::new());
			} else {
				let mut selected_filters = filters.clone();
				selected_filters.push(
					json!({"terms": {"project_id": &project_ids}}),
				);
				let selected_query =
					json!({"bool": {"filter": selected_filters}});
				let selected_body = Self::search_body(
					&selected_query,
					&sort,
					0,
					project_ids.len(),
					false,
					None,
					true,
				);
				let mut selected_body = self
					.execute_search(&alias, &selected_body, false)
					.await?;
				let hits = selected_body["hits"]["hits"]
					.as_array_mut()
					.ok_or_else(|| {
						ApiError::Internal(eyre!(
							"elasticsearch search hits were not an array"
						))
					})?;
				let mut hits_by_project_id = std::mem::take(hits)
					.into_iter()
					.filter_map(|hit| {
						let project_id = hit["sort"]
							.as_array()
							.and_then(|sort| sort.last())
							.and_then(Value::as_str)?
							.to_string();
						Some((project_id, hit))
					})
					.collect::<HashMap<_, _>>();
				*hits = project_ids
					.into_iter()
					.filter_map(|project_id| {
						hits_by_project_id.remove(&project_id)
					})
					.collect();
				body = selected_body;
			}
		}
		let hits = body["hits"]["hits"]
			.as_array()
			.into_iter()
			.flatten()
			.filter_map(|hit| {
				let mut document = hit["_source"].clone();
				let object = document.as_object_mut()?;
				object.remove("document_type");
				object.remove("_search_tokens");
				if filter
					.as_ref()
					.is_some_and(|filter| filter.has_version_filter)
				{
					if let Some(version_id) = matching_version_id(hit) {
						object.insert(
							"version_id".to_string(),
							Value::String(version_id),
						);
					}
				}

				let metadata = info.show_metadata.then(|| {
					json!({
						"score": hit["_score"],
						"sort": hit["sort"]
					})
				});
				let mut result: ResultSearchProject =
					serde_json::from_value::<UploadSearchProject>(document)
						.ok()?
						.into();
				result.search_metadata = metadata;
				Some(result)
			})
			.collect();

		Ok(SearchResults {
			hits,
			page: parsed.page,
			hits_per_page: parsed.hits_per_page,
			total_hits,
		})
	}

}

impl Elasticsearch {
	async fn adjust_candidate_query(
		&self,
		alias: &str,
		query: &Value,
		sort: &[Value],
		filters: &[Value],
		candidate_token_query: &Value,
		candidate_documents_query: &Value,
		candidate_cost: usize,
		candidate_count: usize,
		current_count: usize,
		candidate_query_length: usize,
	) -> Result<Option<Value>, ApiError> {
		let candidate_counts_differ = current_count != candidate_count;
		let constrain_to_candidates = candidate_counts_differ
			&& candidate_count <= BUCKETED_HITS;
		let restrict_to_candidate_terms = (candidate_cost > 0
			&& !constrain_to_candidates
			&& (!candidate_counts_differ || filters.len() > 1))
			|| (candidate_cost == 0 && candidate_count < current_count);
		let promote_candidates = candidate_cost > 0
			&& (candidate_query_length >= 5
				|| candidate_count <= BUCKETED_HITS);
		let promote_filtered_mismatch =
			candidate_counts_differ && filters.len() > 1;
		if !constrain_to_candidates
			&& !promote_candidates
			&& !promote_filtered_mismatch
			&& !restrict_to_candidate_terms
		{
			return Ok(None);
		}

		let candidate_body = Self::search_body(
			candidate_documents_query,
			sort,
			0,
			candidate_count.min(MAX_RESULT_WINDOW),
			false,
			None,
			false,
		);
		let candidate_body =
			self.execute_search(alias, &candidate_body, true).await?;
		let candidate_project_ids = candidate_body["hits"]["hits"]
			.as_array()
			.into_iter()
			.flatten()
			.filter_map(|hit| {
				hit["sort"]
					.as_array()
					.and_then(|sort| sort.last())
					.and_then(Value::as_str)
					.map(ToOwned::to_owned)
			})
			.collect_vec();
		if candidate_project_ids.is_empty() {
			return Ok(None);
		}
		let promoted_candidate_project_ids = candidate_project_ids
			.iter()
			.take(BUCKETED_HITS)
			.collect_vec();

		let candidate_scoring_query = json!({
			"bool": {
				"must": [candidate_token_query],
				"filter": [{
					"terms": {
						"project_id": promoted_candidate_project_ids
					}
				}]
			}
		});
		let promoted_query = json!({
			"dis_max": {
				"queries": [
					query,
					candidate_scoring_query,
					{
						"constant_score": {
							"filter": {
								"bool": {
									"filter": [{
										"term": {
											"document_type": "project"
										}
									}, {
										"terms": {
											"project_id":
												&candidate_project_ids
										}
									}]
								}
							},
							"boost": 0
						}
					}
				],
				"tie_breaker": 0
			}
		});
		let adjusted = if constrain_to_candidates {
			json!({
				"bool": {
					"must": [promoted_query],
					"filter": [{
						"terms": {
							"project_id": &candidate_project_ids
						}
					}]
				}
			})
		} else if restrict_to_candidate_terms {
			json!({
				"bool": {
					"must": [promoted_query],
					"filter": [candidate_token_query]
				}
			})
		} else {
			promoted_query
		};
		Ok(Some(adjusted))
	}

	async fn select_typesense_candidates(
		&self,
		alias: &str,
		filters: &[Value],
		raw_query: &str,
		max_candidates: usize,
	) -> Result<Option<CandidateSelection>, ApiError> {
		let mut query_tokens = raw_query.split_whitespace();
		let Some(query) = query_tokens.next() else {
			return Ok(None);
		};
		if query_tokens.next().is_some() {
			return Ok(None);
		}

		let query = query.to_lowercase();
		let query_length = query.chars().count();
		if !(3..=MAX_CANDIDATE_QUERY_LENGTH)
			.contains(&query_length)
		{
			return Ok(None);
		}

		let maximum_cost: usize = if query_length >= 7 {
			2
		} else if query_length >= 4 {
			1
		} else {
			0
		};
		let max_candidates = max_candidates.clamp(1, 128);
		let aggregation_size = max_candidates
			.saturating_mul(CANDIDATE_AGGREGATION_MULTIPLIER);
		let shard_size = aggregation_size
			.saturating_mul(CANDIDATE_SHARD_MULTIPLIER);
		let native_exact_prefix = query
			.chars()
			.all(|character| character.is_ascii_alphanumeric());

		for cost in 0..=maximum_cost {
			if query_length > 10 && cost > 0 && filters.len() == 1 {
				break;
			}
			let mut runtime_mappings = Map::new();
			let mut aggregations = Map::new();
			for (
				index,
				((prefix_field, candidate_field), _),
			) in SEARCH_CANDIDATE_FIELDS
				.into_iter()
				.zip(SEARCH_TEXT_FIELDS)
				.enumerate()
			{
				let name = format!("candidate_{index}");
				let candidate_prefilter = if query_length > 10 && cost == 0 {
					json!({
						"wildcard": {
							(candidate_field): {
								"value": format!("{query}*")
							}
						}
					})
				} else if query_length > 10 {
					let prefix_query =
						query.chars().take(10).collect::<String>();
					json!({
						"match": {
							(prefix_field): {
								"query": prefix_query,
								"fuzziness": cost.saturating_add(1).min(2),
								"prefix_length": 0,
								"max_expansions": 1024,
								"fuzzy_transpositions": true
							}
						}
					})
				} else {
					json!({
						"match": {
							(prefix_field): {
								"query": &query,
								"fuzziness": cost,
								"prefix_length": 0,
								"max_expansions": 1024,
								"fuzzy_transpositions": true
							}
						}
					})
				};
				let aggregation_field = if cost == 0
					&& native_exact_prefix
				{
					candidate_field.to_string()
				} else {
					runtime_mappings.insert(
						name.clone(),
						json!({
							"type": "keyword",
							"script": {
								"source": CANDIDATE_SCRIPT,
								"params": {
									"query": &query,
									"cost": cost.to_string(),
									"field": candidate_field
								}
							}
						}),
					);
					name.clone()
				};
				let mut terms_aggregation = json!({
					"field": aggregation_field,
					"size": aggregation_size,
					"shard_size": shard_size,
					"order": [
						{"max_batch_score": "desc"},
						{"_key": "asc"}
					]
				});
				if cost == 0 && native_exact_prefix {
					terms_aggregation["include"] =
						Value::String(format!("{query}.*"));
				}
				aggregations.insert(
					name.clone(),
					json!({
						"filter": candidate_prefilter,
						"aggs": {
							"values": {
								"terms": terms_aggregation,
								"aggs": {
									"max_score": {
										"max": {"field": "log_downloads"}
									},
									"max_batch_score": {
										"max": {
											"field": "_search_tokens.batch_score"
										}
									},
									"allowed": {
										"filter": {
											"bool": {"filter": filters}
										}
									}
								}
							}
						}
					}),
				);
			}

			let mut body = json!({
				"_source": false,
				"size": 0,
				"track_total_hits": false,
				"query": {
					"term": {"document_type": "project"}
				},
				"aggs": aggregations
			});
			if !runtime_mappings.is_empty() {
				body["runtime_mappings"] = Value::Object(runtime_mappings);
			}
			let response = self.execute_search(alias, &body, false).await?;
			let selection_limit = max_candidates.saturating_add(8);
			let mut terms = Vec::with_capacity(selection_limit);
			let mut seen_terms = HashSet::with_capacity(selection_limit);
			let mut boundary_overshoot = false;
			for index in 0..SEARCH_CANDIDATE_FIELDS.len() {
				let terms_before_field = terms.len();
				let name = format!("candidate_{index}");
				let buckets = response["aggregations"][&name]["values"]
					["buckets"]
					.as_array()
					.map(Vec::as_slice)
					.unwrap_or_default();
				let ranked_terms = rank_candidate_buckets(
					buckets,
					&query,
					cost,
					max_candidates,
				);
				for term in ranked_terms {
					if seen_terms.insert(term.clone()) {
						terms.push(term);
					}
					if terms.len() >= selection_limit {
						break;
					}
				}
				if terms.len() >= max_candidates {
					boundary_overshoot =
						index == SEARCH_CANDIDATE_FIELDS.len() - 1
						&& terms_before_field > 0
						&& terms_before_field < max_candidates
						&& terms.len() > max_candidates;
					break;
				}
			}

			if !terms.is_empty() {
				if cost == 0
					&& let Some(index) =
						terms.iter().position(|term| term == &query)
				{
					let exact = terms.remove(index);
					terms.insert(0, exact);
				}
				return Ok(Some(CandidateSelection {
					terms,
					cost,
					boundary_overshoot,
				}));
			}
		}

		Ok(None)
	}

	async fn add_fuzzy_name_promotions(
		&self,
		alias: &str,
		query: Value,
		filters: &[Value],
		raw_query: &str,
	) -> Result<Value, ApiError> {
		let mut tokens = raw_query.split_whitespace();
		let Some(token) = tokens.next() else {
			return Ok(query);
		};
		if tokens.next().is_some() {
			return Ok(query);
		}

		let normalized_query = token.to_lowercase();
		let query_chars = normalized_query.chars().collect_vec();
		if !(4..=10).contains(&query_chars.len()) {
			return Ok(query);
		}

		let candidate_query = json!({
			"bool": {
				"must": [{
					"match": {
						"name.prefix": {
							"query": token,
							"fuzziness": 1,
							"prefix_length": 0,
							"max_expansions": 1024,
							"fuzzy_transpositions": true
						}
					}
				}],
				"filter": filters
			}
		});
		let candidate_body = json!({
			"_source": ["project_id", "name"],
			"size": 100,
			"track_total_hits": false,
			"query": candidate_query,
			"sort": [
				{"log_downloads": {"order": "desc", "missing": "_last"}},
				{
					"version_published_timestamp": {
						"order": "desc",
						"missing": "_last"
					}
				},
				{"project_id": {"order": "asc"}}
			]
		});
		let candidates =
			self.execute_search(alias, &candidate_body, false).await?;
		let candidate_hits = candidates["hits"]["hits"]
			.as_array()
			.into_iter()
			.flatten()
			.collect_vec();
		let has_first_character_correction =
			candidate_hits.iter().any(|hit| {
				let Some(name) = hit["_source"]["name"].as_str() else {
					return false;
				};
				tokenize_search_name(name).into_iter().any(|token| {
					let token_chars = token.chars().collect_vec();
					token_chars.len() <= query_chars.len() + 1
						&& token_chars.first() != query_chars.first()
						&& damerau_levenshtein_distance(
							&query_chars,
							&token_chars,
						) == 1
				})
			});
		let promoted_project_ids = candidate_hits
			.into_iter()
			.filter_map(|hit| {
				let name = hit["_source"]["name"].as_str()?;
				let should_promote =
					tokenize_search_name(name).into_iter().any(|token| {
						let token_chars = token.chars().collect_vec();
						let distance = damerau_levenshtein_distance(
							&query_chars,
							&token_chars,
						);
						token_chars.len() <= query_chars.len() + 1
							&& ((query_chars.len() <= 7
								&& token_chars.len()
									== query_chars.len() + 1
								&& distance == 2)
								|| (has_first_character_correction
									&& distance == 1)
								|| (token_chars.len()
									== query_chars.len() + 1
									&& distance == 1))
					});
				should_promote
					.then(|| hit["_source"]["project_id"].as_str())
					.flatten()
					.map(ToOwned::to_owned)
			})
			.collect_vec();
		if promoted_project_ids.is_empty() {
			return Ok(query);
		}

		Ok(json!({
			"dis_max": {
				"queries": [
					query,
					{
						"constant_score": {
							"filter": {
								"bool": {
									"filter": [
										{
											"term": {
												"document_type": "project"
											}
										},
										{
											"terms": {
												"project_id":
													promoted_project_ids
											}
										}
									]
								}
							},
							"boost": 15.5
						}
					}
				],
				"tie_breaker": 0
			}
		}))
	}

}

impl Elasticsearch {
	pub(super) fn typesense_parity_index_schema() -> Value {
		json!({
			"settings": {
				"number_of_shards": 3,
				"number_of_replicas": 1,
				"refresh_interval": "30s",
				"index.mapping.total_fields.limit": 5000,
				"analysis": {
					"char_filter": {
						"hyphen_separator": {
							"type": "pattern_replace",
							"pattern": "-",
							"replacement": " "
						},
						"strip_symbols": {
							"type": "pattern_replace",
							"pattern": r"[^\p{L}\p{N}\s]",
							"replacement": ""
						}
					},
					"filter": {
						"typesense_stemmer": {
							"type": "stemmer",
							"language": "english"
						},
						"typesense_prefix_ngrams": {
							"type": "edge_ngram",
							"min_gram": 1,
							"max_gram": 10
						}
					},
					"analyzer": {
						"typesense_text": {
							"type": "custom",
							"char_filter": ["strip_symbols"],
							"tokenizer": "whitespace",
							"filter": ["lowercase"]
						},
						"typesense_hyphen_text": {
							"type": "custom",
							"char_filter": [
								"hyphen_separator",
								"strip_symbols"
							],
							"tokenizer": "whitespace",
							"filter": ["lowercase"]
						},
						"typesense_stemmed_text": {
							"type": "custom",
							"char_filter": ["strip_symbols"],
							"tokenizer": "whitespace",
							"filter": ["lowercase", "typesense_stemmer"]
						},
						"typesense_prefix_text": {
							"type": "custom",
							"char_filter": [
								"hyphen_separator",
								"strip_symbols"
							],
							"tokenizer": "whitespace",
							"filter": [
								"lowercase",
								"typesense_prefix_ngrams"
							]
						},
						"typesense_plain_prefix_text": {
							"type": "custom",
							"char_filter": ["strip_symbols"],
							"tokenizer": "whitespace",
							"filter": [
								"lowercase",
								"typesense_prefix_ngrams"
							]
						},
						"typesense_stemmed_prefix_text": {
							"type": "custom",
							"char_filter": ["strip_symbols"],
							"tokenizer": "whitespace",
							"filter": [
								"lowercase",
								"typesense_stemmer",
								"typesense_prefix_ngrams"
							]
						}
					}
				}
			},
			"mappings": {
				"dynamic_templates": [
					{
						"strings_as_keywords": {
							"match_mapping_type": "string",
							"mapping": {
								"type": "keyword",
								"ignore_above": 8191
							}
						}
					}
				],
				"properties": {
					"document_type": {
						"type": "join",
						"relations": {"project": "version"},
						"eager_global_ordinals": true
					},
					"version_id": {"type": "keyword"},
					"project_id": {"type": "keyword"},
					"project_types": {"type": "keyword"},
					"all_project_types": {"type": "keyword"},
					"slug": {
						"type": "text",
						"analyzer": "typesense_text",
						"index_options": "docs",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191},
							"prefix": {
								"type": "text",
								"analyzer": "typesense_plain_prefix_text",
								"search_analyzer": "typesense_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"author": {
						"type": "text",
						"analyzer": "typesense_hyphen_text",
						"index_options": "docs",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191},
							"prefix": {
								"type": "text",
								"analyzer": "typesense_prefix_text",
								"search_analyzer": "typesense_hyphen_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"indexed_author": {
						"type": "text",
						"analyzer": "typesense_text",
						"index_options": "docs",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fields": {
							"keyword": {
								"type": "keyword",
								"ignore_above": 8191
							},
							"prefix": {
								"type": "text",
								"analyzer": "typesense_plain_prefix_text",
								"search_analyzer": "typesense_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"name": {
						"type": "text",
						"analyzer": "typesense_hyphen_text",
						"index_options": "positions",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191},
							"prefix": {
								"type": "text",
								"analyzer": "typesense_prefix_text",
								"search_analyzer": "typesense_hyphen_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"indexed_name": {
						"type": "text",
						"analyzer": "typesense_stemmed_text",
						"index_options": "docs",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fielddata": true,
						"fields": {
							"prefix": {
								"type": "text",
								"analyzer": "typesense_stemmed_prefix_text",
								"search_analyzer": "typesense_stemmed_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"summary": {
						"type": "text",
						"analyzer": "typesense_text",
						"index_options": "docs",
						"norms": false,
						"index_prefixes": {
							"min_chars": 1,
							"max_chars": 10
						},
						"fields": {
							"keyword": {"type": "keyword", "ignore_above": 8191},
							"prefix": {
								"type": "text",
								"analyzer": "typesense_plain_prefix_text",
								"search_analyzer": "typesense_text",
								"index_options": "docs",
								"norms": false
							}
						}
					},
					"_search_tokens": {
						"properties": {
							"batch_score": {"type": "double"},
							"name": {"type": "keyword"},
							"indexed_name": {"type": "keyword"},
							"slug": {"type": "keyword"},
							"author": {"type": "keyword"},
							"indexed_author": {"type": "keyword"},
							"summary": {"type": "keyword"}
						}
					},
					"categories": {"type": "keyword"},
					"project_categories": {"type": "keyword"},
					"display_categories": {"type": "keyword"},
					"license": {"type": "keyword"},
					"open_source": {"type": "boolean"},
					"environment": {"type": "keyword"},
					"game_versions": {"type": "keyword"},
					"client_side": {"type": "keyword"},
					"server_side": {"type": "keyword"},
					"dependency_project_ids": {"type": "keyword"},
					"compatible_dependency_project_ids": {"type": "keyword"},
					"downloads": {"type": "integer"},
					"log_downloads": {"type": "double"},
					"follows": {"type": "integer"},
					"created_timestamp": {"type": "long"},
					"modified_timestamp": {"type": "long"},
					"version_published_timestamp": {"type": "long"},
					"date_created": {"type": "date"},
					"date_modified": {"type": "date"},
					"project_loader_fields": {"type": "object", "enabled": false},
					"minecraft_java_server": {
						"properties": {
							"verified_plays_2w": {"type": "long"},
							"is_online": {"type": "boolean"},
							"ping": {
								"properties": {
									"data": {
										"properties": {
											"players_online": {"type": "integer"}
										}
									}
								}
							}
						}
					}
				}
			}
		})
	}

}

impl Elasticsearch {
	pub(super) async fn import_projects_typesense_parity(
		&self,
		indices: &[String],
		documents: &[UploadSearchProject],
	) -> Result<()> {
		let batch_size = self.config.bulk_batch_size.max(1);
		let import_batch_size =
			self.config.project_import_batch_size.max(1);
		for import_batch in documents.chunks(import_batch_size) {
			for candidate_batch in
				import_batch
					.chunks(INTERNAL_INDEX_BATCH_SIZE)
			{
				let batch_score = candidate_batch
					.iter()
					.map(|document| document.log_downloads)
					.max_by(f64::total_cmp)
					.unwrap_or_default();
				for documents in candidate_batch.chunks(batch_size) {
					let body = projects_to_bulk(documents, batch_score)?;
					for index in indices {
						info!(
							index,
							document_count = documents.len(),
							content_length_bytes = body.len(),
							"sending Elasticsearch project bulk request"
						);
						self.client.bulk(index, body.clone()).await?;
					}
				}
			}
		}
		Ok(())
	}

}

fn projects_to_bulk(
	documents: &[UploadSearchProject],
	batch_score: f64,
) -> Result<String> {
	let mut output = String::new();
	for document in documents {
		let id = format!("project:{}", document.project_id);
		push_json_line(
			&mut output,
			&json!({
				"index": {
					"_id": id,
					"routing": document.project_id
				}
			}),
		)?;

		let mut source = serde_json::to_value(document)
			.wrap_err("failed to serialize `UploadSearchProject`")?;
		let object = source
			.as_object_mut()
			.ok_or_else(|| eyre!("project search document is not an object"))?;
		object.insert(
			"document_type".to_string(),
			Value::String("project".to_string()),
		);
		object.insert(
			"_search_tokens".to_string(),
			json!({
				"name": tokenize_candidate_text(&document.name, true),
				"indexed_name": &document.indexed_name,
				"slug": document
					.slug
					.as_deref()
					.map(|slug| tokenize_candidate_text(slug, false))
					.unwrap_or_default(),
				"author": tokenize_candidate_text(&document.author, true),
				"indexed_author": tokenize_candidate_text(
					&document.indexed_author,
					false,
				),
				"summary": tokenize_candidate_text(
					&document.summary,
					false,
				),
				"batch_score": batch_score,
			}),
		);
		add_server_online_field(object);
		push_json_line(&mut output, &source)?;
	}
	Ok(output)
}
