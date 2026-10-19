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
				:mode="mode"
			>
				<template #toolbar>
					<slot name="toolbar" />
				</template>
				<template v-if="slots['toolbar-end']" #toolbar-end>
					<slot name="toolbar-end" />
				</template>
				<template #panel-tree>
					<FileSideBar fill-height :add-border="false" />
				</template>
				<template #panel-details>
					<DetailsPanel>
						<template v-if="slots['project-details']" #source>
							<slot name="project-details" />
						</template>
					</DetailsPanel>
				</template>
				<template #panel-logs>
					<LogsPanel />
				</template>
				<template #panel-settings>
					<SettingsPanel />
				</template>
			</FileWorkbench>
		</template>
	</FilePageLayout>
</template>

<script setup lang="ts">
import { FolderOpenIcon, InfoIcon, SettingsIcon, TerminalSquareIcon } from '@modrinth/assets'
import { computed, ref, type VNode, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { RootZipTreeNode, ZipHandler } from '#ui/helpers/toolbox/zip'
import FileSideBar from '#ui/layouts/shared/files-tab/components/FileSideBar.vue'
import FileWorkbench from '#ui/layouts/shared/files-tab/components/FileWorkbench.vue'
import FilePageLayout from '#ui/layouts/shared/files-tab/layout.vue'
import { provideFileManager } from '#ui/layouts/shared/files-tab/providers/file-manager'
import {
	WORKBENCH_HOSTS,
	WORKBENCH_MODES,
	type WorkbenchMode,
	type WorkbenchPanel,
	type WorkbenchRegion,
} from '#ui/layouts/shared/workbench/types'
import { commonMessages } from '#ui/utils/common-messages'

import { useArchiveFileManager } from '../composables/archive-file-manager'
import { provideArchiveViewer } from '../providers/archive-viewer'
import { SETTINGS_PANEL_ID } from '../settings'
import type { ArchiveSource } from '../types'
import DetailsPanel from './panels/details/index.vue'
import LogsPanel from './panels/LogsPanel.vue'
import SettingsPanel from './panels/settings-panel.vue'

const props = defineProps<{
	root: RootZipTreeNode
	handler: ZipHandler
	source: ArchiveSource
	/** Path within the archive, without a leading slash. */
	path?: string | null
	/** Where the layout is persisted; `null` keeps it in memory only. */
	storageKey: string | null
	hiddenRegions?: WorkbenchRegion[]
	/** An empty page of the same origin, which lets panels be popped out into windows loading it. */
	popoutUrl?: string
}>()

const emit = defineEmits<{
	navigate: [path: string]
}>()

const slots = defineSlots<{
	toolbar?: () => VNode[]
	'toolbar-end'?: () => VNode[]
	'project-details'?: () => VNode[]
}>()

const { formatMessage } = useVIntl()

/** Every panel can be arranged freely, and popped out of any dock when the host has a page for it. */
const mode: WorkbenchMode = {
	...WORKBENCH_MODES.explorer,
	popouts: props.popoutUrl ? { url: props.popoutUrl, docks: [...WORKBENCH_HOSTS] } : undefined,
}

const messages = defineMessages({
	tree: { id: 'archive-viewer.panel.tree', defaultMessage: 'Files' },
	details: { id: 'archive-viewer.panel.details', defaultMessage: 'Details' },
	logs: { id: 'archive-viewer.panel.logs', defaultMessage: 'Logs' },
})

const { context, location, open, fullyLoaded, treeVersion, sha512Hash, archiveHash, nodeAt } =
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
	archiveHash,
	nodeAt,
})

const panels = computed<WorkbenchPanel[]>(() => [
	{ id: 'tree', title: formatMessage(messages.tree), region: 'left', icon: FolderOpenIcon },
	{ id: 'details', title: formatMessage(messages.details), region: 'right', icon: InfoIcon },
	{ id: 'logs', title: formatMessage(messages.logs), region: 'bottom', icon: TerminalSquareIcon },
	{
		id: SETTINGS_PANEL_ID,
		title: formatMessage(commonMessages.settingsLabel),
		region: 'right',
		icon: SettingsIcon,
		padded: true,
		closedByDefault: true,
	},
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
