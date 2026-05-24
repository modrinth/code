import type { Labrinth } from '@modrinth/api-client'
import { type ComputedRef, type Ref, ref } from 'vue'
import type { RouteLocationRaw } from 'vue-router'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { injectNotificationManager } from '#ui/providers/web-notifications'

import type { BrowseSearchResponse } from '../types'

export interface UseFeelingLuckyOptions {
	search: (params: string) => Promise<BrowseSearchResponse>
	totalHits: Ref<number>
	refreshSearch: () => Promise<void>
	isServerType: ComputedRef<boolean> | Ref<boolean>
	buildRequestParams: (options?: { limit?: number; offset?: number }) => string
	getProjectLink: (result: Labrinth.Search.v2.ResultSearchProject) => string | RouteLocationRaw
	getServerProjectLink: (
		result: Labrinth.Search.v3.ResultSearchProject,
	) => string | RouteLocationRaw
	navigate: (to: string | RouteLocationRaw) => void | Promise<void>
}

const messages = defineMessages({
	noResults: {
		id: 'browse.feeling-lucky.no-results',
		defaultMessage: 'No projects match your current filters.',
	},
	failed: {
		id: 'browse.feeling-lucky.failed',
		defaultMessage: 'Could not load a random project. Please try again.',
	},
})

export function useFeelingLucky(options: UseFeelingLuckyOptions) {
	const { formatMessage } = useVIntl()
	const { addNotification } = injectNotificationManager()
	const feelingLuckyLoading = ref(false)

	async function pickRandomProject() {
		if (feelingLuckyLoading.value) return

		feelingLuckyLoading.value = true
		try {
			let total = options.totalHits.value
			if (total === 0) {
				await options.refreshSearch()
				total = options.totalHits.value
			}

			if (total === 0) {
				addNotification({
					title: formatMessage(messages.noResults),
					type: 'warning',
				})
				return
			}

			const offset = Math.floor(Math.random() * total)
			const response = await options.search(options.buildRequestParams({ limit: 1, offset }))

			const hit = options.isServerType.value ? response.serverHits[0] : response.projectHits[0]

			if (!hit) {
				addNotification({
					title: formatMessage(messages.failed),
					type: 'error',
				})
				return
			}

			const link = options.isServerType.value
				? options.getServerProjectLink(hit as Labrinth.Search.v3.ResultSearchProject)
				: options.getProjectLink(hit as Labrinth.Search.v2.ResultSearchProject)

			await options.navigate(link)
		} catch {
			addNotification({
				title: formatMessage(messages.failed),
				type: 'error',
			})
		} finally {
			feelingLuckyLoading.value = false
		}
	}

	return {
		feelingLuckyLoading,
		pickRandomProject,
	}
}
