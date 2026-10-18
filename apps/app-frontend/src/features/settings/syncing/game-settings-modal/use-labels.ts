import { useDebugLogger, useVIntl } from '@modrinth/ui'
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
	const debug = useDebugLogger('GameSettingLabels')
	const { locale } = useVIntl()
	const labels = shallowRef<Record<string, GameSettingLocaleLabel>>({})
	const optionIds = computed(() =>
		toValue(settings)
			.filter((setting) => setting.kind === 'external')
			.map((setting) => setting.option_id)
			.sort(),
	)
	let generation = 0
	let stopListening: UnlistenFn | undefined
	let refreshSources = false

	debug('setup', { opened: toValue(opened), instanceId: toValue(instanceId), locale: locale.value })

	async function refresh(reason: string) {
		if (!toValue(opened) || !optionIds.value.length) {
			debug('refresh skipped', {
				reason,
				opened: toValue(opened),
				optionCount: optionIds.value.length,
			})
			return
		}
		const request = ++generation
		const reindex = refreshSources
		const requestedIds = [...optionIds.value]
		const started = performance.now()
		refreshSources = false
		debug('request started', {
			request,
			reason,
			instanceId: toValue(instanceId),
			locale: locale.value,
			reindex,
			optionIds: requestedIds,
		})
		try {
			const result = await get_game_setting_locale_labels(
				toValue(instanceId),
				locale.value,
				requestedIds,
				reindex,
			)
			const applied = request === generation && toValue(opened)
			debug('request completed', {
				request,
				generation,
				applied,
				opened: toValue(opened),
				elapsedMs: Math.round(performance.now() - started),
				returnedCount: Object.keys(result.settings).length,
				missingOptionIds: requestedIds.filter((id) => !result.settings[id]),
				labelSample: Object.entries(result.settings).slice(0, 8),
			})
			if (applied) labels.value = result.settings
		} catch (error) {
			debug('request failed', {
				request,
				reason,
				elapsedMs: Math.round(performance.now() - started),
				error,
			})
		}
	}

	watch(
		() => toValue(opened),
		(active, _, onCleanup) => {
			debug('modal state changed', { active, generation, optionCount: optionIds.value.length })
			let cancelled = false
			onCleanup(() => {
				cancelled = true
				stopListening?.()
				stopListening = undefined
				generation++
				labels.value = {}
				debug('modal cleanup: listener removed and labels cleared', { generation })
			})
			if (!active) return
			refreshSources = true
			debug('registering locale update listener')
			void listen('game-option-locales-updated', () => {
				debug('received game-option-locales-updated')
				void refresh('locale update event')
			})
				.then((unlisten) => {
					debug('locale update listener registered', { cancelled })
					if (cancelled) unlisten()
					else {
						stopListening = unlisten
						void refresh('listener registered')
					}
				})
				.catch((error) => {
					debug('locale update listener failed', { cancelled, error })
					if (!cancelled) void refresh('listener failed')
				})
		},
		{ flush: 'sync' },
	)

	watch([locale, () => toValue(instanceId), () => optionIds.value.join('\n')], () => {
		generation++
		labels.value = {}
		debug('locale, instance or option IDs changed: labels cleared', {
			generation,
			locale: locale.value,
			instanceId: toValue(instanceId),
			optionCount: optionIds.value.length,
		})
		void refresh('request inputs changed')
	})

	onScopeDispose(() => {
		generation++
		stopListening?.()
		labels.value = {}
		debug('scope disposed', { generation })
	})

	return labels
}
