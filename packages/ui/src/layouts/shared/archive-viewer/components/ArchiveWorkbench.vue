<template>
	<FilePageLayout
		workbench
		:workspace="workspace"
		:default-browsing="defaultBrowsing"
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
				<template v-for="panel in ARCHIVE_PANEL_IDS" :key="panel" #[`panel-${panel}`]>
					<ArchivePanel :panel="panel">
						<template v-if="slots['project-details']" #project-details>
							<slot name="project-details" />
						</template>
						<template v-if="slots.empty" #empty>
							<slot name="empty" />
						</template>
					</ArchivePanel>
				</template>
			</FileWorkbench>
		</template>
	</FilePageLayout>
</template>

<script setup lang="ts">
import { computed, ref, type VNode } from 'vue'

import FileWorkbench from '#ui/layouts/shared/files-tab/components/FileWorkbench.vue'
import FilePageLayout from '#ui/layouts/shared/files-tab/layout.vue'
import {
	type FilesBrowsing,
	useFilesWorkspaceSettings,
} from '#ui/layouts/shared/files-tab/settings'
import {
	WORKBENCH_HOSTS,
	WORKBENCH_MODES,
	type WorkbenchMode,
	type WorkbenchRegion,
} from '#ui/layouts/shared/workbench/types'

import { ARCHIVE_PANEL_IDS, useArchivePanels } from '../panels'
import { injectArchiveSession } from '../providers/archive-session'
import { SETTINGS_PANEL_ID } from '../settings'
import ArchivePanel from './ArchivePanel.vue'

/**
 * The archive viewer's own workbench, laying out its panels around the archive's file tabs.
 * Placed within an `ArchiveSession`, whose archive it shows.
 */
const props = withDefaults(
	defineProps<{
		/** Where the layout is persisted; `null` keeps it in memory only. */
		storageKey: string | null
		hiddenRegions?: WorkbenchRegion[]
		/** An empty page of the same origin, which lets panels be popped out into windows loading it. */
		popoutUrl?: string
		/** Names the place the archive is browsed, which keeps its own files settings. */
		workspace?: string
		/** How folders are browsed there until the user picks otherwise. */
		defaultBrowsing?: FilesBrowsing
		/** Offers the archive viewer's settings as a panel, for hosts without settings of their own. */
		settingsPanel?: boolean
	}>(),
	{ workspace: 'archive-viewer', defaultBrowsing: 'listing', settingsPanel: true },
)

const slots = defineSlots<{
	toolbar?: () => VNode[]
	'toolbar-end'?: () => VNode[]
	'project-details'?: () => VNode[]
	empty?: () => VNode[]
}>()

/** Every panel can be arranged freely, and popped out of any dock when the host has a page for it. */
const mode: WorkbenchMode = {
	...WORKBENCH_MODES.explorer,
	popouts: props.popoutUrl ? { url: props.popoutUrl, docks: [...WORKBENCH_HOSTS] } : undefined,
}

const { fullyLoaded } = injectArchiveSession()
const filesSettings = useFilesWorkspaceSettings(props.workspace, {
	browsing: props.defaultBrowsing,
})

/** Folders opened in tabs of their own trees leave no need for the files sidebar. */
const panels = useArchivePanels({
	include: () =>
		ARCHIVE_PANEL_IDS.filter(
			(id) =>
				(props.settingsPanel || id !== SETTINGS_PANEL_ID) &&
				(id !== 'tree' || filesSettings.store.get(filesSettings.browsing) !== 'tree'),
		),
})

const workbench = ref<InstanceType<typeof FileWorkbench>>()

const treePanel = computed(() => panels.value.find((panel) => panel.id === 'tree'))

/**
 * The files layout's sidebar toggle shows and hides the tree, wherever it was moved to: its
 * region, or the tree itself when it was moved into the center.
 */
const sidebar = {
	isOpen: () => {
		const current = workbench.value?.layout
		if (!current) return true
		if (!treePanel.value) return false
		const host = current.regionOf(treePanel.value)
		return current.isOpen('tree') && (host === 'center' || current.visible[host].value)
	},
	setOpen: (open: boolean) => {
		const current = workbench.value?.layout
		if (!current || !treePanel.value) return
		const host = current.regionOf(treePanel.value)
		if (open) current.revealPanel('tree')
		else if (host === 'center') current.closePanel('tree')
		else current.setRegionVisible(host, false)
	},
}
</script>
