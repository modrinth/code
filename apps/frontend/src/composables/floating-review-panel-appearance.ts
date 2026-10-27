import { moderationSettings } from '@modrinth/moderation'
import { computed } from 'vue'

import { useModerationSettings } from './moderation'

export function useFloatingReviewPanelAppearance() {
	const settings = useModerationSettings()
	const general = moderationSettings.General
	const style = computed(() => {
		const background =
			settings.value.get(general.FloatingReviewPanelBackground) || 'var(--color-orange)'
		const opacity = settings.value.get(general.FloatingReviewPanelBackgroundOpacity)
		const tint = `color-mix(in srgb, ${background} ${opacity}%, transparent)`
		return {
			borderColor:
				settings.value.get(general.FloatingReviewPanelBorderColor) ||
				'var(--color-orange-highlight)',
			borderWidth: `${settings.value.get(general.FloatingReviewPanelBorderWidth)}px`,
			backgroundColor: 'var(--surface-1)',
			backgroundImage: `linear-gradient(${tint}, ${tint})`,
		}
	})
	return { style }
}
