<script setup lang="ts">
import { ArrowUpDownIcon, LayoutGridIcon } from '@modrinth/assets'
import { Combobox, type ComboboxOption } from '@modrinth/ui'

import {
	type LibraryGroupBy,
	libraryGroupOptions,
	type LibrarySort,
	librarySortOptions,
	useLibrary,
} from '@/components/ui/library/use-library'

const { displayState } = useLibrary()

const sortOptions: ComboboxOption<LibrarySort>[] = librarySortOptions.map((option) => ({
	value: option,
	label: option,
}))
const groupOptions: ComboboxOption<LibraryGroupBy>[] = [...libraryGroupOptions]
</script>

<template>
	<Combobox
		v-model="displayState.sortBy"
		class="w-max"
		:options="sortOptions"
		:show-icon-in-selected="false"
		dropdown-min-width="160px"
	>
		<template #prefix>
			<ArrowUpDownIcon class="size-5 text-primary" aria-label="Sort by" />
		</template>
		<template #selected="{ label }">
			<span>{{ label }}</span>
		</template>
	</Combobox>
	<Combobox
		v-model="displayState.group"
		class="w-max"
		:options="groupOptions"
		:show-icon-in-selected="false"
	>
		<template #prefix>
			<LayoutGridIcon class="size-5 text-primary" aria-label="Group by" />
		</template>
		<template #selected="{ label }">
			<span>{{ label }}</span>
		</template>
	</Combobox>
</template>
