const STORAGE_KEY = 'sync-instances-update-notification-shown'
let shown = false

export function markSyncInstancesUpdateNotificationShown(): boolean {
	shown = true
	try {
		localStorage.setItem(STORAGE_KEY, 'true')
		return true
	} catch {
		return false
	}
}

export function shouldShowSyncInstancesUpdateNotification(): boolean {
	if (shown) return false
	try {
		if (localStorage.getItem(STORAGE_KEY) === 'true') return false
	} catch {
		return false
	}
	return markSyncInstancesUpdateNotificationShown()
}
