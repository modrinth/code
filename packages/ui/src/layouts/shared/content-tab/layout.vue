<script setup lang="ts">
import {
	CodeIcon,
	DownloadIcon,
	DropdownIcon,
	FileIcon,
	LinkIcon,
	ShareIcon,
	TextCursorInputIcon,
	TrashIcon,
} from '@modrinth/assets'
import { computed, ref, watch } from 'vue'

import { Button, type ButtonMenuOption, TeleportOverflowMenu } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages, formatContentTypeSentence } from '#ui/utils/common-messages'

import ContentTable from './components/content-table/index.vue'
import ContentSelectionBar from './components/ContentSelectionBar.vue'
import ContentEmptyState from './components/empty-state.vue'
import DeleteFlow from './components/flows/delete-flow.vue'
import ToggleFlow from './components/flows/toggle-flow.vue'
import type { ContentFlowState } from './components/flows/types'
import UpdateAllFlow from './components/flows/update-all-flow.vue'
import ManagedContentCard from './components/managed-content-card/index.vue'
import ConfirmUnlinkModal from './components/modals/ConfirmUnlinkModal.vue'
import ContentToolbar from './components/toolbar/index.vue'
import {
	useBulkOperation,
	useChangingItems,
	useContentPrefetch,
	useContentRows,
	useContentSelection,
	useContentViewState,
} from './composables'
import { injectContentManager } from './providers/content-manager'
import type { ContentSide } from './types'

const props = withDefaults(
	defineProps<{
		bottomPadding?: boolean
		highlightedItemId?: string
	}>(),
	{
		bottomPadding: true,
	},
)

const { formatMessage } = useVIntl()

const messages = defineMessages({
	failedToLoad: {
		id: 'content.page-layout.failed-to-load',
		defaultMessage: 'Failed to load content',
	},
	additionalContent: {
		id: 'content.page-layout.additional-content',
		defaultMessage: 'Additional content',
	},
	searchPlaceholder: {
		id: 'content.page-layout.search-placeholder',
		defaultMessage: 'Search {count, number} {contentType}...',
	},
	noContentFound: {
		id: 'content.page-layout.no-content-found',
		defaultMessage: 'No content found.',
	},
	noExtraContentInstalled: {
		id: 'content.page-layout.empty.no-extra-content-installed',
		defaultMessage: 'No extra content installed',
	},
	noContentInstalled: {
		id: 'content.page-layout.empty.no-content-installed',
		defaultMessage: 'No content installed',
	},
	emptyModpackHint: {
		id: 'content.page-layout.empty.modpack-hint',
		defaultMessage: 'Add additional content on top of this modpack',
	},
	emptyHint: {
		id: 'content.page-layout.empty.projects-hint',
		defaultMessage: 'Browse or upload projects to get started.',
	},
	noTypeInstalled: {
		id: 'content.page-layout.empty.no-type-installed',
		defaultMessage: 'No {contentType} installed',
	},
	shareProjectNames: {
		id: 'content.page-layout.share.project-names',
		defaultMessage: 'Project names',
	},
	shareFileNames: {
		id: 'content.page-layout.share.file-names',
		defaultMessage: 'File names',
	},
	shareProjectLinks: {
		id: 'content.page-layout.share.project-links',
		defaultMessage: 'Project links',
	},
	shareMarkdownLinks: {
		id: 'content.page-layout.share.markdown-links',
		defaultMessage: 'Markdown links',
	},
	share: {
		id: 'content.page-layout.share.label',
		defaultMessage: 'Share',
	},
})

const ctx = injectContentManager()
const isServer = computed(() => ctx.target.value.kind === 'server')

const {
	types,
	activeType,
	typeItems,
	sortMode,
	hasDateAdded,
	searchQuery,
	filters,
	metadataFilterCategories,
	visibleItems,
	retain,
	clearRetained,
	reveal,
} = useContentViewState({
	items: ctx.items,
	types: ctx.types,
	persistKey: ctx.persistKey,
	searchQueryParam: ctx.searchQueryParam,
	showSharedContentFilter: ctx.showSharedContentFilter,
	showEnabledFor: !!ctx.setEnabledFor,
	showEnvironmentWarnings: isServer,
})

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	(item) => item.id,
)
watch(activeType, clearSelection)

