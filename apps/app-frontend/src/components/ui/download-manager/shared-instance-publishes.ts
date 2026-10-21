import { reactive, ref } from 'vue'

const RATE_WINDOW_MS = 5000

export interface SharedInstancePublishTask {
	id: string
	instanceId: string
	instanceName: string
	iconPath: string | null
	status: 'running' | 'succeeded' | 'failed'
	uploadedBytes: number
	totalBytes: number
	rate: number
	createdAt: string
	finishedAt?: string
	error?: string
	details?: string
}

export const sharedInstancePublishes = ref(new Map<string, SharedInstancePublishTask>())

export function findRunningSharedInstancePublish(instanceId: string) {
	for (const task of sharedInstancePublishes.value.values()) {
		if (task.instanceId === instanceId && task.status === 'running') return task
	}
	return null
}

export function trackSharedInstancePublish(instance: {
	id: string
	name: string
	icon_path?: string | null
}) {
	const task = reactive<SharedInstancePublishTask>({
		id: `shared-instance-publish-${crypto.randomUUID()}`,
		instanceId: instance.id,
		instanceName: instance.name,
		iconPath: instance.icon_path ?? null,
		status: 'running',
		uploadedBytes: 0,
		totalBytes: 0,
		rate: 0,
		createdAt: new Date().toISOString(),
	})
	sharedInstancePublishes.value.set(task.id, task)
	const samples: { time: number; bytes: number }[] = []

	function finish(status: SharedInstancePublishTask['status']) {
		task.status = status
		task.finishedAt = new Date().toISOString()
		task.rate = 0
	}

	return {
		update(uploadedBytes: number, totalBytes: number) {
			if (task.status !== 'running') return
			const now = performance.now()
			samples.push({ time: now, bytes: uploadedBytes })
			while (samples.length > 2 && now - samples[1].time >= RATE_WINDOW_MS) samples.shift()
			const oldest = samples[0]
			const elapsed = now - oldest.time
			task.rate = elapsed >= 1000 ? ((uploadedBytes - oldest.bytes) * 1000) / elapsed : 0
			task.uploadedBytes = uploadedBytes
			task.totalBytes = totalBytes
		},
		succeed() {
			finish('succeeded')
		},
		fail(error: string) {
			task.error = error
			finish('failed')
		},
		setDetails(details: string) {
			task.details = details
		},
	}
}

export function getSharedInstancePublishRate(task: SharedInstancePublishTask) {
	return task.status === 'running' ? task.rate : 0
}
