#!/usr/bin/env python3
"""
Query the local Labrinth Typesense collection directly.

Usage:
	python3 scripts/query-typesense.py sodium
	python3 scripts/query-typesense.py foobarqux --per-page 10
"""

import argparse
import json
import os
import urllib.error
import urllib.request


def request_json(base_url, api_key, path, body, timeout):
	request = urllib.request.Request(
		f"{base_url.rstrip('/')}{path}",
		data=json.dumps(body).encode("utf-8"),
		method="POST",
		headers={
			"X-TYPESENSE-API-KEY": api_key,
			"Content-Type": "application/json",
		},
	)

	try:
		with urllib.request.urlopen(request, timeout=timeout) as response:
			return json.load(response)
	except urllib.error.HTTPError as error:
		error_body = error.read().decode("utf-8", errors="replace")
		raise SystemExit(f"Typesense request failed ({error.code}): {error_body}") from error
	except urllib.error.URLError as error:
		raise SystemExit(f"Typesense request failed: {error.reason}") from error


def parse_args():
	parser = argparse.ArgumentParser(description="Query Typesense directly.")
	parser.add_argument("query", help="Search query")
	parser.add_argument(
		"--typesense-url",
		default=os.environ.get("TYPESENSE_URL", "http://localhost:8108"),
		help="Typesense URL",
	)
	parser.add_argument(
		"--api-key",
		default=os.environ.get("TYPESENSE_API_KEY", "modrinth"),
		help="Typesense API key",
	)
	parser.add_argument(
		"--collection",
		default="labrinth_projects__current",
		help="Typesense collection to search",
	)
	parser.add_argument("--per-page", type=int, default=5)
	parser.add_argument("--timeout", type=float, default=30.0)
	return parser.parse_args()


def main():
	args = parse_args()
	payload = {
		"searches": [
			{
				"collection": args.collection,
				"q": args.query,
				"query_by": "name,summary,author,indexed_name,indexed_author",
				"per_page": args.per_page,
				"group_by": "project_id",
				"group_limit": 1,
				"facet_by": "project_id",
				"max_facet_values": 0,
			}
		]
	}

	body = request_json(
		args.typesense_url, args.api_key, "/multi_search", payload, args.timeout
	)
	result = body["results"][0]

	print(f"query: {args.query}")
	print(f"collection: {args.collection}")
	print(f"found: {result.get('found', 0)}")
	print(f"out_of: {result.get('out_of', 0)}")
	print(f"search_time_ms: {result.get('search_time_ms', 0)}")
	print()

	grouped_hits = result.get("grouped_hits", [])
	if not grouped_hits:
		print("no hits")
		return

	for index, group in enumerate(grouped_hits, start=1):
		hit = group["hits"][0]
		document = hit["document"]
		print(f"{index}. {document.get('name', '')} ({document.get('slug', '')})")
		print(f"   project_id: {document.get('project_id', '')}")
		print(f"   author: {document.get('author', '')}")
		print(f"   summary: {document.get('summary', '')}")


if __name__ == "__main__":
	main()
