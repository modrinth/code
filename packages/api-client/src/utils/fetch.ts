import type { RequestOptions } from '../types/request.js'

export function appendRequestParams(url: string, params?: RequestOptions['params']): string {
	if (!params) return url

	const filteredParams: Record<string, string> = {}
	for (const [key, value] of Object.entries(params)) {
		if (value !== undefined && value !== null) {
			filteredParams[key] = String(value)
		}
	}

	const queryString = new URLSearchParams(filteredParams).toString()
	if (!queryString) return url

	return `${url}${url.includes('?') ? '&' : '?'}${queryString}`
}

export function toFetchBody(body: unknown): BodyInit | null | undefined {
	if (!body) return undefined

	if (
		typeof body === 'object' &&
		!(body instanceof FormData) &&
		!(body instanceof URLSearchParams) &&
		!(body instanceof Blob) &&
		!(body instanceof ArrayBuffer) &&
		!ArrayBuffer.isView(body as ArrayBufferView)
	) {
		return JSON.stringify(body)
	}

	return body as BodyInit
}

export async function parseResponseErrorData(response: Response): Promise<unknown> {
	const contentType = response.headers.get('content-type')?.toLowerCase() ?? ''

	try {
		if (contentType.includes('application/json') || contentType.includes('+json')) {
			return await response.json()
		}

		const text = await response.text()
		if (!text) return undefined

		try {
			return JSON.parse(text)
		} catch {
			return text
		}
	} catch {
		return undefined
	}
}
