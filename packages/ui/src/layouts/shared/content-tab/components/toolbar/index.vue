<script setup lang="ts">
import { DownloadIcon, PlusIcon, RefreshCwIcon, SearchIcon, UploadIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { Button, type ButtonMenuOption, IconButton, SplitButton } from '#ui/components/base/buttons'
import type { DropdownFilterBarCategory } from '#ui/components/base/DropdownFilterBar.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentSortMode } from '../../composables/content-sort'
import type { ContentMetadataFilterValue } from '../../composables/use-content-metadata-filters'
import type { ContentType } from '../../types'
import SortMenu from '../sort-menu.vue'
import FilterMenu from './filter-menu.vue'
import TypeTabs from './type-tabs.vue'

const props = defineProps<{
	searchPlaceholder: string
	types: ContentType[]
	addDisabled: boolean
	addTooltip?: string
	showUpdateAll: boolean
	updateAllDisabled: boolean
	refreshing: boolean
	filterCategories: DropdownFilterBarCategory[]
	showDateSort: boolean
}>()

const search = defineModel<string>('search', { required: true })
const type = defineModel<ContentType>('type', { required: true })
const filters = defineModel<ContentMetadataFilterValue>('filters', { required: true })
const sort = defineModel<ContentSortMode>('sort', { required: true })

const emit = defineEmits<{
	add: []
	upload: []
	updateAll: []
	refresh: []
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	addContent: {
		id: 'content.page-layout.add-content',
		defaultMessage: 'Add content',
	},
	moreAddOptions: {
		id: 'content.page-layout.more-add-options',
		defaultMessage: 'More ways to add content',
	},
	uploadFiles: {
		id: 'content.page-layout.upload-files',
		defaultMessage: 'Upload files',
	},
	updateAll: {
		id: 'content.page-layout.update-all',
		defaultMessage: 'Update all',
	},
})

const addOptions = computed<ButtonMenuOption[]>(() => [
	{
		id: 'upload-files',
		label: formatMessage(messages.uploadFiles),
		icon: UploadIcon,
		disabled: props.addDisabled,
		action: () => emit('upload'),
	},
])
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				:spellcheck="false"
				size="medium"
				wrapper-class="flex-1 min-w-0"
				clearable
				:placeholder="searchPlaceholder"
			/>
			<SplitButton
				v-tooltip="addTooltip"
				type="colored"
				color="brand"
				size="lg"
				:options="addOptions"
				:menu-label="formatMessage(messages.moreAddOptions)"
				:disabled="addDisabled"
				@click="emit('add')"
			>
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.addContent) }}
			</SplitButton>
		</div>

		<div class="flex flex-wrap items-center justify-between gap-2">
			<TypeTabs v-if="types.length > 1" v-model="type" :types="types" />
			<div class="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
				<Button
					v-if="showUpdateAll"
					type="quiet"
					color="green"
					size="lg"
					:disabled="updateAllDisabled"
					class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="emit('updateAll')"
				>
					<DownloadIcon />
					{{ formatMessage(messages.updateAll) }}
				</Button>
				<IconButton
					v-tooltip="formatMessage(commonMessages.refreshButton)"
					type="quiet"
					size="lg"
					:label="formatMessage(commonMessages.refreshButton)"
					:disabled="refreshing"
					@click="emit('refresh')"
				>
					<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
				</IconButton>
				<FilterMenu
					v-if="filterCategories.length > 0"
					v-model="filters"
					:categories="filterCategories"
				/>
				<SortMenu v-model="sort" :show-date-options="showDateSort" />
			</div>
		</div>
	</div>
</template>
