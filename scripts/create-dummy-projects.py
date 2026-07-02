#!/usr/bin/env python3
import argparse
import json
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from uuid import uuid4


def make_body(boundary, slug, index):
	data = {
		"name": f"Dummy Load {index:04d}",
		"slug": slug,
		"summary": "A dummy project for local load testing.",
		"description": "This project was generated locally for batch indexing tests.",
		"initial_versions": [],
		"is_draft": True,
		"categories": [],
		"license_id": "MIT",
	}

	payload = json.dumps(data, separators=(",", ":"))
	return (
		f"--{boundary}\r\n"
		'Content-Disposition: form-data; name="data"\r\n'
		"Content-Type: application/json\r\n\r\n"
		f"{payload}\r\n"
		f"--{boundary}--\r\n"
	).encode()


def create_project(base_url, token, boundary, prefix, index, retries):
	slug = f"{prefix}-{index:04d}"
	body = make_body(boundary, slug, index)
	headers = {
		"Authorization": f"Bearer {token}",
		"Content-Type": f"multipart/form-data; boundary={boundary}",
	}

	for attempt in range(retries + 1):
		req = urllib.request.Request(
			f"{base_url}/v3/project",
			data=body,
			headers=headers,
			method="POST",
		)

		try:
			with urllib.request.urlopen(req, timeout=60) as resp:
				resp.read()
				return True, slug, resp.status, ""
		except urllib.error.HTTPError as err:
			text = err.read().decode("utf-8", errors="replace")
			if err.code < 500 or attempt == retries:
				return False, slug, err.code, text
		except Exception as err:
			if attempt == retries:
				return False, slug, "error", repr(err)

		time.sleep(min(2**attempt, 10))

	raise RuntimeError("unreachable")


def main():
	parser = argparse.ArgumentParser()
	parser.add_argument("--base-url", default="http://localhost:8000")
	parser.add_argument("--token", default="mra_admin")
	parser.add_argument("--count", type=int, default=1000)
	parser.add_argument("--concurrency", type=int, default=2)
	parser.add_argument("--retries", type=int, default=5)
	args = parser.parse_args()

	boundary = "----modrinth-dummy-project-boundary"
	prefix = f"dummy-load-{int(time.time())}-{uuid4().hex[:6]}"

	ok = 0
	failures = []

	with ThreadPoolExecutor(max_workers=args.concurrency) as executor:
		futures = [
			executor.submit(
				create_project,
				args.base_url,
				args.token,
				boundary,
				prefix,
				index,
				args.retries,
			)
			for index in range(args.count)
		]

		for completed, future in enumerate(as_completed(futures), 1):
			success, slug, status, text = future.result()
			if success:
				ok += 1
			else:
				failures.append((slug, status, text[:500]))

			if completed % 50 == 0:
				print(
					f"completed={completed} created={ok} failed={len(failures)}",
					flush=True,
				)

	print(
		json.dumps(
			{
				"prefix": prefix,
				"attempted": args.count,
				"created": ok,
				"failed": len(failures),
				"failures": failures[:20],
			},
			indent=2,
		)
	)


if __name__ == "__main__":
	main()
