# Agent notes (Owyx Launcher)

## Cursor Cloud / GitHub

- **Canonical repo:** [`ebluffy/Owyx`](https://github.com/ebluffy/Owyx) (renamed from `KORESHon/Owyx`; old URLs redirect).
- Prefer remotes and `gh` calls against **`ebluffy/Owyx`**, not the old owner name.
- After the rename, the **Cursor GitHub App / cloud integration** may lose `createPullRequest` (and AutoReview) until it is **re-installed or re-authorized** on the `ebluffy` account/org. Local `gh` with a user token can still open PRs.
- Do not commit `*.exe`, `target/`, `node_modules/`, or a local `modrinth-code/` clone (gitignored reference only).

## Build gates

See [`BUILD.md`](./BUILD.md): `npm run check` → `check:rust` → `build:smoke` (debug) → `build:release` (ship only).
