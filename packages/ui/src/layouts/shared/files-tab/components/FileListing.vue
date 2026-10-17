<template>
	<div ref="root" class="flex flex-col">
		<FileManagerError
			v-if="view.error.value"
			class="rounded-b-[20px]"
			:title="formatMessage(messages.errorTitle)"
			:message="formatMessage(messages.errorMessage)"
			@refetch="ui.refresh"
			@home="emit('navigate', HOME)"
		/>
		<div v-else @contextmenu="openFolderMenu">
			<FileUploadDragAndDrop
				ref="fileUploadRef"
				:disabled="ui.isBusy.value"
				@drop-error="ui.handleDropError"
				@files-dropped="ui.uploadFiles"
			>
				<FileTableHeader
					:sort-field="view.sortField.value"
					:sort-desc="view.sortDesc.value"
					:all-selected="view.allSelected.value"
					:some-selected="view.someSelected.value"
					:is-stuck="isLabelBarStuck"
					:columns="shownColumns"
					:enabled-columns="enabledColumns"
					:details-enabled="detailsEnabled"
					:columns-adjustable="ui.advancedView.value"
					:view-settings-enabled="ui.canUseAdvancedView.value && !ui.inWorkbench"
					:advanced-view="ui.advancedView.value"
					:colored-icons="ui.coloredIcons.value"
					@sort="view.handleSort"
					@toggle-all="view.toggleSelectAll"
					@toggle-column="toggleColumn"
					@toggle-details="detailsEnabled = !detailsEnabled"
					@toggle-advanced-view="ui.setAdvancedView(!ui.advancedView.value)"
					@toggle-colored-icons="ui.setColoredIcons(!ui.coloredIcons.value)"
				/>
				<ReadyTransition :pending="listingPending">
					<div
						v-if="filteredItems.length > 0"
						ref="virtualListContainer"
						class="relative w-full"
						:style="{ minHeight: `${totalHeight}px`, overflowAnchor: 'none' }"
					>
						<div class="absolute w-full" :style="{ top: `${visibleTop}px` }">
							<FileRow
								v-for="(item, idx) in visibleItems"
								:key="item.path"
								:class="`h-[${itemHeight}]`"
								:file="item"
								:index="visibleRange.start + idx"
								:is-last="visibleRange.start + idx === filteredItems.length - 1"
								:columns="shownColumns"
								@navigate="(file) => emit('navigate', file)"
								@hover="ui.handleItemPrefetch"
							/>
						</div>
					</div>
					<div
						v-else-if="view.items.value.length === 0 && !view.loading.value"
						class="flex h-full w-full items-center justify-center rounded-b-[20px] bg-surface-2 p-20"
					>
						<div class="flex flex-col items-center gap-4 text-center">
							<FolderOpenIcon class="h-16 w-16 text-secondary" />
							<h3 class="m-0 text-2xl font-bold text-contrast">
								{{ formatMessage(messages.emptyFolderTitle) }}
							</h3>
							<p class="m-0 text-sm text-secondary">
								{{ formatMessage(messages.emptyFolderDescription) }}
							</p>
						</div>
					</div>
				</ReadyTransition>
			</FileUploadDragAndDrop>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FolderOpenIcon } from '@modrinth/assets'
import { useElementSize } from '@vueuse/core'
import { computed, ref, toRef } from 'vue'

import ReadyTransition from '#ui/components/base/ReadyTransition.vue'
import { defineMessages, useVIntl } from '#ui/composables'
import { useStickyObserver } from '#ui/composables/sticky-observer'
import { useVirtualScroll } from '#ui/composables/virtual-scroll.ts'

import type { DirectoryView } from '../composables/directory-view'
import { useEntryMenu } from '../composables/entry-menu'
import { useFileColumns } from '../composables/file-columns'
import { injectFileBrowserUI } from '../providers/file-browser-ui'
import type { FileInfo } from '../providers/file-manager'
import { infoFrom } from '../utils'
import FileManagerError from './FileManagerError.vue'
import FileRow from './FileRow.vue'
import FileTableHeader from './FileTableHeader.vue'
import FileUploadDragAndDrop from './upload/FileUploadDragAndDrop.vue'

const props = withDefaults(
	defineProps<{
		view: DirectoryView
		/** How far below the top of the scroll area the table header sticks, in px. */
		headerStickyTop?: number
	}>(),
	{ headerStickyTop: 0 },
)

const emit = defineEmits<{
	navigate: [file: FileInfo]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	emptyFolderTitle: {
		id: 'files.layout.empty-folder-title',
		defaultMessage: 'This folder is empty',
	},
	emptyFolderDescription: {
		id: 'files.layout.empty-folder-description',
		defaultMessage: 'There are no files or folders.',
	},
	errorTitle: {
		id: 'files.layout.error-title',
		defaultMessage: 'Unable to load files',
	},
	errorMessage: {
		id: 'files.layout.error-message',
		defaultMessage: 'The folder may not exist.',
	},
})

const HOME = infoFrom('/')

const ui = injectFileBrowserUI()
const { openMenu } = useEntryMenu()

/** Right-click on the folder's empty space (rows open their own menu and prevent this one). */
function openFolderMenu(event: MouseEvent) {
	if (event.defaultPrevented) return
	openMenu(event, props.view.directory.value)
}

const filteredItems = computed(() => props.view.filteredItems.value)

/** Only the listing fades while a directory loads for the first time; the rest stays put. */
const listingPending = computed(
	() => props.view.loading.value && props.view.items.value.length === 0,
)

const {
	listContainer: virtualListContainer,
	totalHeight,
	itemHeight,
	visibleRange,
	visibleTop,
	visibleItems,
	syncScrollState,
} = useVirtualScroll(filteredItems, {
	itemHeight: 3.25,
	itemUnit: 'rem',
	bufferSize: 5,
})

const root = ref<HTMLElement | null>(null)
const { width } = useElementSize(root)
const containerWidth = computed(() => (width.value > 0 ? width.value : undefined))

const { detailsEnabled, enabledColumns, shownColumns, toggleColumn } = useFileColumns(
	containerWidth,
	ui.advancedView,
)

const fileUploadRef = ref<InstanceType<typeof FileUploadDragAndDrop>>()
const fileUploadEl = computed(() => fileUploadRef.value?.$el as HTMLElement | null)
const { isStuck: isLabelBarStuck } = useStickyObserver(fileUploadEl, undefined, {
	topOffset: toRef(() => props.headerStickyTop),
})

defineExpose({ syncScrollState })
</script>
