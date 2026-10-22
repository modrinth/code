import { moderationSettings } from '@modrinth/moderation'
import { computed } from 'vue'

import { useModerationSettings } from './moderation'

export function useReviewHighlight() {
	const settings = useModerationSettings()
	const color = computed(
		() =>
			settings.value.get(moderationSettings.General.ReviewHighlightColor) || 'var(--color-orange)',
	)
	const opacity = computed(() =>
		settings.value.get(moderationSettings.General.ReviewHighlightOpacity),
	)
	const background = computed(
		() => `color-mix(in srgb, ${color.value} ${opacity.value}%, transparent)`,
	)
	const flashBackground = computed(
		() => `color-mix(in srgb, ${color.value} ${Math.min(100, opacity.value * 2)}%, transparent)`,
	)
	return { color, opacity, background, flashBackground }
}
