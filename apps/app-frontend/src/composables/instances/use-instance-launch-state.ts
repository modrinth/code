import { reactive } from 'vue'

const startingInstances = reactive(new Set<string>())
const previewChecks = new Map<string, number>()

export function useInstanceLaunchState() {
	return {
		isStarting: (instanceId: string) => startingInstances.has(instanceId),
		isCheckingPreview: (instanceId: string) => (previewChecks.get(instanceId) ?? 0) > 0,
		async runPreviewCheck<T>(instanceId: string, check: () => Promise<T>): Promise<T> {
			previewChecks.set(instanceId, (previewChecks.get(instanceId) ?? 0) + 1)
			try {
				return await check()
			} finally {
				const remaining = (previewChecks.get(instanceId) ?? 1) - 1
				if (remaining > 0) previewChecks.set(instanceId, remaining)
				else previewChecks.delete(instanceId)
			}
		},
		async run(instanceId: string, launch: () => Promise<void>) {
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
