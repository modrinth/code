#!/usr/bin/env python3
"""
Search projects on api.modrinth.com and import results into the local database
with correct author names.

Modes:
  search  - Import top N results for a text query
  top     - Import the top N projects by total downloads (for building a
            representative corpus that mirrors prod IDF distributions)

Usage:
    python3 scripts/import-projects.py search <query> [limit]
    python3 scripts/import-projects.py top [count]

Examples:
    python3 scripts/import-projects.py search "sodium" 5
    python3 scripts/import-projects.py top 1000
"""

import json
import subprocess
import sys
import time
import urllib.parse
import urllib.request

ADMIN_USER_ID = 103587649610509
DB_CONTAINER = "labrinth-postgres"
DB_USER = "labrinth"
DB_NAME = "labrinth"
API_BASE = "https://api.modrinth.com/v2"
HEADERS = {"User-Agent": "import-projects-script/1.0"}

seen_slugs = set()
author_user_ids = {}
next_user_id = 200_000_000_000_000


def api_get(url):
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())


def psql(sql):
    result = subprocess.run(
        [
            "podman",
            "exec",
            DB_CONTAINER,
            "psql",
            "-U",
            DB_USER,
            "-d",
            DB_NAME,
            "-c",
            sql,
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"  DB error: {result.stderr.strip()}", file=sys.stderr)
        return False
    return True


def sql_escape(s):
    return s.replace("'", "''")


def get_or_create_author_user(author_name):
    global next_user_id
    if author_name in author_user_ids:
        return author_user_ids[author_name]
    uid = next_user_id
    next_user_id += 1
    name_e = sql_escape(author_name)
    sql = f"""
    INSERT INTO users (id, username, email, created, role)
    VALUES ({uid}, '{name_e}', '{name_e}@imported.local', NOW(), 'developer')
    ON CONFLICT (id) DO NOTHING;
    """
    if psql(sql):
        author_user_ids[author_name] = uid
    else:
        author_user_ids[author_name] = ADMIN_USER_ID
    return author_user_ids[author_name]


def import_project(hit, counter):
    slug = hit.get("slug", "")
    if slug in seen_slugs:
        return False
    seen_slugs.add(slug)

    title = hit.get("title", "")
    summary = hit.get("description", "")[:2048]
    project_id_api = hit.get("project_id", "")
    downloads = hit.get("downloads", 0)
    follows = hit.get("follows", 0)
    icon_url = hit.get("icon_url") or None
    author_name = hit.get("author", "Unknown")

    print(f"  Fetching: {title}")
    try:
        project_data = api_get(f"{API_BASE}/project/{project_id_api}")
        description = (project_data.get("body") or "")[:65536]
        icon_url = project_data.get("icon_url") or icon_url
    except Exception:
        description = summary

    author_id = get_or_create_author_user(author_name)

    base = int(time.time() * 1e9) % 900_000_000_000_000 + 100_000_000_000_000
    mod_id = base + counter * 5
    team_id = base + counter * 5 + 1
    member_id = base + counter * 5 + 2
    version_id = base + counter * 5 + 3

    title_e = sql_escape(title)
    summary_e = sql_escape(summary)
    description_e = sql_escape(description)
    slug_e = sql_escape(slug)
    icon_col = f"'{sql_escape(icon_url)}'" if icon_url else "NULL"

    print(
        f"  Importing: {title} (author={author_name}, downloads={downloads}, followers={follows})"
    )

    sql = f"""
BEGIN;

INSERT INTO teams (id) VALUES ({team_id});

INSERT INTO mods (
    id, team_id, name, summary, description,
    published, downloads, follows,
    status, license, side_types_migration_review_status,
    components, monetization_status, slug,
    icon_url, raw_icon_url
) VALUES (
    {mod_id},
    {team_id},
    '{title_e}',
    '{summary_e}',
    '{description_e}',
    NOW(),
    {downloads},
    {follows},
    'approved',
    'LicenseRef-All-Rights-Reserved',
    'reviewed',
    '{{}}'::jsonb,
    'monetized',
    LOWER('{slug_e}'),
    {icon_col},
    {icon_col}
);

INSERT INTO team_members (
    id, team_id, user_id, role, permissions,
    accepted, payouts_split, ordering, is_owner
) VALUES (
    {member_id},
    {team_id},
    {author_id},
    'Owner',
    1275068466,
    true,
    1.00000000000000000000,
    0,
    true
);

INSERT INTO versions (
    id, mod_id, name, version_number, version_type,
    author_id, downloads, changelog, status, components
) VALUES (
    {version_id},
    {mod_id},
    '1.0.0',
    '1.0.0',
    'release',
    {author_id},
    {downloads},
    '',
    'listed',
    '{{}}'::jsonb
);

INSERT INTO loaders_versions (loader_id, version_id) VALUES (2, {version_id});

COMMIT;
"""
    return psql(sql)


def mode_search(query, limit=5):
    encoded_query = urllib.parse.quote(query)
    search_url = f"{API_BASE}/search?query={encoded_query}&limit={limit}&facets=[]"
    print(f"Searching Modrinth for: {query} (limit: {limit})")

    search_data = api_get(search_url)
    hits = search_data.get("hits", [])

    if not hits:
        print("No results found.")
        return

    imported = 0
    for i, hit in enumerate(hits):
        if import_project(hit, i):
            imported += 1

    print(f"Done. Imported {imported} project(s).")


def mode_top(count=1000):
    print(f"Fetching top {count} projects by downloads from Modrinth...")

    imported = 0
    batch_size = 50
    counter = 0

    for offset in range(0, count, batch_size):
        limit = min(batch_size, count - offset)
        url = (
            f"{API_BASE}/search?limit={limit}&offset={offset}&index=downloads&facets=[]"
        )
        print(f"\n  Batch offset={offset}, limit={limit}")

        data = api_get(url)
        hits = data.get("hits", [])

        if not hits:
            break

        for hit in hits:
            if import_project(hit, counter):
                imported += 1
            counter += 1

        time.sleep(1)

    print(f"\nDone. Imported {imported} project(s).")


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} search <query> [limit]")
        print(f"       {sys.argv[0]} top [count]")
        sys.exit(1)

    mode = sys.argv[1]

    if mode == "search":
        if len(sys.argv) < 3:
            print("Usage: {sys.argv[0]} search <query> [limit]")
            sys.exit(1)
        query = sys.argv[2]
        limit = int(sys.argv[3]) if len(sys.argv) > 3 else 5
        mode_search(query, limit)
    elif mode == "top":
        count = int(sys.argv[2]) if len(sys.argv) > 2 else 1000
        mode_top(count)
    else:
        print(f"Unknown mode: {mode}. Use 'search' or 'top'.")
        sys.exit(1)


if __name__ == "__main__":
    main()
