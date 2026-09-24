import { type Ref, ref, type ShallowRef, shallowRef } from 'vue'

import { createContext } from './create-context'

export interface ServerOnboardingRequest {
	serverId: string
	worldId: string
	siteUrl: string
	project?: {
		projectId: string
		versionId: string
		name: string
		iconUrl?: string
		contentType?: string
	}
	backToBrowse?: boolean
	onHide?: () => void
}

export interface ServerOnboardingFlow {
	request: ShallowRef<ServerOnboardingRequest | null>
	inviteActive: Ref<boolean>
	open: (request: ServerOnboardingRequest) => Promise<void>
	markShown: () => void
	fail: (error: unknown) => void
	clear: () => void
}

export const [injectServerOnboardingFlow, provideServerOnboardingFlow] =
	createContext<ServerOnboardingFlow>('root', 'serverOnboardingFlow')

export function createServerOnboardingFlow(): ServerOnboardingFlow {
	const request = shallowRef<ServerOnboardingRequest | null>(null)
	const inviteActive = ref(false)
	let pending: Promise<void> | null = null
	let resolveShown: (() => void) | null = null
	let rejectShown: ((error: unknown) => void) | null = null

	function open(nextRequest: ServerOnboardingRequest) {
		if (pending) return pending
		pending = new Promise<void>((resolve, reject) => {
			resolveShown = resolve
			rejectShown = reject
			inviteActive.value = false
			request.value = nextRequest
		})
		return pending
	}

	function markShown() {
		resolveShown?.()
		pending = null
		resolveShown = null
		rejectShown = null
	}

	function fail(error: unknown) {
		rejectShown?.(error)
		clear()
	}

	function clear() {
		request.value = null
		inviteActive.value = false
		pending = null
		resolveShown = null
		rejectShown = null
	}

	return { request, inviteActive, open, markShown, fail, clear }
}
