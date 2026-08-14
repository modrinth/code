import { createContext } from '@modrinth/ui'
import type { ComputedRef, Ref } from 'vue'

import type { ShareRow } from './shared-instance-share-types'

export interface SharedInstanceManagementContext {
	readonly rows: ComputedRef<ShareRow[]>
	readonly actionsLocked: Ref<boolean>
	readonly inviteDisabled: ComputedRef<boolean>
	readonly invitePending: Ref<boolean>
	readonly pushUpdateDisabled: ComputedRef<boolean>
	readonly pushUpdatePending: ComputedRef<boolean>
	invite: (event: MouseEvent) => void
	remove: (row: ShareRow) => void
	pushUpdate: (event: MouseEvent) => void
}

export const [injectSharedInstanceManagement, provideSharedInstanceManagement] =
	createContext<SharedInstanceManagementContext>('InstanceSharePage')
