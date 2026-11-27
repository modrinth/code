import { createContext } from '@modrinth/ui'

export interface InstanceCreationContext {
	showCreationModal: () => void
	showImportModal: () => Promise<void>
}

export const [injectInstanceCreation, provideInstanceCreation] =
	createContext<InstanceCreationContext>('root', 'instanceCreation')
