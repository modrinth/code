<template>
	<div
		aria-hidden="true"
		class="sticky top-0 z-20 flex w-full select-none flex-row items-center justify-between border border-b-0 border-solid border-surface-3 bg-surface-3 p-4 text-sm font-medium transition-[border-radius] duration-100 before:pointer-events-none before:absolute before:inset-x-0 before:-top-5 before:h-5 before:bg-surface-3"
		:class="isStuck ? 'rounded-none' : 'rounded-t-[20px]'"
	>
		<div class="flex flex-1 items-center gap-3">
			<Checkbox
				:model-value="allSelected"
				:indeterminate="someSelected && !allSelected"
				@update:model-value="$emit('toggle-all')"
			/>
			<button
				class="flex appearance-none items-center gap-1.5 bg-transparent text-contrast hover:text-brand"
				@click="$emit('sort', 'name')"
			>
				<span>Name</span>
				<ChevronUpIcon
					v-if="sortField === 'name' && !sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === 'name' && sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
			</button>
		</div>
		<div class="flex shrink-0 items-center gap-4 md:gap-12">
			<button
				class="hidden w-[100px] appearance-none items-center justify-start gap-1 bg-transparent text-primary hover:text-brand md:flex"
				@click="$emit('sort', 'size')"
			>
				<span>Size</span>
				<ChevronUpIcon
					v-if="sortField === 'size' && !sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === 'size' && sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
			</button>
			<button
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 bg-transparent text-primary hover:text-brand md:flex"
				@click="$emit('sort', 'created')"
			>
				<span>Created</span>
				<ChevronUpIcon
					v-if="sortField === 'created' && !sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === 'created' && sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
			</button>
			<button
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 bg-transparent text-primary hover:text-brand md:flex"
				@click="$emit('sort', 'modified')"
			>
				<span>Modified</span>
				<ChevronUpIcon
					v-if="sortField === 'modified' && !sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
				<ChevronDownIcon
					v-if="sortField === 'modified' && sortDesc"
					class="h-4 w-4"
					aria-hidden="true"
				/>
			</button>
			<span class="w-[51px] text-right text-primary">Actions</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { Checkbox } from '@modrinth/ui'

import ChevronDownIcon from './icons/ChevronDownIcon.vue'
import ChevronUpIcon from './icons/ChevronUpIcon.vue'

defineProps<{
	sortField: string
	sortDesc: boolean
	allSelected: boolean
	someSelected: boolean
	isStuck: boolean
}>()

defineEmits<{
	(e: 'sort', field: string): void
	(e: 'toggle-all'): void
}>()
</script>
