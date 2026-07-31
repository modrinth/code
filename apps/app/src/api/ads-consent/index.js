const controller = new AdsConsentController()
const CONSENT_LISTENER_RETRY_INTERVAL = 250
const CONSENT_LISTENER_MAX_ATTEMPTS = 60

let tcfListenerInstalled = false
let gppListenerInstalled = false
let consentListenerInstallAttempts = 0
/** @type {ReturnType<typeof setTimeout> | null} */
let consentListenerRetry = null

function isTopFrame() {
	return window.top === window
}

function installConsentListeners() {
	if (!tcfListenerInstalled && typeof window.__tcfapi === 'function') {
		try {
			window.__tcfapi('addEventListener', 2, (tcData, success) =>
				controller.handleTcfConsentEvent(tcData, success),
			)
			tcfListenerInstalled = true
		} catch {}
	}

	if (!gppListenerInstalled && typeof window.__gpp === 'function') {
		try {
			window.__gpp('addEventListener', (gppData, success) =>
				controller.handleGppConsentEvent(gppData, success),
			)
			gppListenerInstalled = true
		} catch {}
	}

	if (
		(tcfListenerInstalled && gppListenerInstalled) ||
		consentListenerInstallAttempts >= CONSENT_LISTENER_MAX_ATTEMPTS ||
		consentListenerRetry
	) {
		return
	}

	consentListenerInstallAttempts += 1
	consentListenerRetry = setTimeout(() => {
		consentListenerRetry = null
		installConsentListeners()
	}, CONSENT_LISTENER_RETRY_INTERVAL)
}

function initializeTopFrame() {
	if (!isTopFrame()) return

	installConsentStyles()
	installConsentListeners()
	controller.syncConsentPopup()
}

document.addEventListener(
	'click',
	(event) => {
		notifyAdClick()

		const target = event.target instanceof Element ? event.target : null
		if (target?.closest('.qc-cmp2-close-icon')) {
			setTimeout(() => controller.handleTcfClose())
		}

		if (target?.closest('#qc-cmp2-usp .qc-usp-ui-form-content button[mode="primary"]')) {
			controller.beginConsentSubmission()
		}

		const link = target?.closest('a')
		if (link?.href) {
			event.preventDefault()
			openExternalUrl(link.href)
		}
	},
	true,
)

window.open = (url) => {
	if (url) openExternalUrl(url)
	return null
}

window.modrinthPrivacy ??= {}
window.modrinthPrivacy.adsConsentAction = (action) => void controller.performAction(action)
window.modrinthPrivacy.adsReopenConsentPreferences = () => controller.reopenPreferences()

document.addEventListener('DOMContentLoaded', () => {
	muteMediaElements()
	muteAudioContext()
	initializeTopFrame()

	const observer = new MutationObserver(() => {
		muteMediaElements()
		if (isTopFrame()) controller.syncConsentPopup()
	})
	observer.observe(document.body, { childList: true, subtree: true })
})

initializeTopFrame()
