import { createContext, type ManagedContentModalState } from '@modrinth/ui'
import { type ShallowRef, shallowRef } from 'vue'

export interface SavedManagedContentModalState {
	instanceId: string
	state: ManagedContentModalState
}

export type ManagedContentModalMemory = ShallowRef<SavedManagedContentModalState | null>

export const [injectManagedContentModalMemory, provideManagedContentModalMemory] =
	createContext<ManagedContentModalMemory>('root', 'managedContentModalMemory')

export function createManagedContentModalMemory(): ManagedContentModalMemory {
	return shallowRef(null)
}
