const MODRINTH_ORIGIN = 'https://modrinth.com'

document.addEventListener(
	'click',
	function (e) {
		window.top.postMessage({ modrinthAdClick: true }, MODRINTH_ORIGIN)

		let target = e.target
		if (target?.closest?.('.qc-cmp2-close-icon')) {
			if (modrinthAdsConsentReprompt) {
				setTimeout(finishAdsConsentReprompt)
			} else if (document.documentElement.classList.contains('modrinth-ads-consent-preferences')) {
				setTimeout(() => void restoreAdsConsentNotification())
			}
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

let modrinthAdsConsentOverlayShown = false
let modrinthTcfListenerInstalled = false
let modrinthTcfListenerAttempts = 0
let modrinthAdsConsentReprompt = false
let modrinthAdsConsentRepromptManaging = false
let modrinthAdsConsentActionRequestId = 0
const modrinthAdsConsentActionResolvers = new Map()

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

function findAdsConsentButton(action) {
	const container = document.getElementById('qc-cmp2-container')
	if (!container) return null

	const buttonIds = {
		accept: 'accept-btn',
		reject: 'disagree-btn',
		manage: 'more-options-btn',
	}
	const buttonId = buttonIds[action]
	if (!buttonId) return null

	const button = container.querySelector(`#${buttonId}`)
	return button && !button.disabled ? button : null
}

function clickAdsConsentButtonWhenReady(action, timeoutMs, onButtonFound) {
	const deadline = Date.now() + timeoutMs

	return new Promise((resolve) => {
		function tryClick() {
			const button = findAdsConsentButton(action)
			if (button) {
				// CMP navigation can replace this document during the click, so acknowledge it first.
				onButtonFound?.()
				resolve(true)
				button.click()
			} else if (Date.now() >= deadline) {
				resolve(false)
			} else {
				setTimeout(tryClick, 50)
			}
		}

		tryClick()
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
		clickAdsConsentButtonWhenReady(action, timeoutMs, () => settle(true)).then((clicked) => {
			if (!clicked) settle(false)
		})
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

function finishAdsConsentReprompt() {
	modrinthAdsConsentReprompt = false
	modrinthAdsConsentRepromptManaging = false
	modrinthAdsConsentOverlayShown = false
	document.documentElement.classList.remove('modrinth-ads-consent-overlay')
	concealAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'conceal' })
	invokeAdsConsentOverlayCommand(false)
}

async function openAdsConsentPreferences() {
	prepareAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'prepare' })
	await expandAdsConsentWebview()
	await waitForAdsConsentLayout()
	revealAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'reveal' })

	window.dispatchEvent(new Event('resize'))
	sendAdsConsentCommandToChildFrames({ type: 'resize' })

	const clicked = await performAdsConsentActionAcrossFrames('manage', 2500)
	if (!clicked) {
		await restoreAdsConsentNotification()
	}
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

	await performAdsConsentActionAcrossFrames(action, 1000)
}

window.modrinthAdsConsentAction = (action) => {
	void performAdsConsentAction(action)
}

window.modrinthAdsReopenConsentPreferences = async () => {
	modrinthAdsConsentReprompt = true
	modrinthAdsConsentRepromptManaging = false
	prepareAdsConsentPreferences()
	sendAdsConsentCommandToChildFrames({ type: 'prepare' })

	try {
		await expandAdsConsentWebview()
		await waitForAdsConsentLayout()
		revealAdsConsentPreferences()
		sendAdsConsentCommandToChildFrames({ type: 'reveal' })
		window.dispatchEvent(new Event('resize'))
		sendAdsConsentCommandToChildFrames({ type: 'resize' })

		if (typeof window.__tcfapi === 'function') {
			window.__tcfapi('displayConsentUi', 2, () => {})
		} else {
			finishAdsConsentReprompt()
		}
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
		clickAdsConsentButtonWhenReady(command.action, command.timeoutMs, () => {
			window.parent.postMessage({ modrinthAdsConsentResult: command.requestId }, '*')
		})
	}
})

function setAdsConsentOverlay(shown) {
	if (modrinthAdsConsentOverlayShown === shown) return

	modrinthAdsConsentOverlayShown = shown
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
		if (
			event.origin === MODRINTH_ORIGIN &&
			typeof event.data?.modrinthAdsConsentOverlay === 'boolean'
		) {
			setAdsConsentOverlay(event.data.modrinthAdsConsentOverlay)
		}
	})
}

function handleTcfConsentEvent(tcData, success) {
	if (!success || !tcData) return

	if (tcData.eventStatus === 'cmpuishown') {
		if (modrinthAdsConsentReprompt) {
			if (!modrinthAdsConsentRepromptManaging) {
				modrinthAdsConsentRepromptManaging = true
				window.dispatchEvent(new Event('resize'))
				sendAdsConsentCommandToChildFrames({ type: 'resize' })
				void performAdsConsentActionAcrossFrames('manage', 2500)
			}
			return
		}

		setAdsConsentOverlay(true)
	} else if (tcData.eventStatus === 'useractioncomplete') {
		if (modrinthAdsConsentReprompt) {
			finishAdsConsentReprompt()
			return
		}

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
	installTcfConsentListener()

	const observer = new MutationObserver(muteVideos)
	observer.observe(document.body, { childList: true, subtree: true })
})

installTcfConsentListener()
