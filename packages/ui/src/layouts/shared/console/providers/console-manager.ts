import type { ComputedRef, Ref } from 'vue'
import { createContext } from '#ui/providers/create-context'
import type { LogSource } from '../types'

export interface ConsoleManagerContext {
	logLines: Ref<string[]>

	logSources?: ComputedRef<LogSource[]>
	activeLogSourceIndex?: Ref<number>

	sendCommand?: (cmd: string) => void
	showCommandInput?: boolean | Ref<boolean> | ComputedRef<boolean>

	loading?: Ref<boolean> | ComputedRef<boolean>

	onClear?: () => void
	onDelete?: () => Promise<void>

	shareDisabled?: Ref<boolean> | ComputedRef<boolean>
}

export const [injectConsoleManager, provideConsoleManager] =
	createContext<ConsoleManagerContext>('ConsolePageLayout', 'consoleManagerContext')
