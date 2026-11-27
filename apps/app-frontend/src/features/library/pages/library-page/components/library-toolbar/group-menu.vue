<script setup lang="ts">
import { LayoutGridIcon } from '@modrinth/assets'
import { type ButtonMenuOption, defineMessages, TeleportOverflowMenu, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { injectLibraryView } from '../../library-context'
import { type LibraryGroupBy, libraryGroupOptions } from '../../types'

const { displayState } = injectLibraryView()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	customGroup: { id: 'app.library.group-by.custom-group', defaultMessage: 'Custom group' },
	instanceType: { id: 'app.library.group-by.instance-type', defaultMessage: 'Instance type' },
	loader: { id: 'app.library.group-by.loader', defaultMessage: 'Loader' },
	gameVersion: { id: 'app.library.group-by.game-version', defaultMessage: 'Game version' },
	noGrouping: { id: 'app.library.group-by.none', defaultMessage: 'No grouping' },
	groupBy: { id: 'app.library.group-by.label', defaultMessage: 'Group by' },
})

const groupLabels: Record<LibraryGroupBy, (typeof messages)[keyof typeof messages]> = {
	Group: messages.customGroup,
	'Instance type': messages.instanceType,
	Loader: messages.loader,
	'Game version': messages.gameVersion,
	None: messages.noGrouping,
}

const activeLabel = computed(() => formatMessage(groupLabels[displayState.value.group]))

const groupOptions = computed<ButtonMenuOption[]>(() =>
	libraryGroupOptions.map((option) => ({
		id: option,
		label: formatMessage(groupLabels[option]),
		selected: option === displayState.value.group,
		action: () => {
			displayState.value.group = option
		},
	})),
)
</script>

<template>
	<TeleportOverflowMenu
		type="quiet"
		size="lg"
		:label="formatMessage(messages.groupBy)"
		:options="groupOptions"
		:icon-only="false"
		animate-width
	>
		<LayoutGridIcon aria-hidden="true" />
		{{ activeLabel }}
	</TeleportOverflowMenu>
</template>
