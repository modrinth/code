#!/usr/bin/env python3
"""Split legacy Typesense JSONL into project and version documents."""

import argparse
import json
import os
import shutil
import sys
import tempfile
import time
import zlib
from collections import OrderedDict


VERSION_FILTER_PATHS = (
	"project_types",
	"environment",
	"game_versions",
	"client_side",
	"server_side",
)

BASE62_DIGITS = {
	character: index
	for index, character in enumerate(
		"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
	)
}


class ConversionError(Exception):
	pass


def parse_args():
	parser = argparse.ArgumentParser(
		description=(
			"Convert a legacy Typesense JSONL export from one document per "
			"version into separate project and version collections."
		)
	)
	parser.add_argument("input", help="Legacy Typesense JSONL export")
	parser.add_argument("projects_output", help="Destination project-document JSONL")
	parser.add_argument("versions_output", help="Destination version-document JSONL")
	parser.add_argument(
		"--shards",
		type=int,
		default=512,
		help="Temporary hash shards used to bound memory usage (default: 512)",
	)
	parser.add_argument(
		"--max-open-shards",
		type=int,
		default=64,
		help="Maximum temporary shard files held open at once (default: 64)",
	)
	parser.add_argument(
		"--progress-interval",
		type=float,
		default=10.0,
		help="Seconds between progress reports (default: 10)",
	)
	parser.add_argument(
		"--keep-temporary",
		action="store_true",
		help="Keep temporary shard files after completion or failure",
	)
	return parser.parse_args()


def base62_value(value):
	result = 0
	try:
		for character in value:
			result = result * 62 + BASE62_DIGITS[character]
	except KeyError:
		return -1
	return result


def get_path(document, path):
	value = document
	for segment in path.split("."):
		if not isinstance(value, dict) or segment not in value:
			return None, False
		value = value[segment]
	return value, True


def set_path(document, path, value):
	segments = path.split(".")
	current = document
	for segment in segments[:-1]:
		child = current.get(segment)
		if not isinstance(child, dict):
			child = {}
			current[segment] = child
		current = child
	current[segments[-1]] = value


def unique_sorted(values):
	return sorted(set(values))


def extend_values(target, value):
	if isinstance(value, list):
		target.extend(value)
	elif value is not None:
		target.append(value)


def version_filter_document(document):
	version_id = document.get("version_id") or document.get("id")
	result = {
		"id": str(version_id),
		"version_id": str(version_id),
		"project_id": str(document["project_id"]),
		"version_published_timestamp": document.get(
			"version_published_timestamp", -1
		),
	}
	loaders = list(document.get("loaders") or [])
	mrpack_loaders = list(document.get("mrpack_loaders") or [])
	loaders.extend(mrpack_loaders)
	if mrpack_loaders:
		loaders = [loader for loader in loaders if loader != "mrpack"]
	result["categories"] = unique_sorted(loaders)
	for path in VERSION_FILTER_PATHS:
		value, exists = get_path(document, path)
		if exists and value is not None:
			set_path(result, path, value)
	return result


