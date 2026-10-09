const STORAGE_KEY = 'server-sharing-update-notification-shown'
let shown = false

export function markServerSharingUpdateNotificationShown(): boolean {
	shown = true
	try {
		localStorage.setItem(STORAGE_KEY, 'true')
		return true
	} catch {
		return false
	}
}

export function shouldShowServerSharingUpdateNotification(): boolean {
	if (shown) return false
	try {
		return localStorage.getItem(STORAGE_KEY) !== 'true'
	} catch {
		return false
	}
}