const bulk = useBulkOperation()
const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, bulkWaiting } = bulk
const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const { toRow, toDisplayRow } = useContentRows({
	ctx,
	isServer,
	isPending: (item) =>
		ctx.disableWhileMutating !== false && (isChanging(item.id) || isBulkOperating.value),
})
const rows = computed(() => visibleItems.value.map(toRow))

const flowState: ContentFlowState = {
	selectedItems,
	clearSelection,
	removeFromSelection,
	bulk,
	markChanging,
	unmarkChanging,
	retain,
	itemType: activeType,
}

const toggleFlow = ref<InstanceType<typeof ToggleFlow>>()
const deleteFlow = ref<InstanceType<typeof DeleteFlow>>()
const updateAllFlow = ref<InstanceType<typeof UpdateAllFlow>>()
const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()

const isBusy = computed(() => !!ctx.busy.value)
const busyMessage = computed(() => ctx.busy.value?.message)
const addTooltip = computed(
	() => busyMessage.value ?? (ctx.canAdd.value ? undefined : ctx.addDeniedReason),
)
const hasOutdatedItems = computed(() =>
	typeItems.value.some((item) => item.has_update && !item.locked),
)
const toggleableSelectedItems = computed(() =>
	selectedItems.value.filter((item) => ctx.permissions(item).toggle),
)
const removableSelectedItems = computed(() =>
	selectedItems.value.filter((item) => ctx.permissions(item).remove),
)
const searchPlaceholder = computed(() =>
	formatMessage(messages.searchPlaceholder, {
		count: typeItems.value.length,
		contentType: formatContentTypeSentence(formatMessage, activeType.value, typeItems.value.length),
	}),
)
const emptyTableMessage = computed(() =>
	formatMessage(typeItems.value.length > 0 ? messages.noContentFound : messages.noTypeInstalled, {
		contentType: formatContentTypeSentence(formatMessage, activeType.value, 2),
	}),
)

watch(
	() => props.highlightedItemId,
	(id) => {
		const item = id ? ctx.items.value.find((candidate) => candidate.id === id) : undefined
		if (item) reveal(item)
	},
	{ immediate: true },
)

const refreshing = ref(false)
async function refresh() {
	if (refreshing.value) return
	clearRetained()
	refreshing.value = true
	try {
		await Promise.all([ctx.refresh(), new Promise((resolve) => setTimeout(resolve, 250))])
	} finally {
		refreshing.value = false
	}
}

function findItem(id: string) {
	return ctx.items.value.find((item) => item.id === id)
}

const { prefetchItem, prefetchUpdates } = useContentPrefetch()

function prefetchRow(id: string) {
	const item = findItem(id)
	if (item) prefetchItem(item)
}

const updateCandidatesKey = computed(() =>
	ctx.loading.value
		? ''
		: ctx.items.value
				.filter((item) => item.has_update && !item.locked && item.update_version_id)
				.map((item) => `${item.id}:${item.update_version_id}`)
				.join('|'),
)
watch(
	updateCandidatesKey,
	(key) => {
		if (key && ctx.applyUpdates) prefetchUpdates(ctx.items.value)
	},
	{ immediate: true },
)

function toggleItem(id: string, enabled: boolean) {
	const item = findItem(id)
	if (item) void toggleFlow.value?.toggle(item, enabled)
}

function removeItem(id: string, event: MouseEvent) {
	const item = findItem(id)
	if (item) void deleteFlow.value?.promptDelete([item], event)
}

function updateItem(id: string) {
	const item = findItem(id)
	if (item && !item.locked) ctx.updateItem?.(item)
}

function switchVersion(id: string) {
	const item = findItem(id)
	if (item && !item.locked) ctx.switchVersion?.(item)
}

async function setEnabledFor(id: string, side: ContentSide, enabled: boolean) {
	const item = findItem(id)
	if (!item || isBusy.value || !ctx.setEnabledFor) return
	retain(id)
	await ctx.setEnabledFor(item, side, enabled)
}

function sortByProject(direction: 'asc' | 'desc') {
	sortMode.value = direction === 'desc' ? 'alphabetical-desc' : 'alphabetical-asc'
}

const sortedByName = computed(() => sortMode.value.startsWith('alphabetical'))

