const STORAGE_KEY = 'new-icon-editor-notification-shown'

export function shouldShowNewIconEditorNotification(): boolean {
	try {
		if (localStorage.getItem(STORAGE_KEY) === 'true') return false
		localStorage.setItem(STORAGE_KEY, 'true')
		return true
	} catch {
		return true
	}
}
