<script setup lang="ts">
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
	>
		<template #prefix>
			<span class="font-semibold text-primary">Sort by:</span>
		</template>
	</Combobox>
	<Combobox
		v-model="displayState.group"
		class="w-max"
		:options="groupOptions"
		:show-icon-in-selected="false"
	>
		<template #prefix>
			<span class="font-semibold text-primary">Group by:</span>
		</template>
	</Combobox>
</template>
