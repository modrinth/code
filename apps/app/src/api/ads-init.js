const MODRINTH_ORIGIN = 'https://modrinth.com'

document.addEventListener(
	'click',
	function (e) {
		window.top.postMessage({ modrinthAdClick: true }, MODRINTH_ORIGIN)

		let target = e.target
		const uspCloseIcon = target?.closest?.('.qc-usp-close-icon')
		if (target?.closest?.('.qc-cmp2-close-icon') || uspCloseIcon) {
			if (isAdsConsentReprompt()) {
				setTimeout(finishAdsConsentReprompt)
			} else if (document.documentElement.classList.contains('modrinth-ads-consent-preferences')) {
				setTimeout(() => void restoreAdsConsentNotification())
			}
		}

		if (target?.closest?.('#qc-cmp2-usp .qc-usp-ui-form-content button[mode="primary"]')) {
			beginUspConsentCommit()
		}

		while (target != null) {
			if (target.matches('a')) {
				e.preventDefault()
				if (target.href) {
					window.top.postMessage({ modrinthOpenUrl: target.href }, MODRINTH_ORIGIN)
				}
				break
			}
			target = target.parentElement
		}
	},
	true,
)

window.open = (url, target, features) => {
	window.top.postMessage({ modrinthOpenUrl: url }, MODRINTH_ORIGIN)
}

const modrinthAdsConsentState = {
	phase: 'idle',
	variant: null,
	commitTimeout: null,
}
let modrinthTcfListenerInstalled = false
let modrinthTcfListenerAttempts = 0
let modrinthGppListenerInstalled = false
let modrinthGppListenerAttempts = 0
let modrinthAdsConsentActionRequestId = 0
const modrinthAdsConsentActionResolvers = new Map()

function transitionAdsConsent(event) {
	const phase = modrinthAdsConsentState.phase

	if (event === 'prompt-detected') {
		if (phase === 'idle') modrinthAdsConsentState.phase = 'initial'
	} else if (event === 'reprompt-started') {
		modrinthAdsConsentState.phase = 'reprompt'
	} else if (event === 'commit-started') {
		modrinthAdsConsentState.phase = isAdsConsentReprompt()
			? 'reprompt-committing'
			: 'initial-committing'
	} else if (event === 'commit-timed-out') {
		if (phase === 'reprompt-committing') {
			modrinthAdsConsentState.phase = 'reprompt'
		} else if (phase === 'initial-committing') {
			modrinthAdsConsentState.phase = 'initial'
		}
	} else if (event === 'completed') {
		modrinthAdsConsentState.phase = 'complete'
	}
}

function isAdsConsentReprompt() {
	return (
		modrinthAdsConsentState.phase === 'reprompt' ||
		modrinthAdsConsentState.phase === 'reprompt-committing'
	)
}

function isUspConsentCommitPending() {
	return (
		modrinthAdsConsentState.phase === 'initial-committing' ||
		modrinthAdsConsentState.phase === 'reprompt-committing'
	)
}

function installAdsRailStyle() {
	if (document.getElementById('modrinth-ads-rail-style')) {
		return
	}
	const style = document.createElement('style')
	style.id = 'modrinth-ads-rail-style'
	style.textContent = `
		html.modrinth-ads-consent-preferences #modrinth-rail-1 {
			display: none !important;
		}
	`
	document.documentElement.appendChild(style)
}

function installAdsConsentOverlayStyle() {
	if (document.getElementById('modrinth-ads-consent-overlay-style')) {
		return
	}
	const style = document.createElement('style')
	style.id = 'modrinth-ads-consent-overlay-style'
	style.textContent = `
		html.modrinth-ads-consent-overlay:not(.modrinth-ads-consent-preferences) #qc-cmp2-container,
		html.modrinth-ads-consent-preferences:not(.modrinth-ads-consent-preferences-visible) #qc-cmp2-container {
			display: none !important;
		}

		#qc-cmp2-usp .qc-usp-close-icon {
			display: none !important;
		}
	`
	document.documentElement.appendChild(style)
}

function getTauriInvoke() {
	return window.__TAURI__?.core?.invoke ?? window.__TAURI_INTERNALS__?.invoke
}

