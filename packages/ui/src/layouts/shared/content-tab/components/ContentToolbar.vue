<script setup lang="ts">
import { DownloadIcon, PlusIcon, RefreshCwIcon, UploadIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { type ButtonMenuOption, IconButton, SplitButton } from '#ui/components/base/buttons'
import type { DropdownFilterBarCategory } from '#ui/components/base/DropdownFilterBar.vue'
import ListToolbar from '#ui/components/base/ListToolbar.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentSortField, ContentSortMode } from '../composables/content-sort'
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
}>()

const search = defineModel<string>('search', { required: true })
const type = defineModel<ContentType>('type', { required: true })
const filters = defineModel<ContentMetadataFilterValue>('filters', { required: true })
const sort = defineModel<ContentSortMode>('sort', { required: true })

const emit = defineEmits<{
	add: []
	upload: []
	updateAll: []
	prefetchUpdateAll: []
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
	refreshing: {
		id: 'content.page-layout.refreshing',
		defaultMessage: 'Refreshing...',
	},
})

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
		<div class="flex items-center justify-between gap-2">
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
				<ContentSortMenu v-model="sort" :fields="sortFields" />
			</template>
			<template #actions>
				<Transition name="update-all">
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
				<!-- TODO: grouping
				<IconButton type="quiet" size="lg" :label="formatMessage(messages.group)">
					<SquarePlusIcon />
				</IconButton>
				-->
			</template>
		</ListToolbar>
	</div>
</template>

<style scoped>
.update-all-enter-active,
.update-all-leave-active {
	overflow: hidden;
	transition:
		width 200ms cubic-bezier(0.4, 0, 0.2, 1),
		margin-left 200ms cubic-bezier(0.4, 0, 0.2, 1),
		opacity 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.update-all-enter-from,
.update-all-leave-to {
	width: 0;
	margin-left: -0.5rem;
	opacity: 0;
}

.update-all-enter-to,
.update-all-leave-from {
	width: 2.5rem;
	margin-left: 0;
	opacity: 1;
}
</style>
