# Adding a New API Module

How to add a new API endpoint module to `packages/api-client`.

## Steps

### 1. Define types in the module's `types.ts`

Types must match 1:1 with the backend API response. Do not reshape, rename, or omit fields.

Add to an existing namespace or create a new one:

```ts
// modules/labrinth/types.ts (existing namespace)
export namespace Labrinth {
	export namespace MyDomain {
		export namespace v3 {
			export type Thing = {
				id: string
				name: string
				created: string
				// ... matches API response exactly
			}

			export type CreateThingRequest = {
				name: string
			}
		}
	}
}
```

For a new API service, create `modules/<service>/types.ts` with a new top-level namespace and re-export it from `modules/types.ts`.

### 2. Create the module class

Create `modules/<api>/<domain>/v<N>.ts`:

```ts
// modules/labrinth/things/v3.ts
import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthThingsV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_things_v3'
	}

	public async get(id: string): Promise<Labrinth.MyDomain.v3.Thing> {
		return this.client.request<Labrinth.MyDomain.v3.Thing>(`/thing/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'GET',
		})
	}

	public async create(data: Labrinth.MyDomain.v3.CreateThingRequest): Promise<Labrinth.MyDomain.v3.Thing> {
		return this.client.request<Labrinth.MyDomain.v3.Thing>(`/thing`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}

	public async delete(id: string): Promise<void> {
		return this.client.request(`/thing/${id}`, {
			api: 'labrinth',
			version: 3,
			method: 'DELETE',
		})
	}
}
```

#### Request options

| Field | Values | Purpose |
|-------|--------|---------|
| `api` | `'labrinth'`, `'archon'`, or a full URL | Which base URL to use |
| `version` | `2`, `3`, `'internal'`, `'modrinth/v0'`, etc. | URL version segment |
| `method` | `'GET'`, `'POST'`, `'PUT'`, `'PATCH'`, `'DELETE'` | HTTP method |
| `body` | object | JSON request body |
| `params` | `Record<string, string>` | Query parameters |
| `skipAuth` | `boolean` | Skip auth feature for this request |
| `useNodeAuth` | `boolean` | Use node-level auth (kyros) |
| `timeout` | `number` | Request timeout in ms |
| `retry` | `boolean \| number` | Override retry behavior |

#### For uploads

Return an `UploadHandle` instead of a `Promise`:

```ts
public uploadThing(id: string, file: File): UploadHandle<void> {
	return this.client.upload<void>(`/thing/${id}/file`, {
		api: 'labrinth',
		version: 3,
		file,
	})
}

// Or with FormData for multipart:
public createWithFiles(data: CreateRequest, files: File[]): UploadHandle<Thing> {
	const formData = new FormData()
	formData.append('data', JSON.stringify(data))
	files.forEach((f, i) => formData.append(`file-${i}`, f, f.name))

	return this.client.upload<Thing>(`/thing`, {
		api: 'labrinth',
		version: 3,
		formData,
		timeout: 60 * 5 * 1000,  // longer timeout for uploads
	})
}
```

### 3. Register in the MODULE_REGISTRY

Add to `modules/index.ts`:

```ts
import { LabrinthThingsV3Module } from './labrinth/things/v3'

export const MODULE_REGISTRY = {
	// ... existing modules
	labrinth_things_v3: LabrinthThingsV3Module,
} as const
```

The naming convention is `<api>_<domain>_<version>`. This flat key gets transformed into nested access: `client.labrinth.things_v3`.

### 4. Export types

If you added to an existing namespace, types are already re-exported. If you created a new `types.ts`, add it to `modules/types.ts`:

```ts
export * from './<service>/types'
```

## Naming Conventions

| Convention | Example |
|-----------|---------|
| Module class | `LabrinthThingsV3Module` — `{Api}{Domain}V{N}Module` |
| Module ID | `labrinth_things_v3` — `{api}_{domain}_v{n}` |
| Type namespace | `Labrinth.MyDomain.v3.Thing` |
| File path | `modules/labrinth/things/v3.ts` |

## Key Files

- `src/core/abstract-module.ts` — base class all modules extend
- `src/core/abstract-client.ts` — `request()` and `upload()` methods
- `src/modules/index.ts` — `MODULE_REGISTRY` and `buildModuleStructure()`
- `src/modules/<api>/types.ts` — type definitions per API
- `src/types/upload.ts` — `UploadHandle`, `UploadProgress`, `UploadRequestOptions`
