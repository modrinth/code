# @modrinth/api-client

Platform-agnostic API client for Modrinth's services. Works in Nuxt (SSR + CSR), Tauri (desktop app), and plain Node/browser environments.

## Architecture

```
Request Flow:
  Module Method → client.request() → Feature Chain (middleware) → Platform executeRequest()
```

### Key Directories

- **`src/core/`** — base classes (`AbstractModrinthClient`, `AbstractModule`, `AbstractFeature`, etc.)
- **`src/platform/`** — platform implementations (generic, nuxt, tauri, xhr-upload, websocket)
- **`src/features/`** — middleware plugins (auth, retry, circuit-breaker, etc.)
- **`src/modules/`** — API endpoint modules organized by service (`labrinth/`, `archon/`, `kyros/`, `iso3166/`)
- **`src/types/`** — core type definitions (client config, request options, upload types, errors)

### Client Hierarchy

All platform clients extend `XHRUploadClient` → `AbstractModrinthClient`:

- **`GenericModrinthClient`** — uses `ofetch`, attaches WebSocket client to `archon.sockets`
- **`NuxtModrinthClient`** — uses Nuxt's `$fetch`, SSR-aware, blocks `upload()` during SSR
- **`TauriModrinthClient`** — uses `@tauri-apps/plugin-http`

### Module Access

Modules are lazy-loaded and accessed as a nested structure:

```ts
client.labrinth.projects_v2
client.labrinth.projects_v3
client.labrinth.versions_v3
client.labrinth.collections
client.labrinth.billing_internal
client.archon.servers_v0
client.archon.servers_v1
client.archon.backups_v0
client.archon.backups_v1
client.archon.content_v0
client.kyros.files_v0
client.iso3166.data
... ect.
```

This structure is derived at runtime from the flat `MODULE_REGISTRY` in `modules/index.ts` via `buildModuleStructure()`, and the TypeScript types are inferred automatically via `InferredClientModules`.

## Usage

The client is provided to the component tree via DI (see the `dependency-injection` skill). Each app creates a platform-specific client and provides it at the root:

```ts
// apps/frontend/src/app.vue (Nuxt)
const client = new NuxtModrinthClient({ ... })
provideModrinthClient(client)

// apps/app-frontend/src/App.vue (Tauri)
const client = new TauriModrinthClient({ ... })
provideModrinthClient(client)
```

Components anywhere in the tree then inject it:

```ts
const { labrinth, archon, kyros } = injectModrinthClient()

// Fetch data
const project = await labrinth.projects_v3.get(projectId)

// Use with TanStack Query
const { data } = useQuery({
	queryKey: ['project', projectId],
	queryFn: () => labrinth.projects_v3.get(projectId),
})
```

`provideModrinthClient` and `injectModrinthClient` are exported from `@modrinth/ui` (defined in `packages/ui/src/providers/api-client.ts`). The provider is typed as `AbstractModrinthClient`, so shared components in `packages/ui` work with any platform client.

## Types

Types must match 1:1 with how they are returned from the backend API they are fetching from. Do not reshape, rename, or omit fields — the types should be a direct representation of the API response.

Types are organized in namespaces that mirror the backend services:

```ts
import type { Labrinth, Archon, Kyros, ISO3166 } from '@modrinth/api-client'

const project: Labrinth.Projects.v3.Project = ...
const server: Archon.Servers.v0.Server = ...
const auth: Archon.Websocket.v0.WSAuth = ...
```

Each API has a `types.ts` in its module directory (`modules/labrinth/types.ts`, `modules/archon/types.ts`, etc.) using nested namespaces: `Namespace.Domain.Version.Type`.

## Features (Middleware)

Features wrap requests in a chain. Each feature can modify the request, retry, or short-circuit:

- **`AuthFeature`** — injects `Authorization: Bearer <token>`, supports async token providers
- **`RetryFeature`** — exponential/linear/constant backoff, retries on 408/429/5xx and network errors
- **`CircuitBreakerFeature`** — opens after N consecutive failures per endpoint, resets after timeout

## XHR Upload

File uploads use `XMLHttpRequest` for progress tracking (not available via `fetch`). The `upload()` method returns an `UploadHandle<T>`:

```ts
interface UploadHandle<T> {
	promise: Promise<T>
	onProgress(callback: (progress: UploadProgress) => void): UploadHandle<T> // chainable
	cancel(): void
}
```

Supports two modes:

- **Single file** — `{ file: File | Blob }` sends with `Content-Type: application/octet-stream`
- **FormData** — `{ formData: FormData }` for multipart uploads (browser/platform sets boundary)

Uploads go through the feature chain (auth, retry, etc.). Features detect uploads via `context.metadata.isUpload`.

### Usage Example (server file upload)

```ts
const uploader = client.kyros.files_v0.uploadFile(path, file, {
	onProgress: ({ progress }) => {
		uploadProgress.value = Math.round(progress * 100)
	},
})
// Cancel if needed: uploader.cancel()
await uploader.promise
```

### Usage Example (version creation with FormData)

```ts
const handle = client.labrinth.versions_v3.createVersion(draftVersion, files, projectType)
handle.onProgress((progress) => {
	uploadProgress.value = progress
})
await handle.promise
```

See `packages/ui/src/components/servers/files/upload/FileUploadDropdown.vue` and `apps/frontend/src/providers/version/manage-version-modal.ts` for real usage.

## WebSocket

WebSocket support is attached to `client.archon.sockets` (only on `GenericModrinthClient`). It provides event-based communication with Modrinth Hosting servers.

### Connection Flow

```
client.archon.sockets.safeConnect(serverId)
  → fetches JWT auth via archon.servers_v0.getWebSocketAuth()
  → opens wss:// connection
  → sends { event: 'auth', jwt: token }
  → server responds with { event: 'auth-ok' }
  → ready to receive events
```

Auto-reconnects on unexpected disconnection with exponential backoff (base 1s, max 30s, up to 10 attempts).

### Subscribing to Events

```ts
const unsub = client.archon.sockets.on(serverId, 'stats', (data) => {
	// data is typed as Archon.Websocket.v0.WSStatsEvent
	cpuUsage.value = data.cpu_percent
})

// Clean up
onUnmounted(() => {
	unsub()
	client.archon.sockets.disconnect(serverId)
})
```

Event types: `log`, `stats`, `power-state`, `uptime`, `backup-progress`, `installation-result`, `filesystem-ops`, `new-mod`, `auth-expiring`, `auth-incorrect`, `auth-ok`.

### Sending Commands

```ts
client.archon.sockets.send(serverId, { event: 'command', cmd: '/say hello' })
```

See `apps/frontend/src/pages/hosting/manage/[id].vue` for the full server panel WebSocket usage.

## Adding a New API Module

See the `api-module` skill (`.claude/skills/api-module/SKILL.md`) for step-by-step instructions.