function invokeAdsConsentOverlayCommand(shown) {
	const invoke = getTauriInvoke()

	if (typeof invoke !== 'function') {
		return
	}

	const command = shown ? 'show_ads_consent_overlay' : 'hide_ads_consent_overlay'
	const args = shown ? {} : { dpr: window.devicePixelRatio }

	invoke(`plugin:ads|${command}`, args).catch(() => {})
}

function prepareAdsConsentPreferences() {
	installAdsRailStyle()
	installAdsConsentOverlayStyle()
	document.documentElement.classList.add('modrinth-ads-consent-preferences')
	document.documentElement.classList.remove('modrinth-ads-consent-preferences-visible')
}

function revealAdsConsentPreferences() {
	document.documentElement.classList.add('modrinth-ads-consent-preferences-visible')
}

function concealAdsConsentPreferences() {
	document.documentElement.classList.remove('modrinth-ads-consent-preferences')
	document.documentElement.classList.remove('modrinth-ads-consent-preferences-visible')
	installAdsConsentOverlayStyle()
}

async function expandAdsConsentWebview() {
	const invoke = getTauriInvoke()
	if (typeof invoke !== 'function') {
		throw new Error('Tauri invoke is unavailable in the ads webview')
	}

	await invoke('plugin:ads|show_ads_consent_preferences')
}

function sendAdsConsentCommandToChildFrames(command) {
	document.querySelectorAll('iframe').forEach((frame) => {
		frame.contentWindow?.postMessage({ modrinthAdsConsentCommand: command }, '*')
	})
}

function isDirectChildFrame(source) {
	return Array.from(document.querySelectorAll('iframe')).some(
		(frame) => frame.contentWindow === source,
	)
}

function displayUspConsentUi() {
	if (typeof window.__uspapi !== 'function') return false

	try {
		window.__uspapi('displayUspUi', 1, () => {})
		return true
	} catch {
		return false
	}
}

function detectAdsConsentVariant() {
	let variant = null

	if (document.getElementById('qc-cmp2-usp')) {
		variant = 'usp'
	} else if (document.getElementById('qc-cmp2-ui')) {
		variant = 'tcf'
	}

	if (variant) {
		modrinthAdsConsentState.variant = variant
	}

	return variant
}

function findTcfConsentButton(action) {
	const dialog = document.getElementById('qc-cmp2-ui')
	if (!dialog) return null

	const buttonIds = {
		accept: 'accept-btn',
		reject: 'disagree-btn',
		manage: 'more-options-btn',
	}
	const buttonId = buttonIds[action]
	if (!buttonId) return null

	const button = dialog.querySelector(`#${buttonId}`)
	return button && !button.disabled ? button : null
}

function getUspConsentControls() {
	const dialog = document.getElementById('qc-cmp2-usp')
	if (!dialog) return null

	const toggles = Array.from(
		dialog.querySelectorAll('.qc-usp-container button.qc-cmp2-toggle[role="switch"]'),
	)
	const confirmButton = dialog.querySelector('.qc-usp-ui-form-content button[mode="primary"]')

	if (
		toggles.length === 0 ||
		!confirmButton ||
		confirmButton.disabled ||
		toggles.some(
			(toggle) =>
				toggle.disabled || !['true', 'false'].includes(toggle.getAttribute('aria-checked')),
		)
	) {
		return null
	}

	return { toggles, confirmButton }
}

function waitForUspToggleState(index, checked, expectedCount, deadline) {
	return new Promise((resolve) => {
		function checkState() {
			const controls = getUspConsentControls()
			if (
				controls &&
				controls.toggles.length === expectedCount &&
				controls.toggles[index]?.getAttribute('aria-checked') === String(checked)
			) {
				resolve(controls)
			} else if (Date.now() >= deadline) {
				resolve(null)
			} else {
				setTimeout(checkState, 50)
			}
		}

		checkState()
	})
}

async function setUspToggleStates(checked, controls, timeoutMs) {
	const expectedCount = controls.toggles.length
	const deadline = Date.now() + timeoutMs

	for (let index = 0; index < expectedCount; index += 1) {
		controls = getUspConsentControls()
		if (!controls || controls.toggles.length !== expectedCount) return null

		const toggle = controls.toggles[index]
		if (toggle.getAttribute('aria-checked') !== String(checked)) {
			toggle.click()
			controls = await waitForUspToggleState(index, checked, expectedCount, deadline)
			if (!controls) return null
		}
	}

	return controls
}

