<template>
	<div
		aria-hidden="true"
		class="sticky top-0 z-10 flex h-12 w-full select-none flex-row items-center justify-between bg-surface-3 pl-3 pr-4 font-medium transition-[border-radius] duration-100"
		:class="
			isStuck
				? 'rounded-none border-0 border-y border-solid border-surface-4 shadow-md before:pointer-events-none before:absolute before:inset-x-0 before:-top-4 before:h-5 before:bg-surface-3'
				: 'rounded-t-[20px]'
		"
	>
		<div class="flex flex-1 items-center gap-3">
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
		<div class="flex shrink-0 items-center gap-4 @[800px]:gap-12">
			<button
				class="hidden w-[100px] appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[800px]:flex"
				:class="sortField === 'size' ? 'text-contrast' : 'text-secondary'"
				@click="$emit('sort', 'size')"
			>
				<span class="ml-2">{{ formatMessage(messages.size) }}</span>
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
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[800px]:flex"
				:class="sortField === 'created' ? 'text-contrast' : 'text-secondary'"
				@click="$emit('sort', 'created')"
			>
				<span class="ml-2">{{ formatMessage(messages.created) }}</span>
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
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 border-0 bg-transparent p-0 font-semibold hover:text-primary @[800px]:flex"
				:class="sortField === 'modified' ? 'text-contrast' : 'text-secondary'"
				@click="$emit('sort', 'modified')"
			>
				<span class="ml-2">{{ formatMessage(messages.modified) }}</span>
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
			<span class="min-w-[51px] shrink-0 text-right font-semibold text-secondary">{{
				formatMessage(commonMessages.actionsLabel)
			}}</span>
		</div>
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
}>()

defineEmits<{
	sort: [field: FileSortField]
	'toggle-all': []
}>()
</script>
