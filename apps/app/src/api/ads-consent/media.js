function muteAudioContext() {
	const AudioContextClass = window.AudioContext ?? window.webkitAudioContext
	if (!AudioContextClass) return

	const prototype = AudioContextClass.prototype
	const originalCreateGain = prototype.createGain
	const originalCreateMediaElementSource = prototype.createMediaElementSource
	const originalCreateMediaStreamSource = prototype.createMediaStreamSource
	const originalCreateMediaStreamTrackSource = prototype.createMediaStreamTrackSource
	const originalCreateBufferSource = prototype.createBufferSource
	const originalCreateOscillator = prototype.createOscillator

	prototype.createGain = function () {
		const gain = originalCreateGain.call(this)
		gain.gain.value = 0
		return gain
	}

	prototype.createMediaElementSource = function (mediaElement) {
		const source = originalCreateMediaElementSource.call(this, mediaElement)
		source.connect(this.createGain())
		return source
	}

	prototype.createMediaStreamSource = function (mediaStream) {
		const source = originalCreateMediaStreamSource.call(this, mediaStream)
		source.connect(this.createGain())
		return source
	}

	if (originalCreateMediaStreamTrackSource) {
		prototype.createMediaStreamTrackSource = function (mediaStreamTrack) {
			const source = originalCreateMediaStreamTrackSource.call(this, mediaStreamTrack)
			source.connect(this.createGain())
			return source
		}
	}

	prototype.createBufferSource = function () {
		const source = originalCreateBufferSource.call(this)
		source.connect(this.createGain())
		return source
	}

	prototype.createOscillator = function () {
		const oscillator = originalCreateOscillator.call(this)
		oscillator.connect(this.createGain())
		return oscillator
	}
}

function muteMediaElement(mediaElement) {
	const muteCount = Number(mediaElement.dataset.modrinthMutedCount ?? 0)

	if (!mediaElement.muted || mediaElement.volume !== 0) {
		mediaElement.muted = true
		mediaElement.volume = 0
		mediaElement.dataset.modrinthMutedCount = String(muteCount + 1)
	}

	if (muteCount > 5) mediaElement.remove()
}

function muteMediaElements() {
	document.querySelectorAll('video, audio').forEach((mediaElement) => {
		muteMediaElement(mediaElement)

		if (!mediaElement.dataset.modrinthMuted) {
			mediaElement.addEventListener('volumechange', () => muteMediaElement(mediaElement))
			mediaElement.dataset.modrinthMuted = 'true'
		}
	})
}
