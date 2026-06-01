// Injected into the in-app YouTube watch-page webview so it shows only the
// video player filling the popup, with no YouTube chrome (masthead, sidebar,
// comments). Stylesheet rules use `!important` so they override the inline
// sizing YouTube's player JS continuously re-applies to the <video> element.
;(function () {
	const css = `
		html, body {
			margin: 0 !important;
			overflow: hidden !important;
			background: #000 !important;
		}

		/* Hide all YouTube chrome around the player. */
		#masthead-container,
		ytd-masthead,
		#secondary,
		#secondary-inner,
		#below,
		ytd-comments,
		#comments,
		#chat,
		ytd-watch-metadata,
		#related,
		tp-yt-app-drawer {
			display: none !important;
		}

		/* Expand the player to fill the webview (which is the popup area). */
		ytd-watch-flexy #player,
		#player-container,
		#player-container-inner,
		#movie_player,
		.html5-video-player {
			position: fixed !important;
			inset: 0 !important;
			width: 100vw !important;
			height: 100vh !important;
			max-width: none !important;
			max-height: none !important;
			margin: 0 !important;
			padding: 0 !important;
			z-index: 2147483646 !important;
		}

		video.html5-main-video {
			position: fixed !important;
			inset: 0 !important;
			width: 100vw !important;
			height: 100vh !important;
			object-fit: contain !important;
		}
	`

	function inject() {
		if (document.getElementById('__mr_theater_style')) {
			return
		}
		const style = document.createElement('style')
		style.id = '__mr_theater_style'
		style.textContent = css
		;(document.head || document.documentElement).appendChild(style)
	}

	inject()
	document.addEventListener('DOMContentLoaded', inject)

	// YouTube lays out player controls on resize; nudge it after load so the
	// control bar matches the resized player, and try to start playback.
	function nudge() {
		window.dispatchEvent(new Event('resize'))
		const player = document.getElementById('movie_player')
		if (player && typeof player.playVideo === 'function') {
			try {
				player.playVideo()
			} catch (e) {
				/* ignore */
			}
		}
	}

	window.addEventListener('load', function () {
		setTimeout(nudge, 400)
		setTimeout(nudge, 1200)
	})
})()