async function performAdsConsentActionInDocument(action, onHandled) {
	const variant = detectAdsConsentVariant()
	const unknownDialog = document.querySelector('#qc-cmp2-container [role="dialog"]')

	if (action === 'show') {
		if (variant || unknownDialog) {
			onHandled?.()
			return 'handled'
		}
		return 'not-ready'
	}

	if (variant === 'usp') {
		if (action === 'manage') {
			onHandled?.()
			return 'handled'
		}

		if (!['accept', 'reject'].includes(action)) return 'failed'

		const controls = getUspConsentControls()
		if (!controls) return 'not-ready'

		const shouldOptOut = action === 'reject'
		const settledControls = await setUspToggleStates(shouldOptOut, controls, 2000)
		if (!settledControls) return 'failed'

		// CMP navigation can replace this document during the click, so acknowledge it first.
		onHandled?.()
		beginUspConsentCommit()
		settledControls.confirmButton.click()
		return 'handled'
	}

	if (variant === 'tcf') {
		const button = findTcfConsentButton(action)
		if (!button) return 'not-ready'

		// CMP navigation can replace this document during the click, so acknowledge it first.
		onHandled?.()
		button.click()
		return 'handled'
	}

	if (action === 'manage' && unknownDialog) {
		onHandled?.()
		return 'handled'
	}

	return 'not-ready'
}

function performAdsConsentActionWhenReady(action, timeoutMs, onHandled) {
	const deadline = Date.now() + timeoutMs

	return new Promise((resolve) => {
		async function tryAction() {
			const result = await performAdsConsentActionInDocument(action, onHandled)
			if (result === 'handled') {
				resolve(true)
			} else if (result === 'failed' || Date.now() >= deadline) {
				resolve(false)
			} else {
				setTimeout(tryAction, 50)
			}
		}

		void tryAction()
	})
}

function performAdsConsentActionAcrossFrames(action, timeoutMs) {
	const requestId = `${Date.now()}-${++modrinthAdsConsentActionRequestId}`

	return new Promise((resolve) => {
		let settled = false
		const settle = (clicked) => {
			if (settled) return
			settled = true
			clearTimeout(timeout)
			modrinthAdsConsentActionResolvers.delete(requestId)
			resolve(clicked)
		}

		const timeout = setTimeout(() => settle(false), timeoutMs)
		modrinthAdsConsentActionResolvers.set(requestId, () => settle(true))
		sendAdsConsentCommandToChildFrames({ type: 'perform', action, requestId, timeoutMs })
		void performAdsConsentActionWhenReady(action, timeoutMs, () => settle(true))
	})
}

function waitForAdsConsentLayout() {
	return new Promise((resolve) => setTimeout(resolve, 100))
}

async function restoreAdsConsentNotification() {
	concealAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'conceal' })

	const invoke = getTauriInvoke()
	if (typeof invoke === 'function') {
		await invoke('plugin:ads|hide_ads_consent_preferences')
	}
}

async function showNativeAdsConsentUi() {
	prepareAdsConsentPreferences()
	await waitForAdsConsentLayout()
	sendAdsConsentCommandToChildFrames({ type: 'prepare' })
	await expandAdsConsentWebview()
	await waitForAdsConsentLayout()
	revealAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'reveal' })

	window.dispatchEvent(new Event('resize'))
	sendAdsConsentCommandToChildFrames({ type: 'resize' })

	const shown = await performAdsConsentActionAcrossFrames('show', 2500)
	if (!shown) {
		await restoreAdsConsentNotification()
	}

	return shown
}

function finishAdsConsentReprompt() {
	transitionAdsConsent('completed')
	clearTimeout(modrinthAdsConsentState.commitTimeout)
	modrinthAdsConsentState.commitTimeout = null
	document.documentElement.classList.remove('modrinth-ads-consent-overlay')
	concealAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'conceal' })
	invokeAdsConsentOverlayCommand(false)
}

async function openAdsConsentPreferences() {
	if (!(await showNativeAdsConsentUi())) return

	await performAdsConsentActionAcrossFrames('manage', 2500)
}

