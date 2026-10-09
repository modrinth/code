import { type ComputedRef, type Ref, ref, type ShallowRef, shallowRef } from 'vue'

import type {
	InvitePlayersContentProps,
	InvitePlayersInvitePayload,
	InvitePlayersUser,
} from '#ui/components/sharing'

import { createContext } from './create-context'

export interface ServerInviteBinding {
	header: string
	props: InvitePlayersContentProps
	onInvite: (payload: InvitePlayersInvitePayload) => void
	onCancel: (user: InvitePlayersUser) => void
}

/**
 * Lets a root-level modal take over the server play page's invite flow, so the invite
 * players content is rendered inside the modal that is already open instead of a second modal.
 */
export interface ServerInviteHandoff {
	requested: Ref<boolean>
	binding: ShallowRef<ComputedRef<ServerInviteBinding> | null>
	request: () => void
	provide: (binding: ComputedRef<ServerInviteBinding>) => void
	cancel: () => void
}

export const [injectServerInviteHandoff, provideServerInviteHandoff] =
	createContext<ServerInviteHandoff>('root', 'serverInviteHandoff')

export function createServerInviteHandoff(): ServerInviteHandoff {
	const requested = ref(false)
	const binding = shallowRef<ComputedRef<ServerInviteBinding> | null>(null)

	function request() {
		binding.value = null
		requested.value = true
	}

	function provide(next: ComputedRef<ServerInviteBinding>) {
		if (requested.value) binding.value = next
	}

	function cancel() {
		requested.value = false
		binding.value = null
	}

	return { requested, binding, request, provide, cancel }
}