class ProjectAccumulator:
	def __init__(self, document):
		self.latest_document = document
		self.latest_key = self.version_key(document)
		self.versions = {}
		self.categories = []
		self.version_categories = []
		self.loaders = []
		self.project_types = []
		self.client_side = []
		self.server_side = []
		self.add(document)

	@staticmethod
	def version_key(document):
		return (
			document.get("version_published_timestamp", -1),
			base62_value(str(document.get("version_id", ""))),
		)

	def add(self, document):
		project_id = document.get("project_id")
		if project_id != self.latest_document.get("project_id"):
			raise ConversionError("attempted to combine different projects")

		version_id = document.get("version_id") or document.get("id")
		if not version_id:
			raise ConversionError(f"project `{project_id}` has a version without an ID")

		key = self.version_key(document)
		if key > self.latest_key:
			self.latest_document = document
			self.latest_key = key

		version_document = version_filter_document(document)
		self.versions[str(version_id)] = (key, version_document)
		extend_values(self.categories, document.get("categories"))
		extend_values(self.version_categories, version_document.get("categories"))
		extend_values(self.loaders, document.get("loaders"))
		extend_values(self.project_types, document.get("project_types"))
		extend_values(self.project_types, document.get("all_project_types"))
		extend_values(self.client_side, document.get("client_side"))
		extend_values(self.server_side, document.get("server_side"))

	def finish(self):
		result = dict(self.latest_document)
		project_id = str(result["project_id"])
		all_project_types = unique_sorted(self.project_types)

		result["id"] = project_id
		result["version_id"] = str(
			self.latest_document.get("version_id")
			or self.latest_document.get("id")
		)
		result["categories"] = unique_sorted(self.categories)
		result["project_categories"] = unique_sorted(
			set(self.categories) - set(self.version_categories)
		)
		result["loaders"] = unique_sorted(self.loaders)
		result["project_types"] = all_project_types
		result["all_project_types"] = all_project_types

		project_loader_fields = result.get("project_loader_fields")
		if not isinstance(project_loader_fields, dict):
			project_loader_fields = {}
			result["project_loader_fields"] = project_loader_fields
		for field, value in project_loader_fields.items():
			result[field] = value

		if self.client_side:
			result["client_side"] = unique_sorted(self.client_side)
		if self.server_side:
			result["server_side"] = unique_sorted(self.server_side)

		versions = [
			version
			for _, version in sorted(
				self.versions.values(), key=lambda item: item[0]
			)
		]
		result.pop("versions", None)
		return result, versions


class ShardWriter:
	def __init__(self, directory, shard_count, max_open):
		self.directory = directory
		self.shard_count = shard_count
		self.max_open = max_open
		self.handles = OrderedDict()

	def path(self, shard):
		return os.path.join(self.directory, f"shard-{shard:04d}.jsonl")

	def write(self, project_id, line):
		shard = zlib.crc32(project_id.encode("utf-8")) % self.shard_count
		handle = self.handles.pop(shard, None)
		if handle is None:
			if len(self.handles) >= self.max_open:
				_, oldest = self.handles.popitem(last=False)
				oldest.close()
			handle = open(self.path(shard), "ab")
		self.handles[shard] = handle
		handle.write(line)

	def close(self):
		for handle in self.handles.values():
			handle.close()
		self.handles.clear()


def shard_input(args, temporary_directory):
	writer = ShardWriter(
		temporary_directory,
		args.shards,
		args.max_open_shards,
	)
	input_size = os.path.getsize(args.input)
	bytes_read = 0
	document_count = 0
	started_at = time.monotonic()
	last_report = started_at

	try:
		with open(args.input, "rb") as input_file:
			for line_number, line in enumerate(input_file, start=1):
				bytes_read += len(line)
				if not line.strip():
					continue
				try:
					document = json.loads(line)
				except json.JSONDecodeError as error:
					raise ConversionError(
						f"invalid JSON on input line {line_number}: {error}"
					) from error
				project_id = document.get("project_id")
				if not project_id:
					raise ConversionError(
						f"input line {line_number} has no `project_id`"
					)
				writer.write(str(project_id), line)
				document_count += 1

				now = time.monotonic()
				if now - last_report >= args.progress_interval:
					percent = bytes_read / input_size * 100 if input_size else 100
					print(
						f"sharding: {percent:.1f}% ({document_count:,} documents)",
						flush=True,
					)
					last_report = now
	finally:
		writer.close()

	print(
		f"sharding complete: {document_count:,} version documents in "
		f"{time.monotonic() - started_at:.1f}s",
		flush=True,
	)
	return document_count


