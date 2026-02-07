<script setup lang="ts">
import {
	CodeIcon,
	CompassIcon,
	DownloadIcon,
	DropdownIcon,
	EmptyIllustration,
	FileIcon,
	FilterIcon,
	FolderOpenIcon,
	LinkIcon,
	PowerIcon,
	PowerOffIcon,
	RefreshCwIcon,
	SearchIcon,
	ShareIcon,
	SpinnerIcon,
	TextCursorInputIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import { formatProjectType } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { useBulkOperation } from '../../composables/content/bulk-operations'
import { useChangingItems } from '../../composables/content/changing-items'
import { useContentFilters } from '../../composables/content/content-filtering'
import { useContentSearch } from '../../composables/content/content-search'
import { useContentSelection } from '../../composables/content/content-selection'
import { useStickyObserver } from '../../composables/sticky-observer'
import { injectContentManager } from '../../providers/content-manager'
import ButtonStyled from '../base/ButtonStyled.vue'
import FloatingActionBar from '../base/FloatingActionBar.vue'
import OverflowMenu from '../base/OverflowMenu.vue'
import ProgressBar from '../base/ProgressBar.vue'
import ContentCardTable from './ContentCardTable.vue'
import ContentModpackCard from './ContentModpackCard.vue'
import ConfirmBulkUpdateModal from './modals/ConfirmBulkUpdateModal.vue'
import ConfirmDeletionModal from './modals/ConfirmDeletionModal.vue'
import ConfirmUnlinkModal from './modals/ConfirmUnlinkModal.vue'
import type { ContentCardTableItem, ContentItem } from './types'

const ctx = injectContentManager()

const sortedItems = computed(() => {
	const items = [...ctx.items.value]
	return items.sort((a, b) => {
		const nameA = a.project?.title ?? a.file_name
		const nameB = b.project?.title ?? b.file_name
		return nameA.toLowerCase().localeCompare(nameB.toLowerCase())
	})
})

const { searchQuery, search } = useContentSearch(sortedItems, [
	'project.title',
	'owner.name',
	'file_name',
])

const { selectedFilters, filterOptions, toggleFilter, applyFilters } = useContentFilters(
	ctx.items,
	{
		showTypeFilters: true,
		showUpdateFilter: ctx.hasUpdateSupport,
		isPackLocked: ctx.isPackLocked,
		formatProjectType,
	},
)

const { selectedIds, selectedItems, clearSelection, removeFromSelection } = useContentSelection(
	ctx.items,
	ctx.getItemId,
)

const { isBulkOperating, bulkProgress, bulkTotal, bulkOperation, runBulk } = useBulkOperation()

const { isChanging, markChanging, unmarkChanging } = useChangingItems()

const contentTableRef = ref<InstanceType<typeof ContentCardTable> | null>(null)
const contentTableEl = computed(() => contentTableRef.value?.$el as HTMLElement | null)
const { isStuck: isTableStuck } = useStickyObserver(contentTableEl)

const refreshing = ref(false)
async function handleRefresh() {
	if (refreshing.value) return
	refreshing.value = true
	try {
		await ctx.refresh()
	} finally {
		refreshing.value = false
	}
}

const filteredItems = computed(() => applyFilters(search(sortedItems.value)))
const tableItems = computed<ContentCardTableItem[]>(() =>
	filteredItems.value.map((item) => {
		const base = ctx.mapToTableItem(item)
		return {
			...base,
			disabled: isChanging(base.id) || ctx.isBusy.value,
			hasUpdate: !ctx.isPackLocked.value && item.has_update,
			overflowOptions: ctx.getOverflowOptions?.(item),
		}
	}),
)

const hasOutdatedProjects = computed(() => ctx.items.value.some((p) => p.has_update))

//  Deletion
const pendingDeletionItems = ref<ContentItem[]>([])
const confirmDeletionModal = ref<InstanceType<typeof ConfirmDeletionModal>>()

function handleDeleteById(id: string) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (item) {
		pendingDeletionItems.value = [item]
		confirmDeletionModal.value?.show()
	}
}

function showBulkDeleteModal() {
	pendingDeletionItems.value = [...selectedItems.value]
	confirmDeletionModal.value?.show()
}

