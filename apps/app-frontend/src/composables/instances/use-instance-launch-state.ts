import { reactive } from 'vue'

const startingInstances = reactive(new Set<string>())

export function useInstanceLaunchState() {
	return {
		isStarting: (instanceId: string) => startingInstances.has(instanceId),
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
