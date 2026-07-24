const MODRINTH_ORIGIN = 'https://modrinth.com'

/**
 * @typedef {'custom' | 'fallback' | 'hidden'} AdsConsentPopupMode
 *
 * @typedef {(command: string, args?: Record<string, unknown>) => Promise<unknown>} TauriInvoke
 */

/** @returns {TauriInvoke | undefined} */
function getTauriInvoke() {
	const invoke = window.__TAURI__?.core?.invoke ?? window.__TAURI_INTERNALS__?.invoke
	return typeof invoke === 'function' ? invoke : undefined
}

/** @returns {void} */
function notifyAdClick() {
	window.top?.postMessage({ modrinthAdClick: true }, MODRINTH_ORIGIN)
}

/**
 * @param {string | URL} url
 * @returns {void}
 */
function openExternalUrl(url) {
	window.top?.postMessage({ modrinthOpenUrl: String(url) }, MODRINTH_ORIGIN)
}

/**
 * @param {AdsConsentPopupMode} mode
 * @returns {Promise<void>}
 */
async function invokeAdsConsentPopupMode(mode) {
	const invoke = getTauriInvoke()
	if (!invoke) return

	try {
		if (mode === 'hidden') {
			await invoke('plugin:ads|finish_ads_consent_flow', { dpr: window.devicePixelRatio })
			return
		}

		await invoke('plugin:ads|show_ads_consent_ui', {
			notificationEnabled: mode === 'custom',
		})
	} catch {}
}

/** @returns {Promise<void>} */
async function expandAdsConsentWebview() {
	const invoke = getTauriInvoke()
	if (!invoke) throw new Error('Tauri invoke is unavailable in the ads webview')

	await invoke('plugin:ads|expand_ads_consent_webview')
}
