![Modrinth Monorepo Cover](/.github/assets/monorepo_cover.png)

# @modrinth/api-client

[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-c78aff?style=for-the-badge)](https://www.typescriptlang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-c78aff?style=for-the-badge)](LICENSE)

A flexible, type-safe API client for Modrinth's APIs (Labrinth, Kyros & Archon). Works across Node.js, browsers, Nuxt, and Tauri with a feature system for authentication, retries, circuit breaking and other custom processing of requests and responses.

## Installation

```bash
pnpm add @modrinth/api-client
# or
npm install @modrinth/api-client
# or
yarn add @modrinth/api-client
```

## Usage

### Plain JavaScript/Node.js

```typescript
import { GenericModrinthClient, AuthFeature, ProjectV2 } from '@modrinth/api-client'

const client = new GenericModrinthClient({
	userAgent: 'my-app/1.0.0',
	features: [new AuthFeature({ token: 'mrp_...' })],
})

// Explicitly make a request using client.request
const project: any = await client.request('/project/sodium', { api: 'labrinth', version: 2 })

// Example for archon (Modrinth Servers)
const servers = await client.request('/servers?limit=10', { api: 'archon', version: 0 })

// Or use the provided wrappers for better type support.
const project: ProjectV2 = await client.projects_v2.get('sodium')
```

### Nuxt

```typescript
import { NuxtModrinthClient, AuthFeature, NuxtCircuitBreakerStorage } from '@modrinth/api-client'

// Alternatively you can create a singleton of the client and provide it via DI.
export const useModrinthClient = () => {
	const config = useRuntimeConfig()

	// example using the useAuth composable from our frontend, replace this with whatever you're using to store auth token
	const auth = await useAuth()

	return new NuxtModrinthClient({
		userAgent: 'my-nuxt-app/1.0.0', // leave blank to use default user agent from fetch function
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

const client = useModrinthClient()
const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
```

### Tauri

```typescript
import { TauriModrinthClient, AuthFeature } from '@modrinth/api-client'
import { getVersion } from '@tauri-apps/api/app'

const version = await getVersion()
const client = new TauriModrinthClient({
	userAgent: `modrinth/theseus/${version} (support@modrinth.com)`,
	features: [new AuthFeature({ token: 'mrp_...' })],
})

const project = await client.request('/project/sodium', { api: 'labrinth', version: 2 })
```

### Overriding Base URLs

By default, the client uses the production base URLs:

- `labrinthBaseUrl`: `https://api.modrinth.com/` (Labrinth API)
- `archonBaseUrl`: `https://archon.modrinth.com/` (Archon/Servers API)

You can override these for staging environments or custom instances:

```typescript
const client = new GenericModrinthClient({
	userAgent: 'my-app/1.0.0',
	labrinthBaseUrl: 'https://staging-api.modrinth.com/',
	archonBaseUrl: 'https://staging-archon.modrinth.com/',
	features: [new AuthFeature({ token: 'mrp_...' })],
})

// Now requests will use the staging URLs
await client.request('/project/sodium', { api: 'labrinth', version: 2 })
// -> https://staging-api.modrinth.com/v2/project/sodium
```

You can also use custom URLs directly in requests:

```typescript
// One-off custom URL (useful for Kyros nodes or dynamic endpoints)
await client.request('/some-endpoint', {
	api: 'https://eu-lim16.nodes.modrinth.com/',
	version: 0,
})
```

## Features

### Authentication

Supports both static and dynamic tokens:

```typescript
// Static token
new AuthFeature({ token: 'mrp_...' })

// Dynamic token (e.g., from auth state)
const auth = await useAuth()
new AuthFeature({
	token: async () => auth.value.token,
})
```

### Retry

Automatically retries failed requests with configurable backoff:

```typescript
new RetryFeature({
	maxAttempts: 3,
	backoffStrategy: 'exponential',
	initialDelay: 1000,
	maxDelay: 15000,
})
```

### Circuit Breaker

Prevents cascade failures by opening circuits after repeated failures:

```typescript
new CircuitBreakerFeature({
	maxFailures: 3,
	resetTimeout: 30000,
	failureStatusCodes: [500, 502, 503, 504],
})
```

## Documentation

This package is **self-documenting** through TypeScript types and JSDoc comments. Use your IDE's IntelliSense to explore available methods, classes, and configuration options.

For Modrinth API endpoints and routes, refer to the [Modrinth API Documentation](https://docs.modrinth.com).

## Contributing

- Modules are available in the `modules/<api>/...` folders.
  - When a module has different versions available, you should do it like so: `modules/labrinth/projects/v2.ts` etc.
  - Types for a module's requests should be made available in `modules/<api>/module/types.ts` or `.../types/v2.ts`.
  - You should expose these types in the `modules/types.ts` file.
- When creating a new module, add it to the `modules/index.ts`'s `MODULE_REGISTRY` for it to become available in the api client class.

Dont forget to run `pnpm fix` before committing.

## License

Licensed under GPL-3.0 - see the [LICENSE](LICENSE) file for details.
