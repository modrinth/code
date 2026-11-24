<script setup lang="ts">
import {
	DownloadIcon,
	LayoutGridIcon,
	PlusIcon,
	RefreshCwIcon,
	SquarePlusIcon,
	UploadIcon,
} from '@modrinth/assets'
import { computed } from 'vue'

import {
	type ButtonMenuOption,
	IconButton,
	SplitButton,
	TeleportOverflowMenu,
} from '#ui/components/base/buttons'
import type { DropdownFilterBarCategory } from '#ui/components/base/DropdownFilterBar.vue'
import ListToolbar from '#ui/components/base/ListToolbar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentSortField, ContentSortMode } from '../composables/content-sort'
import { type ContentGroupBy, contentGroupByOptions } from '../composables/use-content-grouping'
import type { ContentMetadataFilterValue } from '../composables/use-content-metadata-filters'
import type { ContentType } from '../types'
import ContentSortMenu from './ContentSortMenu.vue'
import ContentTypeTabs from './ContentTypeTabs.vue'

const props = defineProps<{
	searchPlaceholder: string
	types: ContentType[]
	addDisabled: boolean
	addTooltip?: string
	showUpdateAll: boolean
	updateAllDisabled: boolean
	refreshing: boolean
	refreshDisabled: boolean
	busyTooltip?: string
	filterCategories: DropdownFilterBarCategory[]
	sortFields: ContentSortField[]
	showGrouping: boolean
	newGroupDisabled: boolean
}>()

const search = defineModel<string>('search', { required: true })
const type = defineModel<ContentType>('type', { required: true })
const filters = defineModel<ContentMetadataFilterValue>('filters', { required: true })
const sort = defineModel<ContentSortMode>('sort', { required: true })
const groupBy = defineModel<ContentGroupBy>('groupBy', { required: true })

const emit = defineEmits<{
	add: []
	upload: []
	updateAll: []
	prefetchUpdateAll: []
	refresh: []
	newGroup: [event: MouseEvent]
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
	refreshing: {
		id: 'content.page-layout.refreshing',
		defaultMessage: 'Refreshing...',
	},
	groupBy: {
		id: 'content.page-layout.group-by',
		defaultMessage: 'Group by',
	},
	customGroups: {
		id: 'content.page-layout.group-by.custom',
		defaultMessage: 'Custom groups',
	},
	category: {
		id: 'content.page-layout.group-by.category',
		defaultMessage: 'Category',
	},
	author: {
		id: 'content.page-layout.group-by.author',
		defaultMessage: 'Author',
	},
	dateAdded: {
		id: 'content.page-layout.group-by.date-added',
		defaultMessage: 'Date added',
	},
	license: {
		id: 'content.page-layout.group-by.license',
		defaultMessage: 'License',
	},
	noGrouping: {
		id: 'content.page-layout.group-by.none',
		defaultMessage: 'No grouping',
	},
	newGroup: {
		id: 'content.page-layout.new-group',
		defaultMessage: 'New group',
	},
})

const groupByLabels: Record<ContentGroupBy, (typeof messages)[keyof typeof messages]> = {
	custom: messages.customGroups,
	category: messages.category,
	author: messages.author,
	date: messages.dateAdded,
	license: messages.license,
	none: messages.noGrouping,
}

const groupByOptions = computed<ButtonMenuOption[]>(() =>
	contentGroupByOptions.map((option) => ({
		id: option,
		label: formatMessage(groupByLabels[option]),
		selected: groupBy.value === option,
		action: () => {
			groupBy.value = option
		},
	})),
)

const newGroupLabel = computed(() => formatMessage(messages.newGroup))

const updateAllLabel = computed(() => formatMessage(messages.updateAll))

const refreshLabel = computed(() =>
	formatMessage(props.refreshing ? messages.refreshing : commonMessages.refreshButton),
)

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
		<div class="flex min-h-[42px] items-center justify-between gap-2">
			<ContentTypeTabs v-if="types.length > 1" v-model="type" :types="types" />
			<SplitButton
				v-tooltip="addTooltip"
				type="colored"
				color="brand"
				size="lg"
				class="ml-auto"
				:options="addOptions"
				:menu-label="formatMessage(messages.moreAddOptions)"
				:disabled="addDisabled"
				@click="emit('add')"
			>
				<PlusIcon aria-hidden="true" />
				{{ formatMessage(messages.addContent) }}
			</SplitButton>
		</div>

		<ListToolbar
			v-model:search="search"
			v-model:filters="filters"
			:search-placeholder="searchPlaceholder"
			:filter-categories="filterCategories"
		>
			<template #controls>
				<Transition name="toolbar-item">
					<div v-if="showGrouping" class="flex shrink-0">
						<TeleportOverflowMenu
							type="quiet"
							size="lg"
							:label="formatMessage(messages.groupBy)"
							:options="groupByOptions"
							:icon-only="false"
							animate-width
						>
							<LayoutGridIcon aria-hidden="true" />
							{{ formatMessage(groupByLabels[groupBy]) }}
						</TeleportOverflowMenu>
					</div>
				</Transition>
				<ContentSortMenu v-model="sort" :fields="sortFields" />
			</template>
			<template #actions>
				<Transition name="toolbar-item">
					<div v-if="showUpdateAll" class="flex shrink-0">
						<IconButton
							v-tooltip="updateAllDisabled ? busyTooltip : updateAllLabel"
							type="quiet"
							color="green"
							size="lg"
							:label="updateAllLabel"
							:disabled="updateAllDisabled"
							interaction="filled"
							@click="emit('updateAll')"
							@mouseenter="emit('prefetchUpdateAll')"
							@focus="emit('prefetchUpdateAll')"
						>
							<DownloadIcon />
						</IconButton>
					</div>
				</Transition>
				<IconButton
					v-tooltip="refreshDisabled ? (busyTooltip ?? refreshLabel) : refreshLabel"
					type="quiet"
					size="lg"
					:label="refreshLabel"
					:disabled="refreshing || refreshDisabled"
					@click="emit('refresh')"
				>
					<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
				</IconButton>
				<Transition name="toolbar-item">
					<div v-if="showGrouping" class="flex shrink-0">
						<IconButton
							v-tooltip="newGroupLabel"
							type="quiet"
							size="lg"
							:label="newGroupLabel"
							:disabled="newGroupDisabled"
							@click="(event: MouseEvent) => emit('newGroup', event)"
						>
							<SquarePlusIcon />
						</IconButton>
					</div>
				</Transition>
			</template>
		</ListToolbar>
	</div>
</template>

<style scoped>
.toolbar-item-enter-active,
.toolbar-item-leave-active {
	overflow: hidden;
	transition:
		max-width 200ms cubic-bezier(0.4, 0, 0.2, 1),
		margin-left 200ms cubic-bezier(0.4, 0, 0.2, 1),
		opacity 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.toolbar-item-enter-from,
.toolbar-item-leave-to {
	max-width: 0;
	margin-left: -0.5rem;
	opacity: 0;
}

.toolbar-item-enter-to,
.toolbar-item-leave-from {
	max-width: 16rem;
	margin-left: 0;
	opacity: 1;
}
</style>
