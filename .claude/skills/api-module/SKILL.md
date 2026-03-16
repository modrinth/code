---
name: api-module
description: Add a new API endpoint module to packages/api-client from an OpenAPI schema. Use when adding new backend endpoints, creating API client modules, or when an openapi.yml is provided.
argument-hint: <path-to-openapi.yml>
---

Refer to the standard: @standards/frontend/ADDING_API_MODULES.md

## Steps

1. **Read the OpenAPI schema** at `$ARGUMENTS` — identify the endpoints, request/response shapes, and path parameters.
2. **Read the standard above** for naming conventions, type rules, and the module registration pattern.
3. **Determine the service and version** — the URL path prefix tells you which service directory and version namespace to use (e.g. `/v3/projects` → `labrinth/v3/`).
4. **Define types in `types.ts`** — types must match the API response 1:1. Use the OpenAPI schema as the source of truth. Do not reshape or rename fields.
5. **Create the module class** — extend `BaseModule`, implement each endpoint as a method. Use the correct HTTP verb and request options pattern from the standard.
6. **Register in `MODULE_REGISTRY`** — add the module entry so it's auto-instantiated on the client.
7. **Export types** from the service's barrel `index.ts`.
8. **Verify** — check that the module compiles and the types are accessible from `@modrinth/api-client`.
