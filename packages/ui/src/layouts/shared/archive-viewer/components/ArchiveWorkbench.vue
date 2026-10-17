<template>
	<FilePageLayout
		workbench
		compact-navigation
		:search-loaded-only-notice="!fullyLoaded"
		:sidebar="sidebar"
	>
		<template #workspace>
			<FileWorkbench
				ref="workbench"
				:panels="panels"
				:storage-key="storageKey"
				:hidden-regions="hiddenRegions"
			>
				<template #toolbar>
					<slot name="toolbar" />
				</template>
				<template #panel-tree>
					<FileSideBar fill-height :add-border="false" />
				</template>
				<template #panel-file-details>
					<FileDetailsPanel />
				</template>
				<template #panel-archive-details>
					<ArchiveDetailsPanel />
				</template>
				<template v-if="slots['project-details']" #panel-project-details>
					<slot name="project-details" />
				</template>
				<template #panel-logs>
					<LogsPanel />
				</template>
			</FileWorkbench>
		</template>
	</FilePageLayout>
</template>

<script setup lang="ts">
import {
	BoxIcon,
	FolderArchiveIcon,
	FolderOpenIcon,
	InfoIcon,
	TerminalSquareIcon,
} from '@modrinth/assets'
import { computed, ref, type VNode, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { RootZipTreeNode, ZipHandler } from '#ui/helpers/toolbox/zip'
import FileSideBar from '#ui/layouts/shared/files-tab/components/FileSideBar.vue'
import FileWorkbench from '#ui/layouts/shared/files-tab/components/FileWorkbench.vue'
import FilePageLayout from '#ui/layouts/shared/files-tab/layout.vue'
import { provideFileManager } from '#ui/layouts/shared/files-tab/providers/file-manager'
import type { WorkbenchPanel, WorkbenchRegion } from '#ui/layouts/shared/workbench/types'

import { useArchiveFileManager } from '../composables/archive-file-manager'
import { provideArchiveViewer } from '../providers/archive-viewer'
import type { ArchiveSource } from '../types'
import ArchiveDetailsPanel from './panels/ArchiveDetailsPanel.vue'
import FileDetailsPanel from './panels/FileDetailsPanel.vue'
import LogsPanel from './panels/LogsPanel.vue'

const props = defineProps<{
	root: RootZipTreeNode
	handler: ZipHandler
	source: ArchiveSource
	/** Path within the archive, without a leading slash. */
	path?: string | null
	/** Where the layout is persisted; `null` keeps it in memory only. */
	storageKey: string | null
	hiddenRegions?: WorkbenchRegion[]
}>()

const emit = defineEmits<{
	navigate: [path: string]
}>()

const slots = defineSlots<{
	toolbar?: () => VNode[]
	'project-details'?: () => VNode[]
}>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	tree: { id: 'archive-viewer.panel.tree', defaultMessage: 'Files' },
	fileDetails: { id: 'archive-viewer.panel.file-details', defaultMessage: 'Details' },
	archiveDetails: { id: 'archive-viewer.panel.archive-details', defaultMessage: 'Archive' },
	projectDetails: { id: 'archive-viewer.panel.project-details', defaultMessage: 'Project' },
	logs: { id: 'archive-viewer.panel.logs', defaultMessage: 'Logs' },
})

const { context, location, open, fullyLoaded, treeVersion, sha512Hash, nodeAt } =
	useArchiveFileManager({
		root: props.root,
		handler: props.handler,
		source: props.source,
		initialPath: props.path,
	})

provideFileManager(context)
provideArchiveViewer({
	root: props.root,
	handler: props.handler,
	source: props.source,
	fullyLoaded,
	treeVersion,
	sha512Hash,
	nodeAt,
})

const panels = computed<WorkbenchPanel[]>(() => [
	{ id: 'tree', title: formatMessage(messages.tree), region: 'left', icon: FolderOpenIcon },
	...(slots['project-details']
		? [
				{
					id: 'project-details',
					title: formatMessage(messages.projectDetails),
					region: 'right',
					icon: BoxIcon,
				} satisfies WorkbenchPanel,
			]
		: []),
	{
		id: 'file-details',
		title: formatMessage(messages.fileDetails),
		region: 'right',
		icon: InfoIcon,
	},
	{
		id: 'archive-details',
		title: formatMessage(messages.archiveDetails),
		region: 'right',
		icon: FolderArchiveIcon,
	},
	{ id: 'logs', title: formatMessage(messages.logs), region: 'bottom', icon: TerminalSquareIcon },
])

const workbench = ref<InstanceType<typeof FileWorkbench>>()

const treePanel = computed(() => panels.value.find((panel) => panel.id === 'tree')!)

/**
 * The files layout's sidebar toggle shows and hides the tree, wherever it was moved to: its
 * region, or the tree itself when it was moved into the center.
 */
const sidebar = {
	isOpen: () => {
		const current = workbench.value?.layout
		if (!current) return true
		const host = current.regionOf(treePanel.value)
		return current.isOpen('tree') && (host === 'center' || current.visible[host].value)
	},
	setOpen: (open: boolean) => {
		const current = workbench.value?.layout
		if (!current) return
		const host = current.regionOf(treePanel.value)
		if (open) current.revealPanel('tree')
		else if (host === 'center') current.closePanel('tree')
		else current.setRegionVisible(host, false)
	},
}

watch(location, (value) => emit('navigate', value.path.replace(/^\/+/, '')))

watch(
	() => props.path,
	(path) => open(path ?? ''),
)
</script>
