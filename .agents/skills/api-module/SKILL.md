---
name: api-module
description: Add an API endpoint module to packages/api-client from an OpenAPI schema. Use for new backend endpoints, API client modules, or tasks that provide an OpenAPI schema.
---

# Add an API Module

Read the applicable `AGENTS.md` files before you edit code.

Read [the API module standard](../../../standards/frontend/ADDING_API_MODULES.md) in full.

1. Identify the OpenAPI schema from the request. If more than one schema is possible, ask the user to select one.
2. Read the schema. Identify each endpoint, HTTP method, request type, response type, and path parameter.
3. Get the service and version from the URL prefix. For example, map `/v3/projects` to `labrinth/v3/`.
4. Define the API types in `types.ts`. Make each type match the schema exactly.
5. Do not change, rename, or remove API fields.
6. Make a module class that extends `AbstractModule`. Implement each endpoint with `this.client.request()` or `this.client.upload()`.
7. Use the request-option pattern from the standard. Do not call `$fetch`, `fetch`, or another HTTP client directly.
8. Add the module to `MODULE_REGISTRY` so the client can instantiate it.
9. Export new service types from the applicable barrel `index.ts`.
10. Check the module paths, registry key, public type exports, and endpoint types.

Run only the checks that the user or the applicable `AGENTS.md` permits.
