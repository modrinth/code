<template>
	<div
		aria-hidden="true"
		class="sticky top-0 z-10 flex h-12 w-full select-none flex-row items-center justify-between gap-4 bg-surface-3 pl-3 pr-4 font-medium transition-[border-radius] duration-100"
		:class="
			isStuck
				? 'rounded-none border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
				: 'rounded-t-[20px]'
		"
	>
		<div class="flex min-w-0 flex-[2.5_1_0%] items-center gap-3">
			<Checkbox
				:model-value="allSelected"
				:indeterminate="someSelected && !allSelected"
				@update:model-value="$emit('toggle-all')"
			/>
			<button
				class="flex appearance-none items-center gap-1.5 border-0 bg-transparent p-0 font-semibold hover:text-primary"
				:class="sortField === 'name' ? 'text-contrast' : 'text-secondary'"
				@click="$emit('sort', 'name')"
			>
				<span>{{ formatMessage(messages.name) }}</span>
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
		<button
			class="hidden min-w-[100px] flex-[0.5_1_0%] appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[640px]:flex"
			:class="sortField === 'size' ? 'text-contrast' : 'text-secondary'"
			@click="$emit('sort', 'size')"
		>
			<span>{{ formatMessage(messages.size) }}</span>
			<ChevronUpIcon v-if="sortField === 'size' && !sortDesc" class="h-4 w-4" aria-hidden="true" />
			<ChevronDownIcon v-if="sortField === 'size' && sortDesc" class="h-4 w-4" aria-hidden="true" />
		</button>
		<button
			v-if="!hideTimestamps"
			class="hidden min-w-[160px] flex-[0.6_1_0%] @[900px]:ml-8 appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[900px]:flex"
			:class="sortField === 'created' ? 'text-contrast' : 'text-secondary'"
			@click="$emit('sort', 'created')"
		>
			<span>{{ formatMessage(messages.created) }}</span>
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
			v-if="!hideTimestamps"
			class="hidden min-w-[160px] flex-[0.6_1_0%] @[900px]:ml-8 appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[500px]:flex"
			:class="sortField === 'modified' ? 'text-contrast' : 'text-secondary'"
			@click="$emit('sort', 'modified')"
		>
			<span>{{ formatMessage(messages.modified) }}</span>
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
		<span
			class="min-w-[51px] shrink-0 text-nowrap text-right font-semibold text-secondary @[900px]:ml-8"
			>{{ formatMessage(commonMessages.actionsLabel) }}</span
		>
	</div>
</template>

<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon } from '@modrinth/assets'

import Checkbox from '#ui/components/base/Checkbox.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { FileSortField } from '../types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	name: {
		id: 'files.table-header.name',
		defaultMessage: 'Name',
	},
	size: {
		id: 'files.table-header.size',
		defaultMessage: 'Size',
	},
	created: {
		id: 'files.table-header.created',
		defaultMessage: 'Created',
	},
	modified: {
		id: 'files.table-header.modified',
		defaultMessage: 'Modified',
	},
})

defineProps<{
	sortField: FileSortField
	sortDesc: boolean
	allSelected: boolean
	someSelected: boolean
	isStuck: boolean
	hideTimestamps?: boolean
}>()

defineEmits<{
	sort: [field: FileSortField]
	'toggle-all': []
}>()
</script>
