export function useFullImageContextMenu() {
	return function onFullImageContextMenu(event: MouseEvent, fullUrl: string | null | undefined) {
		if (!fullUrl) return

		const img = event.currentTarget as HTMLImageElement
		const originalSrc = img.src
		if (originalSrc === fullUrl) return

		img.src = fullUrl

		let reverted = false
		const revert = () => {
			if (reverted) return
			reverted = true
			img.src = originalSrc
			window.removeEventListener('focus', revert)
			window.removeEventListener('pointerdown', revert, true)
			window.removeEventListener('keydown', revert, true)
			window.removeEventListener('pointermove', revert, true)
			window.removeEventListener('wheel', revert, true)
			window.removeEventListener('touchstart', revert, true)
		}

		window.addEventListener('focus', revert)
		window.addEventListener('pointerdown', revert, true)
		window.addEventListener('keydown', revert, true)
		window.addEventListener('pointermove', revert, true)
		window.addEventListener('wheel', revert, true)
		window.addEventListener('touchstart', revert, true)
	}
}
