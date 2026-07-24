const controller = new AdsConsentController()

function isTopFrame() {
	return window.top === window
}

function initializeTopFrame() {
	if (!isTopFrame()) return

	installConsentStyles()
	controller.syncConsentPopup()
}

document.addEventListener(
	'click',
	(event) => {
		notifyAdClick()

		const target = event.target instanceof Element ? event.target : null
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
