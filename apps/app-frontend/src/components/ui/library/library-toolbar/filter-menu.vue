<script setup lang="ts">
import { DropdownSelect } from '@modrinth/ui'

import { libraryFilterOptions, useLibrary } from '@/components/ui/library/use-library'

const { activeFilter } = useLibrary()
</script>

<template>
	<DropdownSelect
		v-slot="{ selected }"
		v-model="activeFilter"
		name="Filter Dropdown"
		class="max-w-[16rem]"
		:options="[...libraryFilterOptions]"
		placeholder="Select..."
	>
		<span class="font-semibold text-primary">Filter by: </span>
		<span class="font-semibold text-secondary">{{ selected }}</span>
	</DropdownSelect>
</template>
