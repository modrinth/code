import type { IconBackground } from './types'

export function iconBackgroundStyle(background: IconBackground) {
	if (background.type === 'color') return { backgroundColor: background.value }
	return {
		backgroundImage: `linear-gradient(to bottom, ${background.top_color}, ${background.bottom_color})`,
	}
}
