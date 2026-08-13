- [Add an API module](#add-an-api-module)
	- [Procedure](#procedure)
		- [1. Define types in `types.ts`](#1-define-types-in-typests)
		- [2. Create the module class](#2-create-the-module-class)
			- [Request options](#request-options)
			- [File uploads](#file-uploads)
		- [3. Register the module](#3-register-the-module)
		- [4. Export types](#4-export-types)
	- [Naming conventions](#naming-conventions)
	- [Key files](#key-files)

# Add an API Module

Use this procedure to add an API endpoint module to `packages/api-client`.

## Procedure

### 1. Define Types in `types.ts`

Make the types match the backend API response exactly. Do not change, rename, or remove fields.

Add the types to an existing namespace, or make a new namespace:

```ts
// modules/labrinth/types.ts (existing namespace)
export namespace Labrinth {
	export namespace MyDomain {
		export namespace v3 {
			export type Thing = {
				id: string
				name: string
				created: string
				// Match the API response exactly.
			}

			export type CreateThingRequest = {
				name: string
			}
		}
	}
}
```

For a new API service, make `modules/<service>/types.ts` with a new top-level namespace. Export it from `modules/types.ts`.

### 2. Create the Module Class

Make `modules/<api>/<domain>/v<N>.ts`:

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

#### Request Options

| Field         | Values                                            | Purpose                                |
| ------------- | ------------------------------------------------- | -------------------------------------- |
| `api`         | `'labrinth'`, `'archon'`, or a full URL           | Select the base URL.                   |
| `version`     | `2`, `3`, `'internal'`, `'modrinth/v0'`, and more | Set the URL version segment.           |
| `method`      | `'GET'`, `'POST'`, `'PUT'`, `'PATCH'`, `'DELETE'` | Set the HTTP method.                   |
| `body`        | object                                            | Set the JSON request body.             |
| `params`      | `Record<string, string>`                          | Set the query parameters.              |
| `skipAuth`    | `boolean`                                         | Bypass the authentication feature.     |
| `useNodeAuth` | `boolean`                                         | Use node-level Kyros authentication.   |
| `timeout`     | `number`                                          | Set the request timeout in milliseconds. |
| `retry`       | `boolean \| number`                               | Override the retry behavior.           |

#### File Uploads

Return an `UploadHandle` instead of a `Promise`:

```ts
public uploadThing(id: string, file: File): UploadHandle<void> {
	return this.client.upload<void>(`/thing/${id}/file`, {
		api: 'labrinth',
		version: 3,
		file,
	})
}

// Use FormData for a multipart upload.
public createWithFiles(data: CreateRequest, files: File[]): UploadHandle<Thing> {
	const formData = new FormData()
	formData.append('data', JSON.stringify(data))
	files.forEach((f, i) => formData.append(`file-${i}`, f, f.name))

	return this.client.upload<Thing>(`/thing`, {
		api: 'labrinth',
		version: 3,
		formData,
		timeout: 60 * 5 * 1000, // Use a longer upload timeout.
	})
}
```

### 3. Register the Module

Add the module to `MODULE_REGISTRY` in `modules/index.ts`:

```ts
import { LabrinthThingsV3Module } from './labrinth/things/v3'

export const MODULE_REGISTRY = {
	// Existing modules.
	labrinth_things_v3: LabrinthThingsV3Module,
} as const
```

Use `<api>_<domain>_<version>` for the key. The client converts this flat key to `client.labrinth.things_v3`.

### 4. Export Types

Types in an existing namespace already have an export. For a new `types.ts`, add this export to `modules/types.ts`:

```ts
export * from './<service>/types'
```

## Naming Conventions

| Item           | Example                         | Pattern                     |
| -------------- | ------------------------------- | --------------------------- |
| Module class   | `LabrinthThingsV3Module`        | `{Api}{Domain}V{N}Module`   |
| Module ID      | `labrinth_things_v3`            | `{api}_{domain}_v{n}`       |
| Type namespace | `Labrinth.MyDomain.v3.Thing`    | `Api.Domain.version.Type`   |
| File path      | `modules/labrinth/things/v3.ts` | `modules/api/domain/vN.ts`  |

## Key Files

- `src/core/abstract-module.ts`: Base class for all modules.
- `src/core/abstract-client.ts`: Contains the `request()` and `upload()` methods.
- `src/modules/index.ts`: Contains `MODULE_REGISTRY` and `buildModuleStructure()`.
- `src/modules/<api>/types.ts`: Contains the types for each API.
- `src/types/upload.ts`: Contains `UploadHandle`, `UploadProgress`, and `UploadRequestOptions`.
