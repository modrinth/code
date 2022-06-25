/* eslint-disable @typescript-eslint/ban-ts-comment */ /* `svelte-check` doesn't find issues but VSCode does */
// @ts-ignore: `$stores/account` needs to be created in consumer package
import { token as tokenStore } from '$stores/account'
import { get, writable } from 'svelte/store'
import type { operations } from '$generated/openapi'

export const fetching = writable<number>(0)

type method = 'GET' | 'POST' | 'PATCH' | 'DELETE' | 'HEAD'

/* On get requests with query params, pass them in data */
export async function send<Operation extends keyof operations>(
	method: method,
	route: string,
	data: // @ts-ignore: Not always present
	| (operations[Operation]['requestBody']['content']['application/json'] &
				// @ts-ignore
				operations[Operation]['requestBody']['content']['multipart/form-data'] &
				// @ts-ignore
				operations[Operation]['parameters']['query'])
		| FormData
		| null = null,
	options: {
		token?: string
		fetch?: (info: RequestInfo, init?: RequestInit) => Promise<Response>
		file?: File
	} = {
		token: '',
	}
): Promise<
	// @ts-ignore: On some API routes, a response body is available, if not, defaults to `unknown`
	operations[Operation]['responses'][200]['content']['application/json']
> {
	fetching.set(get(fetching) + 1)

	const fetchOptions: RequestInit = {
		method,
		headers: {},
	}

	const token = get(tokenStore) || options.token
	if (token) {
		fetchOptions.headers['Authorization'] = token
	}

	let url = (import.meta.env.VITE_API_URL || 'https://api.modrinth.com/v2/') + route

	if (data) {
		if (data instanceof FormData) {
			fetchOptions.body = data
		} else {
			if (method === 'GET' || options.file) {
				url += '?' + new URLSearchParams(data as Record<string, any>).toString()
			} else {
				fetchOptions.headers['Content-Type'] = 'application/json'
				fetchOptions.body = JSON.stringify(data)
			}
			if (options.file) {
				fetchOptions.headers['Content-Type'] = options.file.type
				fetchOptions.body = options.file
			}
		}
	}

	const response = await (options.fetch || fetch)(url, fetchOptions)

	fetching.set(get(fetching) - 1)

	if (!response.ok) {
		throw response
	}

	let parsed: any
	if (response.status !== 204) {
		try {
			parsed = await response.json()
		} catch {
			console.error('Could not parse API response')
		}
	}

	return parsed
}
