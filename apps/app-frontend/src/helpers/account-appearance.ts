const STORAGE_KEY = 'modrinth-account-appearances'

function readCache(): Record<string, { auto: boolean; theme: string }> {
	try {
		const parsed = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? '')
		if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return {}
		return parsed
	} catch {
		return {}
	}
}

export function rememberAccountAppearance(
	userId: string,
	appearance: { auto: boolean; theme: string },
) {
	try {
		window.localStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({ ...readCache(), [userId]: appearance }),
		)
	} catch {
		// storage blocked or full
	}
}

export function getAccountAppearance(userId: string) {
	return readCache()[userId]
}