async function confirmDelete() {
	const itemsToDelete = [...pendingDeletionItems.value]
	pendingDeletionItems.value = []
	if (itemsToDelete.length === 0) return

	if (itemsToDelete.length === 1) {
		const item = itemsToDelete[0]
		const id = ctx.getItemId(item)
		markChanging(id)
		await ctx.deleteItem(item)
		removeFromSelection(id)
		unmarkChanging(id)
		return
	}

	await runBulk('delete', itemsToDelete, async (item) => {
		await ctx.deleteItem(item)
		removeFromSelection(ctx.getItemId(item))
	})
	clearSelection()
}

async function handleToggleEnabledById(id: string, _value: boolean) {
	const item = ctx.items.value.find((i) => ctx.getItemId(i) === id)
	if (!item) return
	markChanging(id)
	await ctx.toggleEnabled(item)
	unmarkChanging(id)
}

async function bulkEnable() {
	const items = selectedItems.value.filter((item) => !item.enabled)
	if (items.length === 0) return
	await runBulk('enable', items, (item) => ctx.toggleEnabled(item))
	clearSelection()
}

async function bulkDisable() {
	const items = selectedItems.value.filter((item) => item.enabled)
	if (items.length === 0) return
	await runBulk('disable', items, (item) => ctx.toggleEnabled(item))
	clearSelection()
}

function handleUpdateById(id: string) {
	ctx.updateItem?.(id)
}

// Bulk updating
const confirmBulkUpdateModal = ref<InstanceType<typeof ConfirmBulkUpdateModal>>()
const pendingBulkUpdateItems = ref<ContentItem[]>([])

function promptUpdateAll() {
	if (!ctx.bulkUpdateItem) return
	const items = ctx.items.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	confirmBulkUpdateModal.value?.show()
}

function promptUpdateSelected() {
	if (!ctx.bulkUpdateItem) return
	const items = selectedItems.value.filter((item) => item.has_update)
	if (items.length === 0) return
	pendingBulkUpdateItems.value = items
	confirmBulkUpdateModal.value?.show()
}

async function confirmBulkUpdate() {
	const items = pendingBulkUpdateItems.value
	if (items.length === 0 || !ctx.bulkUpdateItem) return

	await runBulk('update', items, ctx.bulkUpdateItem)
	clearSelection()
	pendingBulkUpdateItems.value = []
}

const confirmUnlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
</script>

