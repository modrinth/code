<script setup lang="ts">
import { LayoutGridIcon } from '@modrinth/assets'
import {
	type ButtonMenuOption,
	defineMessages,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

import { libraryGroupOptions, useLibrary } from '@/components/ui/library/use-library'

const { displayState } = useLibrary()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	customGroup: { id: 'app.library.group-by.custom-group', defaultMessage: 'Custom group' },
	instanceType: { id: 'app.library.group-by.instance-type', defaultMessage: 'Instance type' },
	loader: { id: 'app.library.group-by.loader', defaultMessage: 'Loader' },
	gameVersion: { id: 'app.library.group-by.game-version', defaultMessage: 'Game version' },
	noGrouping: { id: 'app.library.group-by.none', defaultMessage: 'No grouping' },
	groupBy: { id: 'app.library.group-by.label', defaultMessage: 'Group by' },
})

const groupLabels = {
	Group: messages.customGroup,
	'Instance type': messages.instanceType,
	Loader: messages.loader,
	'Game version': messages.gameVersion,
	None: messages.noGrouping,
}

const activeLabel = computed(() => formatMessage(groupLabels[displayState.value.group]))

const groupOptions = computed<ButtonMenuOption[]>(() =>
	libraryGroupOptions.map((option) => ({
		id: option.value,
		label: formatMessage(groupLabels[option.value]),
		selected: option.value === displayState.value.group,
		action: () => {
			displayState.value.group = option.value
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
