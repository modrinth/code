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

export interface ServerInviteMigrationBinding {
	modpackName: string | null
	start: () => Promise<boolean>
}

/**
 * Lets a root-level modal take over the server play page's invite flow, so the invite
 * players content is rendered inside the modal that is already open instead of a second modal.
 */
export interface ServerInviteHandoff {
	requested: Ref<boolean>
	binding: ShallowRef<ComputedRef<ServerInviteBinding> | null>
	migration: ShallowRef<ServerInviteMigrationBinding | null>
	/** Set when the user still has to pick which server to invite players to. */
	awaitingServer: Ref<boolean>
	request: () => void
	provide: (binding: ComputedRef<ServerInviteBinding>) => void
	provideMigration: (migration: ServerInviteMigrationBinding) => void
	cancel: () => void
}

export const [injectServerInviteHandoff, provideServerInviteHandoff] =
	createContext<ServerInviteHandoff>('root', 'serverInviteHandoff')

export function createServerInviteHandoff(): ServerInviteHandoff {
	const requested = ref(false)
	const binding = shallowRef<ComputedRef<ServerInviteBinding> | null>(null)
	const migration = shallowRef<ServerInviteMigrationBinding | null>(null)
	const awaitingServer = ref(false)

	function request() {
		binding.value = null
		migration.value = null
		requested.value = true
	}

	function provide(next: ComputedRef<ServerInviteBinding>) {
		if (requested.value) binding.value = next
	}

	function provideMigration(next: ServerInviteMigrationBinding) {
		if (requested.value) migration.value = next
	}

	function cancel() {
		requested.value = false
		binding.value = null
		migration.value = null
	}

	return {
		requested,
		binding,
		migration,
		awaitingServer,
		request,
		provide,
		provideMigration,
		cancel,
	}
}
