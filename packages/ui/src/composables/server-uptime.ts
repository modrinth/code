import { useNow } from '@vueuse/core'
import { computed, type Ref } from 'vue'

import { currentSnapshotUptime, useServerContextRuntime } from './server-context-runtime'

export function formatServerUptime(uptime: number) {
	const days = Math.floor(uptime / (24 * 3600))
	const hours = Math.floor((uptime % (24 * 3600)) / 3600)
	const minutes = Math.floor((uptime % 3600) / 60)
	const seconds = uptime % 60

	let formatted = ''
	if (days > 0) formatted += `${days}d `
	if (hours > 0 || days > 0) formatted += `${hours}h `
	formatted += `${minutes}m ${seconds}s`
	return formatted.trim()
}

export function useServerUptime(serverId: Readonly<Ref<string | null>>) {
	const powerSnapshot = useServerContextRuntime(serverId)
	const now = useNow({ interval: 1000 })

	return computed(() => {
		const snapshot = powerSnapshot.value
		const uptime = snapshot ? currentSnapshotUptime(snapshot, now.value.getTime()) : 0
		return uptime > 0 ? uptime : null
	})
}