<template>
	<div class="flex flex-col gap-4">
		<div
			v-if="ctx.loading.value"
			class="flex min-h-[50vh] w-full flex-col items-center justify-center gap-2 text-center text-secondary"
		>
			<SpinnerIcon class="animate-spin" />
			Loading content...
		</div>

		<div
			v-else-if="ctx.error.value"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="universal-card flex flex-col items-center gap-4 p-6">
				<h2 class="m-0 text-xl font-bold">Failed to load content</h2>
				<p class="text-secondary">{{ ctx.error.value.message }}</p>
				<ButtonStyled color="brand">
					<button @click="handleRefresh">Retry</button>
				</ButtonStyled>
			</div>
		</div>

		<template v-else>
			<ContentModpackCard
				v-if="ctx.modpack.value"
				:project="ctx.modpack.value.project"
				:project-link="ctx.modpack.value.projectLink"
				:version="ctx.modpack.value.version"
				:version-link="ctx.modpack.value.versionLink"
				:owner="ctx.modpack.value.owner"
				:categories="ctx.modpack.value.categories"
				:has-update="ctx.modpack.value.hasUpdate"
				:disabled="ctx.modpack.value.disabled || ctx.isBusy.value"
				:disabled-text="ctx.modpack.value.disabledText"
				@update="ctx.updateModpack?.()"
				@content="ctx.viewModpackContent?.()"
				@unlink="ctx.unlinkModpack ? confirmUnlinkModal?.show() : undefined"
			/>

			<template v-if="ctx.items.value.length > 0">
				<span v-if="ctx.modpack.value" class="text-xl font-semibold text-contrast">
					Additional content
				</span>

				<div class="flex flex-col gap-2 lg:flex-row lg:items-center">
					<div class="iconified-input flex-1">
						<SearchIcon aria-hidden="true" class="text-lg" />
						<input
							v-model="searchQuery"
							class="!h-10"
							autocomplete="off"
							spellcheck="false"
							type="text"
							:placeholder="`Search ${ctx.items.value.length} ${ctx.contentTypeLabel.value}${ctx.items.value.length === 1 ? '' : 's'}...`"
						/>
						<ButtonStyled v-if="searchQuery" circular type="transparent" class="r-btn">
							<button @click="searchQuery = ''">
								<XIcon />
							</button>
						</ButtonStyled>
					</div>

					<div class="flex gap-2">
						<ButtonStyled color="brand">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 flex items-center gap-2"
								@click="ctx.browse"
							>
								<CompassIcon class="size-5" />
								<span>Browse content</span>
							</button>
						</ButtonStyled>
						<ButtonStyled type="outlined">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 !border-button-bg !border-[1px]"
								@click="ctx.uploadFiles"
							>
								<FolderOpenIcon class="size-5" />
								Upload files
							</button>
						</ButtonStyled>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<div
						class="flex flex-col justify-between gap-2 min-[1200px]:flex-row min-[1200px]:items-center"
					>
						<div class="flex flex-wrap items-center gap-1.5">
							<FilterIcon class="size-5 text-secondary" />
							<button
								class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
								:class="
									selectedFilters.length === 0
										? 'border-green bg-brand-highlight text-brand'
										: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
								"
								@click="selectedFilters = []"
							>
								All
							</button>
							<button
								v-for="option in filterOptions"
								:key="option.id"
								class="cursor-pointer rounded-full border border-solid px-3 py-1.5 text-base font-semibold leading-5 transition-all duration-100 active:scale-[0.97]"
								:class="
									selectedFilters.includes(option.id)
										? 'border-green bg-brand-highlight text-brand'
										: 'border-surface-5 bg-surface-4 text-primary hover:bg-surface-5'
								"
								@click="toggleFilter(option.id)"
							>
								{{ option.label }}
							</button>
						</div>

						<div class="flex items-center gap-2">
							<ButtonStyled
								v-if="ctx.bulkUpdateItem && !ctx.isPackLocked.value && hasOutdatedProjects"
								color="green"
								type="transparent"
								color-fill="text"
								hover-color-fill="background"
							>
								<button :disabled="isBulkOperating || ctx.isBusy.value" @click="promptUpdateAll">
									<DownloadIcon />
									Update all
								</button>
							</ButtonStyled>

							<ButtonStyled type="transparent" hover-color-fill="none">
								<button :disabled="refreshing || ctx.isBusy.value" @click="handleRefresh">
									<RefreshCwIcon :class="refreshing ? 'animate-spin' : ''" />
									Refresh
								</button>
							</ButtonStyled>
						</div>
					</div>

				<ContentCardTable
					ref="contentTableRef"
					v-model:selected-ids="selectedIds"
					:items="tableItems"
					:show-selection="true"
					:is-stuck="isTableStuck"
					@update:enabled="handleToggleEnabledById"
					@delete="handleDeleteById"
					@update="handleUpdateById"
				>
					<template #empty>
						<span>No content found.</span>
					</template>
				</ContentCardTable>
				</div>
			</template>

			<div v-else class="mx-auto flex flex-col justify-center gap-8 p-6 text-center">
				<EmptyIllustration class="h-[80px] w-auto" />
				<div class="flex flex-col gap-4" :class="ctx.modpack.value ? '' : '-mt-4'">
					<div class="flex flex-col items-center gap-1.5">
						<span class="text-2xl font-semibold text-contrast">
							{{ ctx.modpack.value ? 'No extra content added' : 'Your instance is empty' }}
						</span>
						<span class="text-primary">
							{{
								ctx.modpack.value
									? 'You can add content on top of a modpack!'
									: 'Add some content to bring it to life!'
							}}
						</span>
					</div>
					<div class="mx-auto flex gap-2">
						<ButtonStyled type="outlined">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 !border-button-bg !border-[1px]"
								@click="ctx.uploadFiles"
							>
								<FolderOpenIcon class="size-5" />
								Upload files
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button
								:disabled="ctx.isBusy.value"
								class="!h-10 flex items-center gap-2"
								@click="ctx.browse"
							>
								<CompassIcon class="size-5" />
								<span>Browse content</span>
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</template>

		<FloatingActionBar :shown="selectedItems.length > 0 || isBulkOperating">
			<template v-if="!isBulkOperating">
				<div class="flex items-center gap-0.5">
					<span class="px-4 py-2.5 text-base font-semibold text-contrast">
						{{ selectedItems.length }}
						{{ ctx.contentTypeLabel.value }}{{ selectedItems.length === 1 ? '' : 's' }} selected
					</span>
					<div class="mx-1 h-6 w-px bg-surface-5" />
					<ButtonStyled type="transparent">
						<button class="!text-primary" @click="clearSelection">Clear</button>
					</ButtonStyled>
				</div>

				<div class="ml-auto flex items-center gap-0.5">
					<ButtonStyled
						v-if="
							ctx.bulkUpdateItem &&
							!ctx.isPackLocked.value &&
							selectedItems.some((m) => m.has_update)
						"
						type="transparent"
						color="green"
						color-fill="text"
						hover-color-fill="background"
					>
						<button :disabled="ctx.isBusy.value" @click="promptUpdateSelected">
							<DownloadIcon />
							Update
						</button>
					</ButtonStyled>

					<ButtonStyled v-if="ctx.shareItems" type="transparent">
						<OverflowMenu
							:options="[
								{
									id: 'share-names',
									action: () => ctx.shareItems!(selectedItems, 'names'),
								},
								{
									id: 'share-file-names',
									action: () => ctx.shareItems!(selectedItems, 'file-names'),
								},
								{
									id: 'share-urls',
									action: () => ctx.shareItems!(selectedItems, 'urls'),
								},
								{
									id: 'share-markdown',
									action: () => ctx.shareItems!(selectedItems, 'markdown'),
								},
							]"
						>
							<ShareIcon />
							Share
							<DropdownIcon />
							<template #share-names>
								<TextCursorInputIcon />
								Project names
							</template>
							<template #share-file-names>
								<FileIcon />
								File names
							</template>
							<template #share-urls>
								<LinkIcon />
								Project links
							</template>
							<template #share-markdown>
								<CodeIcon />
								Markdown links
							</template>
						</OverflowMenu>
					</ButtonStyled>

					<ButtonStyled v-if="selectedItems.every((m) => !m.enabled)" type="transparent">
						<button :disabled="ctx.isBusy.value" @click="bulkEnable">
							<PowerIcon />
							Enable
						</button>
					</ButtonStyled>
					<ButtonStyled v-else type="transparent">
						<button :disabled="ctx.isBusy.value" @click="bulkDisable">
							<PowerOffIcon />
							Disable
						</button>
					</ButtonStyled>

					<div class="mx-1 h-6 w-px bg-surface-5" />

					<ButtonStyled
						type="transparent"
						color="red"
						color-fill="text"
						hover-color-fill="background"
					>
						<button :disabled="ctx.isBusy.value" @click="showBulkDeleteModal">
							<TrashIcon />
							Delete
						</button>
					</ButtonStyled>
				</div>
			</template>

			<template v-else>
				<div class="flex flex-1 flex-col gap-2">
					<span class="text-sm font-medium text-contrast">
						{{
							bulkOperation === 'enable'
								? 'Enabling'
								: bulkOperation === 'disable'
									? 'Disabling'
									: bulkOperation === 'update'
										? 'Updating'
										: 'Deleting'
						}}
						content... ({{ bulkProgress }}/{{ bulkTotal }})
					</span>
					<ProgressBar full-width :progress="bulkProgress" :max="bulkTotal" color="brand" />
				</div>
			</template>
		</FloatingActionBar>

		<ConfirmDeletionModal
			ref="confirmDeletionModal"
			:count="pendingDeletionItems.length"
			:item-type="ctx.contentTypeLabel.value"
			@delete="confirmDelete"
		/>
		<ConfirmBulkUpdateModal
			v-if="ctx.bulkUpdateItem"
			ref="confirmBulkUpdateModal"
			:count="pendingBulkUpdateItems.length"
			@update="confirmBulkUpdate"
		/>
		<ConfirmUnlinkModal
			v-if="ctx.unlinkModpack"
			ref="confirmUnlinkModal"
			@unlink="ctx.unlinkModpack!()"
		/>

		<slot name="modals" />
	</div>
</template>
