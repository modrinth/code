import { computed } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'
import {
	type FilterType,
	type FilterValue,
	parseServerContentEnvironment,
	SERVER_CONTENT_ENVIRONMENTS,
	type ServerContentEnvironment,
} from '#ui/utils/search'

import type { BrowseManagerContext } from '../providers/browse-manager'

const messages = defineMessages({
	serverEnvironment: {
		id: 'search.server-environment.title',
		defaultMessage: 'Environment',
	},
	playerAndServer: {
		id: 'search.server-environment.player-and-server',
		defaultMessage: 'Player and server',
	},
	playerOnly: {
		id: 'search.server-environment.player-only',
		defaultMessage: 'Player only',
	},
})

export const SERVER_ENVIRONMENT_FILTER_ID = 'server_environment'

export function useServerEnvironmentFilter(ctx: BrowseManagerContext) {
	const { formatMessage } = useVIntl()

	function getLabel(environment: ServerContentEnvironment) {
		switch (environment) {
			case 'server_and_player':
				return formatMessage(messages.playerAndServer)
			case 'server':
				return formatMessage(commonMessages.serverOnlyLabel)
			case 'player':
				return formatMessage(messages.playerOnly)
		}
	}

	const filterType = computed<FilterType>(() => ({
		id: SERVER_ENVIRONMENT_FILTER_ID,
		formatted_name: formatMessage(messages.serverEnvironment),
		options: SERVER_CONTENT_ENVIRONMENTS.map((environment) => ({
			id: environment,
			formatted_name: getLabel(environment),
			method: 'or',
			value: environment,
		})),
		supported_project_types: [],
		query_param: '',
		supports: ['include'],
		searchable: false,
		display: 'radio',
	}))

	/** Bridges the single-select server environment onto the multi-select filter model. */
	const selection = computed<FilterValue[]>({
		get: () =>
			ctx.showServerEnvironment?.value && ctx.serverEnvironment?.value
				? [
						{
							type: SERVER_ENVIRONMENT_FILTER_ID,
							option: ctx.serverEnvironment.value,
							negative: false,
						},
					]
				: [],
		set: (filters) => {
			if (!ctx.serverEnvironment) return
			ctx.serverEnvironment.value = parseServerContentEnvironment(filters.at(-1)?.option)
			ctx.onFilterChange()
		},
	})

	return { filterType, selection }
}
