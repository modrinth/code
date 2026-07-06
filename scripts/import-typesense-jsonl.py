#!/usr/bin/env python3
"""
Import a JSONL Typesense export into local Labrinth Typesense collections.

By default this imports tmp/export.jsonl into the Labrinth project search alias,
resolving the alias to its current backing collection first.

Usage:
	python3 scripts/import-typesense-jsonl.py
	python3 scripts/import-typesense-jsonl.py tmp/export.jsonl --continue
	python3 scripts/import-typesense-jsonl.py tmp/export.jsonl --alias labrinth_projects
	python3 scripts/import-typesense-jsonl.py --collection labrinth_projects__alt
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.parse
import urllib.request


class TypesenseError(Exception):
	pass


def default_aliases():
	prefix = os.environ.get("TYPESENSE_INDEX_PREFIX", "labrinth")
	return [f"{prefix}_projects"]


def request_typesense(base_url, api_key, method, path, timeout, body=None, content_type=None):
	url = f"{base_url.rstrip('/')}{path}"
	request = urllib.request.Request(url, data=body, method=method)
	request.add_header("X-TYPESENSE-API-KEY", api_key)
	if content_type:
		request.add_header("Content-Type", content_type)

	try:
		with urllib.request.urlopen(request, timeout=timeout) as response:
			return response.read()
	except urllib.error.HTTPError as error:
		error_body = error.read().decode("utf-8", errors="replace")
		raise TypesenseError(f"{method} {path} failed ({error.code}): {error_body}") from error
	except urllib.error.URLError as error:
		raise TypesenseError(f"{method} {path} failed: {error.reason}") from error


def get_json_or_none(base_url, api_key, path, timeout):
	try:
		body = request_typesense(base_url, api_key, "GET", path, timeout)
	except TypesenseError as error:
		if " failed (404):" in str(error):
			return None
		raise
	return json.loads(body.decode("utf-8"))


def request_json(base_url, api_key, method, path, timeout, body=None):
	encoded_body = None
	if body is not None:
		encoded_body = json.dumps(body).encode("utf-8")

	response = request_typesense(
		base_url,
		api_key,
		method,
		path,
		timeout,
		body=encoded_body,
		content_type="application/json" if body is not None else None,
	)
	if not response:
		return None
	return json.loads(response.decode("utf-8"))


def resolve_alias_or_collection(base_url, api_key, name, timeout):
	escaped = urllib.parse.quote(name, safe="")
	alias = get_json_or_none(base_url, api_key, f"/aliases/{escaped}", timeout)
	if alias is not None:
		collection_name = alias.get("collection_name")
		if not collection_name:
			raise TypesenseError(f"Alias {name} did not include collection_name")
		return collection_name

	collection = get_json_or_none(base_url, api_key, f"/collections/{escaped}", timeout)
	if collection is None:
		raise TypesenseError(f"No Typesense alias or collection exists for {name}")
	return name


def collection_document_count(base_url, api_key, collection, timeout):
	escaped = urllib.parse.quote(collection, safe="")
	body = get_json_or_none(base_url, api_key, f"/collections/{escaped}", timeout)
	if body is None:
		raise TypesenseError(f"No Typesense collection exists for {collection}")
	return int(body.get("num_documents", 0))


def collection_schema_exists(base_url, api_key, collection, timeout):
	escaped = urllib.parse.quote(collection, safe="")
	return get_json_or_none(base_url, api_key, f"/collections/{escaped}", timeout) is not None


def collection_schema_for_create(name, source):
	schema = {
		"name": name,
		"fields": source["fields"],
		"default_sorting_field": source.get("default_sorting_field"),
		"enable_nested_fields": source.get("enable_nested_fields", False),
		"token_separators": source.get("token_separators", []),
		"symbols_to_index": source.get("symbols_to_index", []),
	}
	return {key: value for key, value in schema.items() if value is not None}


def sibling_collection_name(collection):
	if collection.endswith("__current"):
		return f"{collection[:-len('__current')]}__alt"
	if collection.endswith("__alt"):
		return f"{collection[:-len('__alt')]}__current"
	return None


def create_collection_from_source(base_url, api_key, collection, source_collection, timeout):
	escaped_source = urllib.parse.quote(source_collection, safe="")
	source = get_json_or_none(base_url, api_key, f"/collections/{escaped_source}", timeout)
	if source is None:
		raise TypesenseError(
			f"Cannot create {collection}: source collection {source_collection} does not exist"
		)

	request_json(
		base_url,
		api_key,
		"POST",
		"/collections",
		timeout,
		body=collection_schema_for_create(collection, source),
	)
	print(f"Created {collection} from schema of {source_collection}")


def ensure_collection_exists(collection, args):
	if collection_schema_exists(args.typesense_url, args.api_key, collection, args.timeout):
		return

	if not args.create_missing_collection:
		raise TypesenseError(f"No Typesense collection exists for {collection}")

	source_collection = args.create_from_collection or sibling_collection_name(collection)
	if not source_collection:
		raise TypesenseError(
			f"No Typesense collection exists for {collection}; pass --create-from-collection"
		)

	create_collection_from_source(
		args.typesense_url,
		args.api_key,
		collection,
		source_collection,
		args.timeout,
	)


def typesense_health_ok(base_url, api_key, timeout):
	try:
		body = request_typesense(base_url, api_key, "GET", "/health", timeout)
	except TypesenseError as error:
		if " failed (503):" in str(error):
			return False
		raise

	return json.loads(body.decode("utf-8")).get("ok") is True


def wait_for_typesense_drain(collection, expected_documents, args):
	started_at = time.monotonic()
	last_reported_at = 0.0

	while True:
		document_count = collection_document_count(
			args.typesense_url, args.api_key, collection, args.timeout
		)
		healthy = typesense_health_ok(args.typesense_url, args.api_key, args.timeout)

		if document_count >= expected_documents and healthy:
			elapsed = time.monotonic() - started_at
			print(
				f"{collection}: Typesense reports {document_count} document(s) "
				f"and healthy after flush wait ({elapsed:.1f}s)"
			)
			return elapsed

		elapsed = time.monotonic() - started_at
		if elapsed > args.flush_timeout:
			raise TypesenseError(
				f"Timed out waiting for Typesense to flush {collection}: "
				f"{document_count}/{expected_documents} document(s), "
				f"health ok={healthy}"
			)

		if elapsed - last_reported_at >= args.flush_report_interval:
			print(
				f"{collection}: waiting for Typesense flush; "
				f"{document_count}/{expected_documents} document(s), "
				f"health ok={healthy} ({elapsed:.1f}s)"
			)
			last_reported_at = elapsed

		time.sleep(args.flush_poll_interval)


def unique(values):
	result = []
	seen = set()
	for value in values:
		if value in seen:
			continue
		seen.add(value)
		result.append(value)
	return result


def iter_jsonl_batches(path, batch_lines, max_batch_bytes, skip_documents=0):
	batch = []
	batch_bytes = 0
	start_line = None
	end_line = None
	skipped = 0

	with open(path, "rb") as export_file:
		for line_number, line in enumerate(export_file, start=1):
			if not line.strip():
				continue

			if skipped < skip_documents:
				skipped += 1
				continue

			line_bytes = len(line)
			should_flush = batch and (
				len(batch) >= batch_lines or batch_bytes + line_bytes > max_batch_bytes
			)
			if should_flush:
				yield start_line, end_line, len(batch), b"".join(batch)
				batch = []
				batch_bytes = 0
				start_line = None

			if start_line is None:
				start_line = line_number
			end_line = line_number
			batch.append(line)
			batch_bytes += line_bytes

	if batch:
		yield start_line, end_line, len(batch), b"".join(batch)


def count_jsonl_batches(path, batch_lines, max_batch_bytes, skip_documents=0):
	batch_count = 0
	batch_lines_count = 0
	batch_bytes = 0
	skipped = 0

	with open(path, "rb") as export_file:
		for line in export_file:
			if not line.strip():
				continue

			if skipped < skip_documents:
				skipped += 1
				continue

			line_bytes = len(line)
			if batch_lines_count and (
				batch_lines_count >= batch_lines or batch_bytes + line_bytes > max_batch_bytes
			):
				batch_count += 1
				batch_lines_count = 0
				batch_bytes = 0

			batch_lines_count += 1
			batch_bytes += line_bytes

	if batch_lines_count:
		batch_count += 1

	return batch_count


def parse_import_response(body, failures_to_print):
	imported = 0
	failed = 0
	failures = []
	text = body.decode("utf-8", errors="replace")

	for line in text.splitlines():
		if not line:
			continue
		try:
			result = json.loads(line)
		except json.JSONDecodeError:
			failed += 1
			if len(failures) < failures_to_print:
				failures.append(line)
			continue

		if result.get("success") is True:
			imported += 1
		else:
			failed += 1
			if len(failures) < failures_to_print:
				failures.append(json.dumps(result, sort_keys=True))

	if imported == 0 and failed == 0 and text.strip():
		failed = 1
		failures.append(text.strip())

	return imported, failed, failures


def import_batch(base_url, api_key, collection, args, body):
	query = urllib.parse.urlencode(
		{
			"action": args.action,
			"dirty_values": args.dirty_values,
		}
	)
	escaped = urllib.parse.quote(collection, safe="")
	if not body.endswith(b"\n"):
		body += b"\n"

	return request_typesense(
		base_url,
		api_key,
		"POST",
		f"/collections/{escaped}/documents/import?{query}",
		args.timeout,
		body=body,
		content_type="text/plain",
	)


def import_file(collection, args):
	imported_total = 0
	batch_count = 0
	skip_documents = 0
	next_flush_at = args.flush_interval
	started_at = time.monotonic()
	flush_batch_started_at = started_at
	flush_batch_start_total = 0

	ensure_collection_exists(collection, args)

	if args.continue_import:
		skip_documents = collection_document_count(
			args.typesense_url, args.api_key, collection, args.timeout
		)
		print(f"{collection}: continuing after {skip_documents} existing document(s)")

	total_batches = count_jsonl_batches(
		args.file, args.batch_lines, args.max_batch_bytes, skip_documents
	)
	if total_batches == 0:
		print(f"{collection}: no remaining documents to import")
		return 0

	for start_line, end_line, document_count, body in iter_jsonl_batches(
		args.file, args.batch_lines, args.max_batch_bytes, skip_documents
	):
		batch_count += 1
		batch_started_at = time.monotonic()
		response = import_batch(args.typesense_url, args.api_key, collection, args, body)
		batch_elapsed = time.monotonic() - batch_started_at
		imported, failed, failures = parse_import_response(response, args.failures_to_print)

		if failed:
			print(
				f"Typesense reported {failed} failed document(s) for "
				f"{collection}, input lines {start_line}-{end_line}.",
				file=sys.stderr,
			)
			for failure in failures:
				print(failure, file=sys.stderr)
			raise TypesenseError(f"Import into {collection} failed")

		if imported != document_count:
			raise TypesenseError(
				f"Typesense acknowledged {imported} document(s) for "
				f"{collection}, but the batch contained {document_count}"
			)

		imported_total += imported
		elapsed = time.monotonic() - started_at
		percent = batch_count / total_batches * 100
		print(
			f"{collection}: batch {batch_count}/{total_batches} ({percent:.1f}%) "
			f"imported {imported} document(s) "
			f"from lines {start_line}-{end_line}; {imported_total} total "
			f"(batch {batch_elapsed:.1f}s, elapsed {elapsed:.1f}s)"
		)

		if args.flush_interval and imported_total >= next_flush_at:
			expected_documents = skip_documents + imported_total
			flush_batch_elapsed = time.monotonic() - flush_batch_started_at
			flush_batch_documents = imported_total - flush_batch_start_total
			print(
				f"{collection}: flush checkpoint at {imported_total} imported document(s); "
				f"imported {flush_batch_documents} document(s) since previous flush "
				f"in {flush_batch_elapsed:.1f}s"
			)
			flush_wait_elapsed = wait_for_typesense_drain(collection, expected_documents, args)
			print(
				f"{collection}: flush checkpoint complete; "
				f"import batch {flush_batch_elapsed:.1f}s, "
				f"flush wait {flush_wait_elapsed:.1f}s"
			)
			while imported_total >= next_flush_at:
				next_flush_at += args.flush_interval
			flush_batch_started_at = time.monotonic()
			flush_batch_start_total = imported_total

	elapsed = time.monotonic() - started_at
	print(f"{collection}: imported {imported_total} documents in {batch_count} batches ({elapsed:.1f}s)")
	return imported_total


def parse_args():
	parser = argparse.ArgumentParser(
		description="Import a JSONL export into Typesense in payload-safe batches."
	)
	parser.add_argument("jsonl_file", nargs="?", help="JSONL export to import")
	parser.add_argument("--file", help="JSONL export to import")
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
		"--alias",
		action="append",
		dest="aliases",
		help="Typesense alias or collection name to import into. Repeat for multiple targets.",
	)
	parser.add_argument(
		"--collection",
		action="append",
		dest="collections",
		help="Exact Typesense collection name. Repeat for multiple targets.",
	)
	parser.add_argument("--action", default="create", choices=["create", "update", "upsert", "emplace"])
	parser.add_argument("--dirty-values", default="coerce_or_drop")
	parser.add_argument("--batch-lines", type=int, default=5_000)
	parser.add_argument("--max-batch-bytes", type=int, default=4 * 1024 * 1024)
	parser.add_argument("--flush-interval", type=int, default=10_000)
	parser.add_argument("--flush-timeout", type=float, default=900.0)
	parser.add_argument("--flush-poll-interval", type=float, default=2.0)
	parser.add_argument("--flush-report-interval", type=float, default=10.0)
	parser.add_argument(
		"--create-missing-collection",
		action=argparse.BooleanOptionalAction,
		default=True,
		help="Create a missing __current/__alt collection by cloning its sibling schema.",
	)
	parser.add_argument(
		"--create-from-collection",
		help="Collection whose schema should be cloned when creating a missing target collection.",
	)
	parser.add_argument("--failures-to-print", type=int, default=20)
	parser.add_argument("--timeout", type=float, default=120.0)
	parser.add_argument(
		"--continue",
		action="store_true",
		dest="continue_import",
		help="Resume by skipping the number of documents already in the target collection.",
	)
	return parser.parse_args()


def main():
	args = parse_args()

	if args.batch_lines <= 0:
		raise TypesenseError("--batch-lines must be greater than zero")
	if args.max_batch_bytes <= 0:
		raise TypesenseError("--max-batch-bytes must be greater than zero")
	if args.flush_interval < 0:
		raise TypesenseError("--flush-interval must be greater than or equal to zero")
	if args.flush_timeout <= 0:
		raise TypesenseError("--flush-timeout must be greater than zero")
	if args.flush_poll_interval <= 0:
		raise TypesenseError("--flush-poll-interval must be greater than zero")
	if args.flush_report_interval <= 0:
		raise TypesenseError("--flush-report-interval must be greater than zero")
	if args.aliases and args.collections:
		raise TypesenseError("Use either --alias or --collection, not both")
	if args.jsonl_file and args.file:
		raise TypesenseError("Pass the JSONL path either positionally or with --file, not both")
	args.file = args.jsonl_file or args.file or "tmp/export.jsonl"
	if not os.path.isfile(args.file):
		raise TypesenseError(f"File does not exist: {args.file}")

	if args.collections:
		collections = unique(args.collections)
	else:
		aliases = unique(args.aliases or default_aliases())
		collections = [
			resolve_alias_or_collection(args.typesense_url, args.api_key, alias, args.timeout)
			for alias in aliases
		]
		collections = unique(collections)

	print(f"Importing {args.file} into {', '.join(collections)}")
	for collection in collections:
		import_file(collection, args)


if __name__ == "__main__":
	try:
		main()
	except KeyboardInterrupt:
		print("Interrupted.", file=sys.stderr)
		sys.exit(130)
	except TypesenseError as error:
		print(error, file=sys.stderr)
		sys.exit(1)
