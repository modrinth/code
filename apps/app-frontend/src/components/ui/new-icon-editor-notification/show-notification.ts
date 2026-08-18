const STORAGE_KEY = 'new-icon-editor-notification-shown'

export function shouldShowNewIconEditorNotification(showOnboardingChecklist: boolean): boolean {
	try {
		if (localStorage.getItem(STORAGE_KEY) === 'true') return false
		if (showOnboardingChecklist) return false
		localStorage.setItem(STORAGE_KEY, 'true')
		return true
	} catch {
		return !showOnboardingChecklist
	}
}
