import type { Ref } from 'vue'
import { computed, ref } from 'vue'

export interface FormRequestAttributes {
	form_type: 'W-9' | 'W-8BEN' | 'W-8BEN-E' | string
	company_id: number
	company_name: string
	company_email: string
	reference_id: string | null
	form_id: string | null
	signed_at: string | null
	tin_match_status: string | null
	expires_at: string | null
}

export interface FormRequestLinks {
	action_validate?: string
	action_complete?: string
	[key: string]: unknown
}

export interface FormRequestData {
	id: string
	type: 'form_request' | string
	attributes: FormRequestAttributes
	links?: FormRequestLinks
	[key: string]: unknown
}

export interface FormRequestResponse {
	data: FormRequestData
	[key: string]: unknown
}

export interface UseAvalara1099Options {
	prefill?: Record<string, unknown>
	// Optional override for the origin (defaults to vendor CDN domain)
	origin?: string
	// Optional hook to further style the injected dialog/iframe
	styleDialog?: (dialog: HTMLDialogElement, iframe: HTMLIFrameElement | null) => void
	// Poll interval while waiting for global to appear
	pollIntervalMs?: number
	// Max time to wait for script before rejecting (ms); 0/undefined => no timeout
	timeoutMs?: number
}

interface AvalaraGlobal {
	requestW9: (
		formRequest: FormRequestResponse | FormRequestData,
		opts?: { prefill?: Record<string, unknown> },
	) => Promise<FormRequestResponse> | any
	requestW8BEN: (
		formRequest: FormRequestResponse | FormRequestData,
		opts?: { prefill?: Record<string, unknown> },
	) => Promise<FormRequestResponse> | any
	requestW8BENE: (
		formRequest: FormRequestResponse | FormRequestData,
		opts?: { prefill?: Record<string, unknown> },
	) => Promise<FormRequestResponse> | any
	origin?: string
	[key: string]: unknown
}

declare global {
	interface Window {
		Avalara1099?: AvalaraGlobal
	}
}

const injectedKey = '__avalara1099_script_injected__'

function ensureScriptInjected(origin: string) {
	if (import.meta.server) return
	const w = window as any
	if (w[injectedKey]) return
	w[injectedKey] = true
	useHead({
		script: [
			{
				src: `${origin.replace(/\/$/, '')}/api/request_form.js`,
				crossorigin: 'anonymous',
				type: 'module',
			},
		],
	})
}

async function waitForAvalara(opts: {
	pollIntervalMs: number
	timeoutMs?: number
	origin: string
}): Promise<AvalaraGlobal> {
	if (import.meta.server) throw new Error('Avalara 1099 is client-side only')
	ensureScriptInjected(opts.origin)
	const start = Date.now()
	return await new Promise((resolve, reject) => {
		const poll = () => {
			const g = window.Avalara1099
			if (g) return resolve(g)
			if (opts.timeoutMs && opts.timeoutMs > 0 && Date.now() - start > opts.timeoutMs) {
				return reject(new Error('Timed out waiting for Avalara1099 script to load'))
			}
			setTimeout(poll, opts.pollIntervalMs)
		}
		poll()
	})
}

export function useAvalara1099(
	initial: FormRequestResponse | FormRequestData,
	options: UseAvalara1099Options = {},
) {
	const origin = options.origin || 'https://www.track1099.com'
	const pollIntervalMs = options.pollIntervalMs ?? 250
	const timeoutMs = options.timeoutMs

	const request: Ref<FormRequestResponse | FormRequestData> = ref(initial)
	const loading = ref(false)
	const error: Ref<unknown> = ref(null)

	const signedAt = computed(() => {
		const data = (request.value as FormRequestResponse).data || request.value
		return data.attributes?.signed_at ? new Date(data.attributes.signed_at) : null
	})

	const status = computed(() => (signedAt.value ? 'signed' : 'incomplete'))

	async function start(): Promise<FormRequestResponse | FormRequestData> {
		loading.value = true
		error.value = null
		try {
			const g = await waitForAvalara({ pollIntervalMs, timeoutMs, origin })
			const data = (request.value as FormRequestResponse).data || request.value
			const formType = data.attributes?.form_type

			// Defensive deep clone to strip proxies / non-cloneable refs before postMessage
			// (DataCloneError guard)
			let safeRequest: any
			try {
				safeRequest = JSON.parse(JSON.stringify(request.value))
			} catch {
				// Fallback shallow copy
				safeRequest = Array.isArray(request.value)
					? [...(request.value as any)]
					: { ...(request.value as any) }
			}
			let safePrefill: any = undefined
			if (options.prefill) {
				try {
					safePrefill = JSON.parse(JSON.stringify(options.prefill))
				} catch {
					safePrefill = { ...options.prefill }
				}
			}

			let promise: any
			if (formType === 'W-8BEN') {
				promise = g.requestW8BEN(safeRequest, { prefill: safePrefill })
			} else if (formType === 'W-9') {
				promise = g.requestW9(safeRequest, { prefill: safePrefill })
			} else if (formType === 'W-8BEN-E' || formType === 'W-8BEN E') {
				promise = g.requestW8BENE(safeRequest, { prefill: safePrefill })
			} else {
				throw new Error(`Unsupported form_type: ${formType}`)
			}

			// The vendor promise resolves with an updated form request (signed state)
			const newReq = await promise
			request.value = newReq
			return newReq
		} catch (e) {
			error.value = e
			throw e
		} finally {
			loading.value = false
		}
	}

	return {
		start,
		request,
		signedAt,
		status, // 'signed' | 'incomplete'
		loading,
		error,
	}
}