async function performAdsConsentAction(action) {
	if (!['accept', 'reject', 'manage'].includes(action)) return

	if (action === 'manage') {
		try {
			await openAdsConsentPreferences()
		} catch {
			await restoreAdsConsentNotification()
		}
		return
	}

	const handled = await performAdsConsentActionAcrossFrames(action, 2500)
	if (!handled) {
		try {
			await showNativeAdsConsentUi()
		} catch {
			await restoreAdsConsentNotification()
		}
	}
}

window.modrinthAdsConsentAction = (action) => {
	void performAdsConsentAction(action)
}

function isUspConsentApplicable() {
	if (detectAdsConsentVariant() === 'usp') return Promise.resolve(true)
	if (typeof window.__uspapi !== 'function') return Promise.resolve(false)

	return new Promise((resolve) => {
		let settled = false
		const settle = (applicable) => {
			if (settled) return
			settled = true
			clearTimeout(timeout)
			resolve(applicable)
		}
		const timeout = setTimeout(() => settle(false), 500)

		try {
			window.__uspapi('uspPing', 1, (data, success) => {
				if (!success || !data) {
					settle(false)
					return
				}

				const modes = Array.isArray(data.mode) ? data.mode : [data.mode]
				const jurisdictions = Array.isArray(data.jurisdiction)
					? data.jurisdiction
					: [data.jurisdiction]
				const location = String(data.location ?? '').toUpperCase()
				const hasUspMode = modes.some((mode) =>
					String(mode ?? '')
						.toUpperCase()
						.includes('USP'),
				)
				const locationApplies =
					!location ||
					jurisdictions.some((jurisdiction) =>
						String(jurisdiction ?? '')
							.toUpperCase()
							.includes(location),
					)

				settle(hasUspMode && locationApplies)
			})
		} catch {
			settle(false)
		}
	})
}

async function displayAdsConsentReprompt() {
	if (
		(modrinthAdsConsentState.variant === 'usp' || (await isUspConsentApplicable())) &&
		typeof window.__uspapi === 'function'
	) {
		modrinthAdsConsentState.variant = 'usp'
		return displayUspConsentUi()
	}

	if (typeof window.__tcfapi === 'function') {
		modrinthAdsConsentState.variant = 'tcf'
		window.__tcfapi('displayConsentUi', 2, () => {})
		return true
	}

	return false
}

window.modrinthAdsReopenConsentPreferences = async () => {
	if (document.documentElement.classList.contains('modrinth-ads-consent-overlay')) {
		try {
			await openAdsConsentPreferences()
		} catch {
			await restoreAdsConsentNotification()
		}
		return
	}

	transitionAdsConsent('reprompt-started')
	prepareAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'prepare' })

	try {
		await expandAdsConsentWebview()
		await waitForAdsConsentLayout()
		revealAdsConsentPreferences()
		sendAdsConsentCommandToChildFrames({ type: 'reveal' })
		window.dispatchEvent(new Event('resize'))
		sendAdsConsentCommandToChildFrames({ type: 'resize' })

		if (!(await displayAdsConsentReprompt())) {
			finishAdsConsentReprompt()
			return
		}

		if (!(await performAdsConsentActionAcrossFrames('show', 2500))) {
			finishAdsConsentReprompt()
			return
		}

		await performAdsConsentActionAcrossFrames('manage', 2500)
	} catch {
		finishAdsConsentReprompt()
	}
}

