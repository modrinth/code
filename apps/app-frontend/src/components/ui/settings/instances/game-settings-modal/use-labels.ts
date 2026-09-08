import { useVIntl } from '@modrinth/ui'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { computed, type MaybeRefOrGetter, onScopeDispose, shallowRef, toValue, watch } from 'vue'

import {
	type EditableGameSetting,
	type GameSettingLocaleLabel,
	get_game_setting_locale_labels,
} from '@/helpers/game-options'

export function useGameSettingLabels(
	opened: MaybeRefOrGetter<boolean>,
	instanceId: MaybeRefOrGetter<string | undefined>,
	settings: MaybeRefOrGetter<EditableGameSetting[]>,
) {
	const { locale } = useVIntl()
	const labels = shallowRef<Record<string, GameSettingLocaleLabel>>({})
	const optionIds = computed(() =>
		toValue(settings)
			.map((setting) => setting.option_id)
			.sort(),
	)
	let generation = 0
	let stopListening: UnlistenFn | undefined
	let refreshSources = false

	async function refresh() {
		if (!toValue(opened) || !optionIds.value.length) return
		const request = ++generation
		const reindex = refreshSources
		refreshSources = false
		try {
			const result = await get_game_setting_locale_labels(
				toValue(instanceId),
				locale.value,
				optionIds.value,
				reindex,
			)
			if (request === generation && toValue(opened)) labels.value = result.settings
		} catch (error) {
			console.debug('Could not load Minecraft setting labels', error)
		}
	}

	watch(
		() => toValue(opened),
		(active, _, onCleanup) => {
			let cancelled = false
			onCleanup(() => {
				cancelled = true
				stopListening?.()
				stopListening = undefined
				generation++
				labels.value = {}
			})
			if (!active) return
			refreshSources = true
			void listen('game-option-locales-updated', () => void refresh())
				.then((unlisten) => {
					if (cancelled) unlisten()
					else {
						stopListening = unlisten
						void refresh()
					}
				})
				.catch((error) => {
					console.debug('Could not listen for Minecraft setting labels', error)
					if (!cancelled) void refresh()
				})
		},
		{ flush: 'sync' },
	)

	watch([locale, () => toValue(instanceId), () => optionIds.value.join('\n')], () => {
		generation++
		labels.value = {}
		void refresh()
	})

	onScopeDispose(() => {
		generation++
		stopListening?.()
		labels.value = {}
	})

	return labels
}
