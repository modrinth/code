const STORAGE_KEY = 'moderation.project-review.slicer-bridge.v1'

/**
 * Version of `bridge.js`. Bump whenever the script changes so reviewers with an older copy
 * installed in slicer are prompted to import the new one.
 */
export const SLICER_BRIDGE_VERSION = 7

interface SavedSlicerBridge {
	version: number
}

/** Returns the bridge version slicer last confirmed it has installed, if any. */
export function readInstalledBridgeVersion(): number | undefined {
	try {
		const saved: SavedSlicerBridge | null = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null')
		return typeof saved?.version === 'number' && Number.isFinite(saved.version)
			? saved.version
			: undefined
	} catch {
		return undefined
	}
}

export function saveInstalledBridgeVersion(version: number) {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify({ version } satisfies SavedSlicerBridge))
	} catch {
		// no-op
	}
}

export function forgetInstalledBridge() {
	try {
		localStorage.removeItem(STORAGE_KEY)
	} catch {
		// no-op
	}
}