window.addEventListener('message', (event) => {
	const resultRequestId = event.data?.modrinthAdsConsentResult
	if (typeof resultRequestId === 'string' && isDirectChildFrame(event.source)) {
		if (window.top === window) {
			modrinthAdsConsentActionResolvers.get(resultRequestId)?.()
		} else {
			window.parent.postMessage({ modrinthAdsConsentResult: resultRequestId }, '*')
		}
		return
	}

	if (window.top === window || event.source !== window.parent) return

	const command = event.data?.modrinthAdsConsentCommand
	if (!command || typeof command !== 'object') return

	if (command.type === 'prepare') {
		prepareAdsConsentPreferences()
		sendAdsConsentCommandToChildFrames(command)
	} else if (command.type === 'reveal') {
		revealAdsConsentPreferences()
		sendAdsConsentCommandToChildFrames(command)
	} else if (command.type === 'conceal') {
		concealAdsConsentPreferences()
		sendAdsConsentCommandToChildFrames(command)
	} else if (command.type === 'resize') {
		window.dispatchEvent(new Event('resize'))
		sendAdsConsentCommandToChildFrames(command)
	} else if (
		command.type === 'perform' &&
		typeof command.action === 'string' &&
		typeof command.requestId === 'string' &&
		typeof command.timeoutMs === 'number'
	) {
		sendAdsConsentCommandToChildFrames(command)
		performAdsConsentActionWhenReady(command.action, command.timeoutMs, () => {
			window.parent.postMessage({ modrinthAdsConsentResult: command.requestId }, '*')
		})
	}
})

function setAdsConsentOverlay(shown) {
	if (document.documentElement.classList.contains('modrinth-ads-consent-overlay') === shown) return

	installAdsConsentOverlayStyle()
	document.documentElement.classList.toggle('modrinth-ads-consent-overlay', shown)
	if (!shown) {
		document.documentElement.classList.remove('modrinth-ads-consent-preferences')
		document.documentElement.classList.remove('modrinth-ads-consent-preferences-visible')
	}

	if (window.top === window) {
		invokeAdsConsentOverlayCommand(shown)
	} else {
		window.top.postMessage({ modrinthAdsConsentOverlay: shown }, MODRINTH_ORIGIN)
	}
}

if (window.top === window) {
	window.addEventListener('message', (event) => {
		if (event.origin !== MODRINTH_ORIGIN) return

		if (typeof event.data?.modrinthAdsConsentOverlay === 'boolean') {
			setAdsConsentOverlay(event.data.modrinthAdsConsentOverlay)
		}
	})
}

function finishUspConsentCommit() {
	if (!isUspConsentCommitPending()) return

	const wasReprompt = isAdsConsentReprompt()
	transitionAdsConsent('completed')
	clearTimeout(modrinthAdsConsentState.commitTimeout)
	modrinthAdsConsentState.commitTimeout = null

	if (wasReprompt) {
		finishAdsConsentReprompt()
	} else {
		setAdsConsentOverlay(false)
	}
}

function beginUspConsentCommit() {
	if (!document.getElementById('qc-cmp2-usp')) return

	modrinthAdsConsentState.variant = 'usp'
	transitionAdsConsent('commit-started')
	clearTimeout(modrinthAdsConsentState.commitTimeout)

	const deadline = Date.now() + 2500
	function checkForDialogClosure() {
		if (!isUspConsentCommitPending()) return

		if (!document.getElementById('qc-cmp2-usp')) {
			finishUspConsentCommit()
		} else if (Date.now() >= deadline) {
			transitionAdsConsent('commit-timed-out')
			modrinthAdsConsentState.commitTimeout = null
		} else {
			modrinthAdsConsentState.commitTimeout = setTimeout(checkForDialogClosure, 50)
		}
	}

	modrinthAdsConsentState.commitTimeout = setTimeout(checkForDialogClosure, 50)
}

function syncAdsConsentUi() {
	const variant = detectAdsConsentVariant()

	if (variant && !isAdsConsentReprompt() && modrinthAdsConsentState.phase !== 'complete') {
		transitionAdsConsent('prompt-detected')
		setAdsConsentOverlay(true)
	}

	if (isUspConsentCommitPending() && !document.getElementById('qc-cmp2-usp')) {
		finishUspConsentCommit()
	}
}

function handleGppConsentEvent(gppData, success) {
	if (
		success &&
		gppData?.eventName === 'sectionChange' &&
		modrinthAdsConsentState.variant === 'usp' &&
		isUspConsentCommitPending()
	) {
		finishUspConsentCommit()
	}
}

function installGppConsentListener() {
	if (modrinthGppListenerInstalled) return

	if (typeof window.__gpp === 'function') {
		modrinthGppListenerInstalled = true
		window.__gpp('addEventListener', handleGppConsentEvent)
		return
	}

	if (modrinthGppListenerAttempts < 60) {
		modrinthGppListenerAttempts += 1
		setTimeout(installGppConsentListener, 500)
	}
}

