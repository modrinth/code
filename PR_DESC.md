We really suck at changelogs - we should be documenting everything we do - this will encourage it.

- Introduces a fragment-based changelog system using YAML files in `.github/changelog/`, with scripts to create and bake them into `packages/blog/changelog.ts`
- Moves changelog data from `@modrinth/utils` to `@modrinth/blog`
- Adds CI workflow to remind PR authors to include a changelog fragment, with automatic comment cleanup when one is added - it's always going to be optional though.
- Adds automatic GitHub release creation with changelog body to the Modrinth App release workflow - will document this in Notion.
- Fragment filenames are now descriptive: `<product>-<type>-<hash>.yml`

Workflow changes include:

- `pnpm changelog <product> "description"` — fast path to create a changelog fragment (type inferred from prefix e.g. "Fixed ...", "Added ...")
- `pnpm changelog` — interactive prompt to create changelog fragments
- `pnpm changelog:bake -- --product <name> [--version X.Y.Z]` — consumes fragments and inserts entries into `changelog.ts`
- `pnpm changelog:bake -- --product app --version X.Y.Z --extract` — outputs the baked body for a version (used by CI)
- `pnpm changelog:bake -- --product web` is the same as running it for both hosting + platform.

CI changes:

- **`changelog-check.yml`** — on PRs, posts a bot comment if no changelog fragment is present; minimizes it as outdated once one is added
- **`theseus-release.yml`** — now extracts the app changelog and creates a GitHub release with it

Also did some small improvements to the `changelog.vue` page
