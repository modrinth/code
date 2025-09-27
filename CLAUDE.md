# Architecture

## Frontend

There are two similar frontends in the Modrinth monorepo, the website (apps/frontend) and the app frontend (apps/app-frontend).

Both use Tailwind v3, and their respective configs can be seen at `tailwind.config.ts` and `tailwind.config.js` respectively.

Both utilize shared and common components from `@modrinth/ui` which can be found at `packages/ui`, and stylings from `@modrinth/assets` which can be found at `packages/assets`.

Both can utilize icons from `@modrinth/assets`, which are automatically generated based on what's available within the `icons` folder of the `packages/assets` directory. You can see the generated icons list in `generated-icons.ts`.

Both have access to our dependency injection framework, examples as seen in `packages/ui/src/providers/`. Ideally any state which is shared between a page and it's subpages should be shared using this dependency injection framework.

### Website (apps/frontend)

Before a pull request can be opened for the website, `pnpm web:fix` and `pnpm web:intl:extract` must be run, otherwise CI will fail.

To run a development version of the frontend, you must first copy over the relevant `.env` template file (prod, staging or local, usually prod) within the `apps/frontend` folder into `apps/frontend/.env`. Then you can run the frontend by running `pnpm web:dev` in the root folder.

### App Frontend (apps/app-frontend)

Before a pull request can be opened for the website, you must CD into the `app-frontend` folder; `pnpm fix` and `pnpm intl:extract` must be run, otherwise CI will fail.

To run a development version of the app frontend, you must first copy over the relevant `.env` template file (prod, staging or local, usually prod) within `packages/app-lib` into `packages/app-lib/.env`. Then you must run the app itself by running `pnpm app:dev` in the root folder.

### Localization

Refer to `.github/instructions/i18n-convert.instructions.md` if the user asks you to perform any i18n conversion work on a component, set of components, pages or sets of pages.

## Labrinth

Labrinth is the backend API service for Modrinth.

### Testing

Before a pull request can be opened, run `cargo clippy -p labrinth --all-targets` and make sure there are ZERO warnings, otherwise CI will fail.

Use `cargo test -p labrinth --all-targets` to test your changes. All tests must pass, otherwise CI will fail.

Read the root `docker-compose.yml` to see what running services are available while developing. Use `docker exec` to access these services.

### Clickhouse

Use `docker exec labrinth-clickhouse clickhouse-client` to access the Clickhouse instance. We use the `staging_ariadne` database to store data in testing.

### Postgres

Use `docker exec labrinth-postgres psql -U postgres` to access the PostgreSQL instance.
