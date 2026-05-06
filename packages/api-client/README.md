# @modrinth/api-client

[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-c78aff?style=for-the-badge)](https://www.typescriptlang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-c78aff?style=for-the-badge)](LICENSE)

Platform-agnostic TypeScript client for Modrinth services. It supports Labrinth, Archon, and Kyros across Node.js, browsers, Nuxt, and Tauri.

## Installation

```bash
pnpm add @modrinth/api-client
```

Tauri apps also need the optional peer dependency:

```bash
pnpm add @modrinth/api-client @tauri-apps/plugin-http
```

## Usage

### Generic Node.js or Browser Client

```ts
import { AuthFeature, GenericModrinthClient, type Labrinth } from '@modrinth/api-client'

const client = new GenericModrinthClient({
	userAgent: 'my-app/1.0.0',
	features: [new AuthFeature({ token: 'mrp_...' })],
})

const project: Labrinth.Projects.v2.Project = await client.labrinth.projects_v2.get('sodium')
const members = await client.labrinth.projects_v3.getMembers(project.id)
const servers = await client.archon.servers_v0.list({ limit: 10 })
```

You can still make direct requests through the same platform layer:

```ts
const project = await client.request<Labrinth.Projects.v2.Project>('/project/sodium', {
	api: 'labrinth',
	version: 2,
})
```

### Nuxt

```ts
import {
	AuthFeature,
	CircuitBreakerFeature,
	NuxtCircuitBreakerStorage,
	NuxtModrinthClient,
} from '@modrinth/api-client'

export const useModrinthClient = async () => {
	const config = useRuntimeConfig()
	const auth = await useAuth()

	return new NuxtModrinthClient({
		userAgent: 'my-nuxt-app/1.0.0',
		rateLimitKey: import.meta.server ? config.rateLimitKey : undefined,
		features: [
			new AuthFeature({
				token: async () => auth.value.token,
			}),
			new CircuitBreakerFeature({
				storage: new NuxtCircuitBreakerStorage(),
			}),
		],
	})
}
```

### Tauri

```ts
import { getVersion } from '@tauri-apps/api/app'
import { AuthFeature, TauriModrinthClient } from '@modrinth/api-client'

const version = await getVersion()
const client = new TauriModrinthClient({
	userAgent: `modrinth/theseus/${version} (support@modrinth.com)`,
	features: [new AuthFeature({ token: 'mrp_...' })],
})

const project = await client.labrinth.projects_v2.get('sodium')
```

## Modules

Modules are available as nested properties on the client:

> [!WARNING]
> Modrinth Hosting API modules: You are welcome to use them to access your own server through the API, but they may break at any time because Archon and Kyros are internal services only used by Modrinth. Abuse may lead to your server being suspended without notice.

```ts
client.labrinth.projects_v2
client.labrinth.projects_v3
client.labrinth.versions_v3
client.archon.servers_v0
client.archon.backups_v1
client.kyros.files_v0
```

Types are exported from the package root:

```ts
import type { Archon, Kyros, Labrinth } from '@modrinth/api-client'

const project: Labrinth.Projects.v3.Project = await client.labrinth.projects_v3.get('sodium')
const server: Archon.Servers.v0.Server = await client.archon.servers_v0.get('server-id')
```

## Base URLs

By default, the client uses Modrinth production services:

- `labrinthBaseUrl`: `https://api.modrinth.com`
- `archonBaseUrl`: `https://archon.modrinth.com`

Override them for staging or custom deployments:

```ts
const client = new GenericModrinthClient({
	userAgent: 'my-app/1.0.0',
	labrinthBaseUrl: 'https://staging-api.modrinth.com',
	archonBaseUrl: 'https://staging-archon.modrinth.com',
})
```

External APIs can be targeted per request by passing a full URL as `api` and disabling auth:

```ts
await client.request('/endpoint', {
	api: 'https://example.com',
	version: 1,
	skipAuth: true,
})
```

## Features

Features wrap requests before they reach the platform implementation:

```ts
import { AuthFeature, CircuitBreakerFeature, RetryFeature } from '@modrinth/api-client'

const client = new GenericModrinthClient({
	features: [
		new AuthFeature({ token: async () => getToken() }),
		new RetryFeature({ maxAttempts: 3, backoffStrategy: 'exponential' }),
		new CircuitBreakerFeature({ maxFailures: 3, resetTimeout: 30_000 }),
	],
})
```

Built-in features include authentication, node auth, retries, circuit breaking, panel version headers, and verbose logging.

## Uploads

Upload endpoints return an `UploadHandle<T>` with progress and cancellation support:

```ts
const upload = client.kyros.files_v0.uploadFile(path, file)

upload.onProgress(({ progress }) => {
	console.log(Math.round(progress * 100))
})

await upload.promise
```

Uploads use `XMLHttpRequest` for progress tracking and are only available in browser-capable contexts. `NuxtModrinthClient.upload()` throws during SSR.

## Third-Party API Typings

- This package also includes some third-party API modules and typings used by Modrinth internals. They are not part of the stable public API surface and should be used at your own risk.

## Development

```bash
pnpm --filter @modrinth/api-client build
pnpm --filter @modrinth/api-client lint
# or pnpm prepr:frontend:lib in turborepo root.
```

When adding a module, add it to `src/modules/index.ts` so it is included in the typed client structure.

## License

Licensed under GPL-3.0. See [LICENSE](LICENSE).