const shareOptions = computed<ButtonMenuOption[]>(() => {
	const share = ctx.share
	if (!share) return []
	return [
		{
			id: 'share-names',
			label: formatMessage(messages.shareProjectNames),
			action: () => share(selectedItems.value, 'names'),
		},
		{
			id: 'share-file-names',
			label: formatMessage(messages.shareFileNames),
			action: () => share(selectedItems.value, 'file-names'),
		},
		{
			id: 'share-urls',
			label: formatMessage(messages.shareProjectLinks),
			action: () => share(selectedItems.value, 'urls'),
		},
		{
			id: 'share-markdown',
			label: formatMessage(messages.shareMarkdownLinks),
			action: () => share(selectedItems.value, 'markdown'),
		},
	]
})
</script>

<template>
	<div class="flex flex-col gap-4" :class="{ 'pb-6': props.bottomPadding }">
		<template v-if="!ctx.loading.value">
			<div
				v-if="ctx.error.value"
				class="flex w-full flex-col items-center justify-center gap-4 p-4"
			>
				<div class="universal-card flex flex-col items-center gap-4 p-6">
					<h2 class="m-0 text-xl font-bold">{{ formatMessage(messages.failedToLoad) }}</h2>
					<p class="text-secondary">{{ ctx.error.value.message }}</p>
					<Button type="colored" color="brand" @click="refresh">{{
						formatMessage(commonMessages.retryButton)
					}}</Button>
				</div>
			</div>

			<template v-else>
				<ManagedContentCard
					v-if="ctx.managed.value"
					:data="ctx.managed.value.card"
					:disabled="!!ctx.managed.value.disabledReason"
					:disabled-text="ctx.managed.value.disabledReason"
					:show-view-content="!!ctx.managed.value.view"
					:show-settings="!!ctx.managed.value.openSettings"
					:show-primary-action="!!ctx.managed.value.primaryAction"
					@view-content="ctx.managed.value?.view?.()"
					@settings="ctx.managed.value?.openSettings?.()"
					@primary-action="ctx.managed.value?.primaryAction?.($event)"
					@prefetch-view-content="ctx.managed.value?.prefetchView?.()"
					@prefetch-primary-action="ctx.managed.value?.prefetchPrimaryAction?.()"
				/>

				<div v-if="ctx.items.value.length > 0" class="flex flex-col gap-4">
					<span v-if="ctx.managed.value" class="text-2xl font-semibold leading-8 text-contrast">
						{{ formatMessage(messages.additionalContent) }}
					</span>

					<ContentToolbar
						v-model:search="searchQuery"
						v-model:type="activeType"
						v-model:filters="filters"
						v-model:sort="sortMode"
						:search-placeholder="searchPlaceholder"
						:types="types"
						:add-disabled="isBusy || !ctx.canAdd.value"
						:add-tooltip="addTooltip"
						:show-update-all="!!updateAllFlow?.supported && hasOutdatedItems"
						:update-all-disabled="isBulkOperating || !!updateAllFlow?.loading"
						:refreshing="refreshing"
						:filter-categories="metadataFilterCategories"
						:show-date-sort="hasDateAdded"
						@add="ctx.browse(activeType)"
						@upload="ctx.upload()"
						@update-all="updateAllFlow?.promptUpdate(typeItems)"
						@prefetch-update-all="prefetchUpdates(typeItems)"
						@refresh="refresh"
					/>

					<ContentTable
						v-model:selected-ids="selectedIds"
						:sections="[{ id: activeType, items: rows }]"
						:columns="{
							selection: true,
							enabledFor: !!ctx.setEnabledFor,
							actions: ['update', 'switchVersion', 'toggle', 'remove'],
						}"
						:highlighted-item-id="highlightedItemId"
						:sortable="sortedByName"
						:sort-by="sortedByName ? 'project' : undefined"
						:sort-direction="sortMode === 'alphabetical-desc' ? 'desc' : 'asc'"
						@sort="(_, direction) => sortByProject(direction)"
						@toggle="toggleItem"
						@enabled-for="setEnabledFor"
						@remove="removeItem"
						@update="updateItem"
						@switch-version="switchVersion"
						@prefetch="prefetchRow"
					>
						<template #empty>
							<span>{{ emptyTableMessage }}</span>
						</template>
					</ContentTable>
				</div>

				<ContentEmptyState
					v-else
					:heading="
						formatMessage(
							ctx.managed.value ? messages.noExtraContentInstalled : messages.noContentInstalled,
						)
					"
					:description="
						formatMessage(ctx.managed.value ? messages.emptyModpackHint : messages.emptyHint)
					"
					:add-disabled="isBusy || !ctx.canAdd.value"
					:add-tooltip="addTooltip"
					@add="ctx.browse(activeType)"
					@upload="ctx.upload()"
				/>
			</template>
		</template>

		<ContentSelectionBar
			v-if="!ctx.updatesRunInBackground || bulkOperation !== 'update'"
			:selected-items="
				bulkOperation === 'update' ? (updateAllFlow?.bulkUpdateItems ?? []) : selectedItems
			"
			:content-type-label="activeType"
			:is-busy="isBusy"
			:busy-tooltip="busyMessage"
			:is-bulk-operating="isBulkOperating"
			:bulk-operation="bulkOperation"
			:bulk-progress="bulkProgress"
			:bulk-total="bulkTotal"
			:bulk-waiting="bulkWaiting"
			:bulk-item-count="updateAllFlow?.bulkItemCount ?? 0"
			:aria-label="formatMessage(commonMessages.selectionActionsLabel)"
			:get-item-id="(item) => item.id"
			:toggle-items="toggleableSelectedItems"
			@clear="clearSelection"
			@enable="toggleFlow?.enableItems(toggleableSelectedItems)"
			@disable="toggleFlow?.disableEnabledItems(toggleableSelectedItems)"
		>
			<template #actions>
				<Button
					v-if="
						updateAllFlow?.supported &&
						selectedItems.some((item) => item.has_update && !item.locked)
					"
					v-tooltip="formatMessage(commonMessages.updateButton)"
					type="quiet"
					color="green"
					:disabled="!!updateAllFlow?.loading"
					class="hover:!bg-green focus-visible:!bg-green hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="updateAllFlow?.promptUpdate(selectedItems)"
					@mouseenter="prefetchUpdates(selectedItems)"
					@focus="prefetchUpdates(selectedItems)"
				>
					<DownloadIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.updateButton) }}</span>
				</Button>

				<TeleportOverflowMenu
					v-if="ctx.share"
					type="quiet"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					:options="shareOptions"
					class="!w-auto !px-2.5 !rounded-xl"
				>
					<ShareIcon />
					<span class="bar-label">{{ formatMessage(messages.share) }}</span>
					<DropdownIcon />
					<template #share-names>
						<TextCursorInputIcon />
						{{ formatMessage(messages.shareProjectNames) }}
					</template>
					<template #share-file-names>
						<FileIcon />
						{{ formatMessage(messages.shareFileNames) }}
					</template>
					<template #share-urls>
						<LinkIcon />
						{{ formatMessage(messages.shareProjectLinks) }}
					</template>
					<template #share-markdown>
						<CodeIcon />
						{{ formatMessage(messages.shareMarkdownLinks) }}
					</template>
				</TeleportOverflowMenu>
			</template>

			<template #actions-end>
				<div v-if="removableSelectedItems.length > 0" class="mx-1 h-6 w-px bg-surface-5" />

				<Button
					v-if="removableSelectedItems.length > 0"
					v-tooltip="formatMessage(commonMessages.deleteLabel)"
					type="quiet"
					color="red"
					class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
					@click="deleteFlow?.promptDelete([...selectedItems], $event)"
				>
					<TrashIcon />
					<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
				</Button>
			</template>
		</ContentSelectionBar>

		<ToggleFlow ref="toggleFlow" :state="flowState" />
		<DeleteFlow
			ref="deleteFlow"
			:state="flowState"
			:to-display-row="toDisplayRow"
			:disable-items="(items) => toggleFlow?.disableItems(items) ?? Promise.resolve()"
		/>
		<UpdateAllFlow ref="updateAllFlow" :state="flowState" />
		<ConfirmUnlinkModal
			v-if="ctx.managed.value?.unlink"
			ref="confirmUnlinkModal"
			:server="isServer"
			:backup-tip="ctx.managed.value.card.manager.name"
			:action-disabled="isBusy"
			:action-disabled-tooltip="busyMessage"
			@unlink="ctx.managed.value?.unlink?.()"
		/>

		<slot name="modals" />
	</div>
</template>
