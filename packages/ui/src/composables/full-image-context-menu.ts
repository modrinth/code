export function useFullImageContextMenu() {
	return function onFullImageContextMenu(event: MouseEvent, fullUrl: string | null | undefined) {
		if (!fullUrl) return

		const img = event.currentTarget as HTMLImageElement
		const originalSrc = img.src
		if (originalSrc === fullUrl) return

		img.src = fullUrl
		window.addEventListener(
			'focus',
			() => {
				img.src = originalSrc
			},
			{ once: true },
		)
	}
}