function handleTcfConsentEvent(tcData, success) {
	if (!success || !tcData) return
	detectAdsConsentVariant()

	if (tcData.eventStatus === 'cmpuishown') {
		if (isAdsConsentReprompt()) return

		transitionAdsConsent('prompt-detected')
		setAdsConsentOverlay(true)
	} else if (
		tcData.eventStatus === 'useractioncomplete' &&
		modrinthAdsConsentState.variant === 'tcf'
	) {
		if (isAdsConsentReprompt()) {
			finishAdsConsentReprompt()
			return
		}

		transitionAdsConsent('completed')
		setAdsConsentOverlay(false)
	}
}

// polling to install listener on tcf api
function installTcfConsentListener() {
	if (modrinthTcfListenerInstalled) return

	if (typeof window.__tcfapi === 'function') {
		modrinthTcfListenerInstalled = true
		window.__tcfapi('addEventListener', 2, handleTcfConsentEvent)
		return
	}

	if (modrinthTcfListenerAttempts < 60) {
		modrinthTcfListenerAttempts += 1
		setTimeout(installTcfConsentListener, 500)
	}
}

function muteAudioContext() {
	if (window.AudioContext || window.webkitAudioContext) {
		const AudioContext = window.AudioContext || window.webkitAudioContext
		const originalCreateMediaElementSource = AudioContext.prototype.createMediaElementSource
		const originalCreateMediaStreamSource = AudioContext.prototype.createMediaStreamSource
		const originalCreateMediaStreamTrackSource = AudioContext.prototype.createMediaStreamTrackSource
		const originalCreateBufferSource = AudioContext.prototype.createBufferSource
		const originalCreateOscillator = AudioContext.prototype.createOscillator

		AudioContext.prototype.createGain = function () {
			const gain = originalCreateGain.call(this)
			gain.gain.value = 0
			return gain
		}

		AudioContext.prototype.createMediaElementSource = function (mediaElement) {
			const source = originalCreateMediaElementSource.call(this, mediaElement)
			source.connect(this.createGain())
			return source
		}

		AudioContext.prototype.createMediaStreamSource = function (mediaStream) {
			const source = originalCreateMediaStreamSource.call(this, mediaStream)
			source.connect(this.createGain())
			return source
		}

		AudioContext.prototype.createMediaStreamTrackSource = function (mediaStreamTrack) {
			const source = originalCreateMediaStreamTrackSource.call(this, mediaStreamTrack)
			source.connect(this.createGain())
			return source
		}

		AudioContext.prototype.createBufferSource = function () {
			const source = originalCreateBufferSource.call(this)
			source.connect(this.createGain())
			return source
		}

		AudioContext.prototype.createOscillator = function () {
			const oscillator = originalCreateOscillator.call(this)
			oscillator.connect(this.createGain())
			return oscillator
		}
	}
}

function muteVideo(mediaElement) {
	let count = Number(mediaElement.getAttribute('data-modrinth-muted-count') ?? 0)

	if (!mediaElement.muted || mediaElement.volume !== 0) {
		mediaElement.muted = true
		mediaElement.volume = 0

		mediaElement.setAttribute('data-modrinth-muted-count', count + 1)
	}

	if (count > 5) {
		// Video is detected as malicious, so it is removed from the page
		mediaElement.remove()
	}
}

function muteVideos() {
	document.querySelectorAll('video, audio').forEach(function (mediaElement) {
		muteVideo(mediaElement)

		if (!mediaElement.hasAttribute('data-modrinth-muted')) {
			mediaElement.addEventListener('volumechange', () => muteVideo(mediaElement))

			mediaElement.setAttribute('data-modrinth-muted', 'true')
		}
	})
}

document.addEventListener('DOMContentLoaded', () => {
	installAdsRailStyle()
	installAdsConsentOverlayStyle()
	muteVideos()
	muteAudioContext()
	syncAdsConsentUi()
	installTcfConsentListener()
	installGppConsentListener()

	const observer = new MutationObserver(() => {
		muteVideos()
		syncAdsConsentUi()
	})
	observer.observe(document.body, { childList: true, subtree: true })
})

syncAdsConsentUi()
installTcfConsentListener()
installGppConsentListener()
