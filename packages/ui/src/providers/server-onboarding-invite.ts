import { type ShallowRef, shallowRef } from 'vue'

import { createContext } from './create-context'

export interface ServerOnboardingInviteRequest {
	serverId: string
	worldId: string
	siteUrl: string
}

export interface ServerOnboardingInviteFlow {
	request: ShallowRef<ServerOnboardingInviteRequest | null>
	open: (request: ServerOnboardingInviteRequest) => Promise<void>
	markShown: () => void
	fail: (error: unknown) => void
	clear: () => void
}

export const [injectServerOnboardingInviteFlow, provideServerOnboardingInviteFlow] =
	createContext<ServerOnboardingInviteFlow>('root', 'serverOnboardingInviteFlow')

export function createServerOnboardingInviteFlow(): ServerOnboardingInviteFlow {
	const request = shallowRef<ServerOnboardingInviteRequest | null>(null)
	let resolveShown: (() => void) | null = null
	let rejectShown: ((error: unknown) => void) | null = null

	function open(nextRequest: ServerOnboardingInviteRequest) {
		return new Promise<void>((resolve, reject) => {
			resolveShown = resolve
			rejectShown = reject
			request.value = nextRequest
		})
	}

	function markShown() {
		resolveShown?.()
		resolveShown = null
		rejectShown = null
	}

	function fail(error: unknown) {
		request.value = null
		rejectShown?.(error)
		resolveShown = null
		rejectShown = null
	}

	function clear() {
		request.value = null
		resolveShown = null
		rejectShown = null
	}

	return { request, open, markShown, fail, clear }
}
