<script setup lang="ts">
import { ArrowUpDownIcon, LayoutGridIcon } from '@modrinth/assets'
import { Combobox, type ComboboxOption, defineMessages, useVIntl } from '@modrinth/ui'

import {
	type LibraryGroupBy,
	libraryGroupOptions,
	type LibrarySort,
	librarySortOptions,
	useLibrary,
} from '@/components/ui/library/use-library'

const { displayState } = useLibrary()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	name: { id: 'app.library.sort.name', defaultMessage: 'Name' },
	loaderSort: { id: 'app.library.sort.loader', defaultMessage: 'Loader' },
	gameVersionSort: { id: 'app.library.sort.game-version', defaultMessage: 'Game version' },
	lastPlayed: { id: 'app.library.sort.last-played', defaultMessage: 'Last played' },
	hoursPlayed: { id: 'app.library.sort.hours-played', defaultMessage: 'Hours played' },
	dateCreated: { id: 'app.library.sort.date-created', defaultMessage: 'Date created' },
	dateModified: { id: 'app.library.sort.date-modified', defaultMessage: 'Date modified' },
	customGroup: { id: 'app.library.group-by.custom-group', defaultMessage: 'Custom group' },
	instanceType: { id: 'app.library.group-by.instance-type', defaultMessage: 'Instance type' },
	loader: { id: 'app.library.group-by.loader', defaultMessage: 'Loader' },
	gameVersion: { id: 'app.library.group-by.game-version', defaultMessage: 'Game version' },
	noGrouping: { id: 'app.library.group-by.none', defaultMessage: 'No grouping' },
	sortBy: { id: 'app.library.sort.label', defaultMessage: 'Sort by' },
	groupBy: { id: 'app.library.group-by.label', defaultMessage: 'Group by' },
})

const sortLabels = {
	Name: messages.name,
	'Last played': messages.lastPlayed,
	'Hours played': messages.hoursPlayed,
	'Date created': messages.dateCreated,
	'Date modified': messages.dateModified,
	Loader: messages.loaderSort,
	'Game version': messages.gameVersionSort,
}

const groupLabels = {
	Group: messages.customGroup,
	'Instance type': messages.instanceType,
	Loader: messages.loader,
	'Game version': messages.gameVersion,
	None: messages.noGrouping,
}

const sortOptions: ComboboxOption<LibrarySort>[] = librarySortOptions.map((option) => ({
	value: option,
	label: formatMessage(sortLabels[option]),
}))
const groupOptions: ComboboxOption<LibraryGroupBy>[] = libraryGroupOptions.map((option) => ({
	value: option.value,
	label: formatMessage(groupLabels[option.value]),
}))
</script>

<template>
	<Combobox
		v-model="displayState.sortBy"
		class="w-max"
		:options="sortOptions"
		:show-icon-in-selected="false"
		:max-height="320"
		dropdown-min-width="160px"
	>
		<template #prefix>
			<ArrowUpDownIcon class="size-5 text-primary" :aria-label="formatMessage(messages.sortBy)" />
		</template>
		<template #selected="{ label }">
			<span>{{ label }}</span>
		</template>
	</Combobox>
	<Combobox
		v-model="displayState.group"
		class="w-max"
		:options="groupOptions"
		dropdown-min-width="160px"
		:show-icon-in-selected="false"
	>
		<template #prefix>
			<LayoutGridIcon class="size-5 text-primary" :aria-label="formatMessage(messages.groupBy)" />
		</template>
		<template #selected="{ label }">
			<span>{{ label }}</span>
		</template>
	</Combobox>
</template>
