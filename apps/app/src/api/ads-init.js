document.addEventListener(
  'click',
  function (e) {
    window.top.postMessage({ modrinthAdClick: true }, 'https://modrinth.com')

    let target = e.target
    while (target != null) {
      if (target.matches('a')) {
        e.preventDefault()
        if (target.href) {
          window.top.postMessage({ modrinthOpenUrl: target.href }, 'https://modrinth.com')
        }
        break
      }
      target = target.parentElement
    }
  },
  true,
)

window.open = (url, target, features) => {
  window.top.postMessage({ modrinthOpenUrl: url }, 'https://modrinth.com')
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

  const observer = new MutationObserver(muteVideos)
  observer.observe(document.body, { childList: true, subtree: true })
})
