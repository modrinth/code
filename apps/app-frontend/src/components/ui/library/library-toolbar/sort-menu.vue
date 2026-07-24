<script setup lang="ts">
import { DropdownSelect } from '@modrinth/ui'

import {
	libraryGroupOptions,
	librarySortOptions,
	useLibrary,
} from '@/components/ui/library/use-library'

const { displayState } = useLibrary()
</script>

<template>
	<DropdownSelect
		v-slot="{ selected }"
		v-model="displayState.sortBy"
		name="Sort Dropdown"
		class="max-w-[16rem]"
		:options="[...librarySortOptions]"
		placeholder="Select..."
	>
		<span class="font-semibold text-primary">Sort by: </span>
		<span class="font-semibold text-secondary">{{ selected }}</span>
	</DropdownSelect>
	<DropdownSelect
		v-slot="{ selected }"
		v-model="displayState.group"
		class="max-w-[16rem]"
		name="Group Dropdown"
		:options="[...libraryGroupOptions]"
		placeholder="Select..."
	>
		<span class="font-semibold text-primary">Group by: </span>
		<span class="font-semibold text-secondary">{{ selected }}</span>
	</DropdownSelect>
</template>