def convert_shards(
	args,
	temporary_directory,
	partial_projects_output,
	partial_versions_output,
):
	project_count = 0
	version_count = 0
	started_at = time.monotonic()
	last_report = started_at

	with (
		open(partial_projects_output, "w", encoding="utf-8") as projects_file,
		open(partial_versions_output, "w", encoding="utf-8") as versions_file,
	):
		for shard in range(args.shards):
			path = os.path.join(temporary_directory, f"shard-{shard:04d}.jsonl")
			if not os.path.exists(path):
				continue

			projects = {}
			with open(path, "rb") as shard_file:
				for line_number, line in enumerate(shard_file, start=1):
					try:
						document = json.loads(line)
					except json.JSONDecodeError as error:
						raise ConversionError(
							f"invalid JSON in shard {shard}, line {line_number}: {error}"
						) from error
					project_id = str(document["project_id"])
					if project_id in projects:
						projects[project_id].add(document)
					else:
						projects[project_id] = ProjectAccumulator(document)
					version_count += 1

			for project_id in sorted(projects):
				project, versions = projects[project_id].finish()
				json.dump(
					project,
					projects_file,
					separators=(",", ":"),
					ensure_ascii=False,
				)
				projects_file.write("\n")
				for version in versions:
					json.dump(
						version,
						versions_file,
						separators=(",", ":"),
						ensure_ascii=False,
					)
					versions_file.write("\n")
				project_count += 1

			os.remove(path)
			now = time.monotonic()
			if now - last_report >= args.progress_interval:
				print(
					f"converting: shard {shard + 1}/{args.shards}, "
					f"{project_count:,} projects",
					flush=True,
				)
				last_report = now

	print(
		f"conversion complete: {version_count:,} versions into "
		f"{project_count:,} projects in {time.monotonic() - started_at:.1f}s",
		flush=True,
	)
	return project_count, version_count


def main():
	args = parse_args()
	if args.shards <= 0:
		raise ConversionError("--shards must be greater than zero")
	if args.max_open_shards <= 0:
		raise ConversionError("--max-open-shards must be greater than zero")
	if args.progress_interval <= 0:
		raise ConversionError("--progress-interval must be greater than zero")
	if not os.path.isfile(args.input):
		raise ConversionError(f"input file does not exist: {args.input}")
	paths = {
		os.path.abspath(args.input),
		os.path.abspath(args.projects_output),
		os.path.abspath(args.versions_output),
	}
	if len(paths) != 3:
		raise ConversionError("input and output paths must all differ")

	projects_directory = os.path.dirname(os.path.abspath(args.projects_output))
	versions_directory = os.path.dirname(os.path.abspath(args.versions_output))
	os.makedirs(projects_directory, exist_ok=True)
	os.makedirs(versions_directory, exist_ok=True)
	temporary_directory = tempfile.mkdtemp(
		prefix="typesense-project-convert-",
		dir=projects_directory,
	)
	partial_projects_output = f"{args.projects_output}.partial"
	partial_versions_output = f"{args.versions_output}.partial"

	try:
		expected_versions = shard_input(args, temporary_directory)
		project_count, version_count = convert_shards(
			args,
			temporary_directory,
			partial_projects_output,
			partial_versions_output,
		)
		if version_count != expected_versions:
			raise ConversionError(
				f"sharded {expected_versions} versions but converted {version_count}"
			)
		os.replace(partial_projects_output, args.projects_output)
		os.replace(partial_versions_output, args.versions_output)
		print(
			f"wrote {project_count:,} project documents to "
			f"{args.projects_output} and {version_count:,} version documents to "
			f"{args.versions_output}",
			flush=True,
		)
	except Exception:
		for partial_output in (
			partial_projects_output,
			partial_versions_output,
		):
			if os.path.exists(partial_output):
				os.remove(partial_output)
		raise
	finally:
		if args.keep_temporary:
			print(f"temporary shards kept at {temporary_directory}", file=sys.stderr)
		else:
			shutil.rmtree(temporary_directory, ignore_errors=True)


if __name__ == "__main__":
	try:
		main()
	except KeyboardInterrupt:
		print("interrupted", file=sys.stderr)
		sys.exit(130)
	except (ConversionError, OSError) as error:
		print(error, file=sys.stderr)
		sys.exit(1)
