import { Channel, invoke } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'

export interface StoreVerification {
	checked: number
	repaired: number
	issues: { sha512: string; message: string }[]
}

export const storeVerificationReport = ref<StoreVerification | null>(null)

export const storeVerificationTask = ref<{
	id: string
	status: 'running' | 'succeeded' | 'failed'
	current: number
	total: number
	rate: number
	lastRead: number
} | null>(null)

export const verifyingStore = computed(() => storeVerificationTask.value?.status === 'running')

export async function verifyStore(): Promise<StoreVerification> {
	if (verifyingStore.value) throw new Error('Content verification is already running')
	const task = {
		id: `store-verification-${crypto.randomUUID()}`,
		status: 'running' as const,
		current: 0,
		total: 0,
		rate: 0,
		lastRead: performance.now(),
	}
	storeVerificationTask.value = task
	storeVerificationReport.value = null
	let previousBytes = 0
	let previousTime = performance.now()
	const onProgress = new Channel<[number, number]>()
	onProgress.onmessage = ([current, total]) => {
		const active = storeVerificationTask.value
		if (!active || active.id !== task.id || active.status !== 'running') return
		const now = performance.now()
		active.current = current
		active.total = total
		if (current > previousBytes) {
			active.rate = ((current - previousBytes) * 1000) / Math.max(1, now - previousTime)
			active.lastRead = now
		}
		previousBytes = current
		previousTime = now
	}
	try {
		const report = await invoke<StoreVerification>('plugin:settings|store_verify', {
			repair: true,
			onProgress,
		})
		storeVerificationReport.value = report
		storeVerificationTask.value!.status = report.issues.length ? 'failed' : 'succeeded'
		return report
	} catch (error) {
		storeVerificationTask.value!.status = 'failed'
		throw error
	} finally {
		storeVerificationTask.value!.rate = 0
	}
}
