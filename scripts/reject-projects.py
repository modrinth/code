#!/usr/bin/env python3
"""
Bulk post a private moderator message to project threads and reject projects.

CSV input must include a project_id column:

project_id,internal_project_id
11lK1mXq,3621769036554
14cWGNk6,3784105343430
"""

import argparse
import csv
import json
import os
import sys
import time
import urllib.error
import urllib.request

DEFAULT_MESSAGE = "TODO include message"


def api_request(method, url, token, payload=None):
    headers = {
        "Authorization": f"Bearer {token}",
        "Content-Type": "application/json",
        "User-Agent": "modrinth-bulk-reject-projects/1.0",
    }
    body = None
    if payload is not None:
        body = json.dumps(payload).encode("utf-8")

    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = resp.read().decode("utf-8")
            return json.loads(data) if data else None
    except urllib.error.HTTPError as err:
        error_body = err.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"{method} {url} failed with {err.code}: {error_body}"
        ) from err


def load_project_ids(csv_path):
    with open(csv_path, newline="") as file:
        reader = csv.DictReader(file)
        if "project_id" not in (reader.fieldnames or []):
            raise RuntimeError("CSV must include a `project_id` column")

        for row in reader:
            project_id = row["project_id"].strip()
            if project_id:
                yield project_id


def reject_project(base_url, token, project_id, message):
    print(f"  GET project {project_id}")
    project = api_request(
        "GET",
        f"{base_url}/v3/project/{project_id}",
        token,
    )
    thread_id = project.get("thread_id")
    if not thread_id:
        raise RuntimeError(f"project `{project_id}` has no `thread_id`")

    print(
        f"  POST public moderator message to thread {thread_id} "
        "(Labrinth hides moderator identity server-side)"
    )
    api_request(
        "POST",
        f"{base_url}/v3/thread/{thread_id}",
        token,
        {
            "body": {
                "type": "text",
                "body": message,
                "private": False,
                "replying_to": None,
                "associated_images": [],
            },
        },
    )

    print("  PATCH project status to rejected")
    api_request(
        "PATCH",
        f"{base_url}/v3/project/{project_id}",
        token,
        {"status": "rejected"},
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("csv_path", nargs="?")
    parser.add_argument("--base-url", default="https://api.modrinth.com")
    parser.add_argument(
        "--token",
        default=os.environ.get("MODRINTH_TOKEN"),
        help="moderator PAT, defaults to MODRINTH_TOKEN",
    )
    parser.add_argument("--message", default=DEFAULT_MESSAGE)
    parser.add_argument(
        "--project-id",
        help="process only this project ID instead of reading the CSV",
    )
    parser.add_argument("--sleep", type=float, default=0.25)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    if not args.token and not args.dry_run:
        parser.error("--token is required unless --dry-run is used")

    if args.project_id:
        project_ids = [args.project_id]
    elif args.csv_path:
        project_ids = list(load_project_ids(args.csv_path))
    else:
        parser.error("csv_path is required unless --project-id is provided")

    touched = 0
    failed = 0

    for project_id in project_ids:
        print(f"Processing {project_id}")
        if args.dry_run:
            print("  DRY RUN: would GET project")
            print(
                "  DRY RUN: would POST public moderator message "
                "with server-side hidden moderator identity"
            )
            print("  DRY RUN: would PATCH project status to rejected")
            touched += 1
            continue

        try:
            reject_project(
                args.base_url.rstrip("/"),
                args.token,
                project_id,
                args.message,
            )
        except Exception as err:
            print(f"  failed: {err}", file=sys.stderr)
            failed += 1
            continue

        touched += 1
        print("  rejected")
        time.sleep(args.sleep)

    print(
        f"Done. Attempted {len(project_ids)} project(s), touched {touched}, failed {failed}."
    )


if __name__ == "__main__":
    main()
