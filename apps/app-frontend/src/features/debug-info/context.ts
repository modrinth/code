import { createContext } from '@modrinth/ui'
import type { Ref } from 'vue'

import type { DebugInfoExportProgress } from '@/platform/app-lib/debug-info/commands'

export interface DebugInfoExportInstance {
	id: string
	name: string
	iconPath: string | null
}

export interface DebugInfoExportTask extends DebugInfoExportProgress {
	id: string
	path: string
	instance: DebugInfoExportInstance | null
	status: 'running' | 'succeeded' | 'failed' | 'canceled'
	started: boolean
	canceling: boolean
	createdAt: string
	finishedAt?: string
	error?: string
	cancel: () => Promise<void>
}

export interface DebugInfoExportsContext {
	tasks: Ref<Map<string, DebugInfoExportTask>>
	isExporting: (instanceId?: string | null) => boolean
	exportDebugInfo: (title: string, instance?: DebugInfoExportInstance | null) => Promise<void>
}

export const [injectDebugInfoExports, provideDebugInfoExports] =
	createContext<DebugInfoExportsContext>('root', 'debugInfoExports')
