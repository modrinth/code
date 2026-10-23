import { createContext } from '@modrinth/ui'
import { reactive } from 'vue'

export interface InstanceLaunchState {
	isStarting: (instanceId: string) => boolean
	isCheckingPreview: (instanceId: string) => boolean
	runPreviewCheck: <T>(instanceId: string, check: () => Promise<T>) => Promise<T>
	run: (instanceId: string, launch: () => Promise<void>) => Promise<void>
}

/** Tracks instances that are launching or checking for shared updates, so duplicate launches are ignored. */
export function createInstanceLaunchState(): InstanceLaunchState {
	const startingInstances = reactive(new Set<string>())
	const previewChecks = new Map<string, number>()

	return {
		isStarting: (instanceId) => startingInstances.has(instanceId),
		isCheckingPreview: (instanceId) => (previewChecks.get(instanceId) ?? 0) > 0,
		async runPreviewCheck(instanceId, check) {
			previewChecks.set(instanceId, (previewChecks.get(instanceId) ?? 0) + 1)
			try {
				return await check()
			} finally {
				const remaining = (previewChecks.get(instanceId) ?? 1) - 1
				if (remaining > 0) previewChecks.set(instanceId, remaining)
				else previewChecks.delete(instanceId)
			}
		},
		async run(instanceId, launch) {
			if (startingInstances.has(instanceId)) return
			startingInstances.add(instanceId)
			try {
				await launch()
			} finally {
				startingInstances.delete(instanceId)
			}
		},
	}
}

export const [injectInstanceLaunchState, provideInstanceLaunchState] =
	createContext<InstanceLaunchState>('root', 'instanceLaunchState')
