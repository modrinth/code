import type { Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type { SharingInvite } from '../types'

export interface SharingSettingsContext {
	targetKey: Readonly<Ref<string | null>>
	invites: Readonly<Ref<SharingInvite[]>>
	loading: Readonly<Ref<boolean>>
	error: Readonly<Ref<unknown>>
	busy: Readonly<Ref<boolean>>
	refresh: () => Promise<void>
	revokeInvite: (inviteId: string, targetKey: string) => Promise<void>
	unpublish: (targetKey: string) => Promise<void>
	onError: (error: unknown) => void
	reviewChangesBeforePlaying?: Ref<boolean>
}

export const [injectSharingSettings, provideSharingSettings] =
	createContext<SharingSettingsContext>('SharingSettingsLayout', 'sharingSettings')
