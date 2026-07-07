const MODRINTH_ORIGIN = 'https://modrinth.com'

document.addEventListener(
	'click',
	function (e) {
		window.top.postMessage({ modrinthAdClick: true }, MODRINTH_ORIGIN)

		let target = e.target
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

function installAdsConsentOverlayStyle() {
	if (document.getElementById('modrinth-ads-consent-overlay-style')) {
		return
	}
	const style = document.createElement('style')
	style.id = 'modrinth-ads-consent-overlay-style'
	style.textContent = `
		html.modrinth-ads-consent-overlay #modrinth-rail-1 {
			visibility: hidden !important;
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

function setAdsConsentOverlay(shown) {
	if (modrinthAdsConsentOverlayShown === shown) return

	modrinthAdsConsentOverlayShown = shown
	installAdsConsentOverlayStyle()
	document.documentElement.classList.toggle('modrinth-ads-consent-overlay', shown)

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
		setAdsConsentOverlay(true)
	} else if (tcData.eventStatus === 'useractioncomplete' || tcData.eventStatus === 'tcloaded') {
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
	muteVideos()
	muteAudioContext()
	installTcfConsentListener()

	const observer = new MutationObserver(muteVideos)
	observer.observe(document.body, { childList: true, subtree: true })
})

installTcfConsentListener()
