<template>
	<div
		aria-hidden="true"
		class="sticky top-0 z-10 flex h-12 w-full select-none flex-row items-center justify-between bg-surface-3 px-3 text-sm font-medium transition-[border-radius] duration-100"
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
				class="flex appearance-none items-center gap-1.5 bg-transparent font-semibold text-secondary"
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
		<div class="flex shrink-0 items-center gap-4 md:gap-12">
			<button
				class="hidden w-[100px] appearance-none items-center justify-start gap-1 bg-transparent font-semibold text-secondary md:flex"
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
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 bg-transparent font-semibold text-secondary md:flex"
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
				class="hidden w-[160px] appearance-none items-center justify-start gap-1 bg-transparent font-semibold text-secondary md:flex"
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
			<span class="w-[51px] text-right font-semibold text-secondary">{{
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
	sortField: string
	sortDesc: boolean
	allSelected: boolean
	someSelected: boolean
	isStuck: boolean
}>()

defineEmits<{
	sort: [field: string]
	'toggle-all': []
}>()
</script>
