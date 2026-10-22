<template>
	<WorkbenchLayout class="min-w-0 flex-1" :workbench="workbench">
		<template v-for="panel in reviewPanels" :key="panel.id" #[`panel-${panel.id}`]>
			<ProjectReviewPanel :slot-name="panel.id as ProjectReviewSlot" />
		</template>
		<template v-if="nestedArchive" #panel-archive>
			<ArchiveWorkbench
				storage-key="moderation.project-review.archive"
				workspace="project-review"
				default-browsing="sidebar"
				:settings-panel="false"
			>
				<template #toolbar>
					<ArchiveProgress />
				</template>
				<template v-if="slots['archive-source']" #project-details>
					<SlotOutlet :slots="slots" name="archive-source" :slot-props="{ projectShown }" />
				</template>
			</ArchiveWorkbench>
		</template>
		<template v-for="panel in archivePanelSlots" :key="panel" #[`panel-${panel}`]>
			<ArchivePanel :panel="panel">
				<template v-if="slots['archive-source']" #project-details>
					<SlotOutlet :slots="slots" name="archive-source" :slot-props="{ projectShown }" />
				</template>
			</ArchivePanel>
		</template>
		<template #tab="{ panel }">
			<template v-for="(member, index) in panel.members ?? [panel]" :key="member.id">
				<span v-if="index" class="text-secondary" aria-hidden="true">·</span>
				<component
					:is="member.icon"
					v-if="member.icon"
					class="size-4 shrink-0"
					aria-hidden="true"
				/>
				<ProjectReviewTabLabel
					v-if="!(ARCHIVE_TAB_IDS as readonly string[]).includes(member.id)"
					:tab="member.id as ProjectReviewSlot"
				/>
				<template v-else>
					<span>{{ member.title }}</span>
					<KbdChip
						v-if="member.id === filesPanel"
						keybind="review-panel-files"
						class="relative bottom-px"
					/>
				</template>
			</template>
		</template>
		<template v-if="slots.footer" #toolbar>
			<SlotOutlet :slots="slots" name="footer" />
		</template>
	</WorkbenchLayout>
	<FloatingWindow
		v-if="slots.settings"
		v-model:open="settingsWindow"
		title="Settings"
		:icon="SettingsIcon"
		storage-key="moderation.project-review.settings-window"
	>
		<template #actions>
			<IconButton
				v-tooltip="'Open as panel'"
				type="quiet"
				size="sm"
				label="Open as panel"
				@click="settingsAsPanel"
			>
				<LayoutPanelLeftIcon aria-hidden="true" />
			</IconButton>
		</template>
		<SlotOutlet :slots="slots" name="settings" :slot-props="{ floating: true }" />
	</FloatingWindow>
</template>

<script setup lang="ts">
import {
	ClipboardListIcon,
	CodeIcon,
	FileTextIcon,
	FolderArchiveIcon,
	HistoryIcon,
	ImagesIcon,
	InfoIcon,
	IssuesIcon,
	KeyIcon,
	LayoutPanelLeftIcon,
	MessagesSquareIcon,
	SettingsIcon,
	ShieldCheckIcon,
	VersionIcon,
} from '@modrinth/assets'
import {
	ARCHIVE_PANEL_IDS,
	ArchivePanel,
	type ArchivePanelId,
	ArchiveProgress,
	ArchiveWorkbench,
	createWorkbench,
	FloatingWindow,
	IconButton,
	injectFileBrowserUI,
	provideWorkbench,
	SlotOutlet,
	useArchivePanels,
	useFileDocuments,
	useFilesWorkspaceSettings,
	WORKBENCH_HOSTS,
	WORKBENCH_MODES,
	type WorkbenchHost,
	WorkbenchLayout,
	type WorkbenchPanel,
} from '@modrinth/ui'
import { type Component, computed, onScopeDispose, ref, watch, watchEffect } from 'vue'

import { useModerationKeybinds } from '~/composables/moderation'

import KbdChip from '../kdb-chip.vue'
import { injectReviewContext } from '../review-panel/context'
import {
	injectReviewShortcuts,
	reviewShortcutBlocked,
	useReviewEventListener,
	useReviewShortcut,
} from '../shortcuts'
import { provideProjectReviewContext } from './context'
import { reviewPanelLabels } from './labels'
import ProjectReviewPanel from './panel.vue'
import ProjectReviewTabLabel from './tab.vue'
import type {
	ProjectReviewSidePanel,
	ProjectReviewSlot,
	ProjectReviewSlots,
	ProjectReviewTab,
} from './types'

const POPOUT_URL = '/project-review-popout.html'

/** Panels that clip their own content rather than scrolling as a whole. */
const UNSCROLLED_SLOTS: ProjectReviewSlot[] = [
	'issues',
	're-review',
	'thread',
	'disclosures',
	'permissions',
	'gallery',
]

/**
 * Where each side panel starts: the project on the left, the rulings above the conversation
 * on the right, and the settings in the center, beside the review's tabs.
 */
const SIDE_PANEL_PLACEMENT: Record<
	ProjectReviewSidePanel,
	{ region: WorkbenchHost; group: string }
> = {
	project: { region: 'left', group: 'project' },
	rulings: { region: 'right', group: 'actions' },
	issues: { region: 'right', group: 'conversation' },
	're-review': { region: 'right', group: 'conversation' },
	thread: { region: 'right', group: 'conversation' },
	settings: { region: 'center', group: 'settings' },
}

/** Each panel's icon, shown on its tab and wherever its dock is collapsed to its tabs. */
const SLOT_ICONS: Record<ProjectReviewSlot, Component> = {
	project: InfoIcon,
	rulings: ClipboardListIcon,
	issues: IssuesIcon,
	're-review': HistoryIcon,
	thread: MessagesSquareIcon,
	settings: SettingsIcon,
	description: FileTextIcon,
	gallery: ImagesIcon,
	disclosures: ShieldCheckIcon,
	versions: VersionIcon,
	permissions: KeyIcon,
	'tech-review': CodeIcon,
}

/** The archive's panels in the review: its files and their details below the project. */
const ARCHIVE_PANELS: ArchivePanelId[] = ['tree', 'details', 'logs']

/** The panel holding the whole archive, when it's kept to a single panel. */
const ARCHIVE_PANEL = 'archive'

/** Panels of the archive, whose tabs are labelled by their title. */
const ARCHIVE_TAB_IDS = [...ARCHIVE_PANEL_IDS, ARCHIVE_PANEL]

/**
 * The review's workbench: its panels and tabs, along with the archive's panels and file tabs, or
 * the whole archive in a single panel with a layout of its own. Placed within the review's archive
 * session, and spreading the archive among its panels, within the files layout's workspace.
 */
const props = defineProps<{
	slots: ProjectReviewSlots
	/** Keeps the whole archive in a single panel, rather than among the review's panels. */
	nestedArchive: boolean
	tabs: readonly ProjectReviewTab[]
	sidePanels: readonly ProjectReviewSidePanel[]
	/** Item counts shown in the panels' tabs, e.g. how many issues there are. */
	counts?: Partial<Record<ProjectReviewSlot, number>>
	resetKey: string
	ready: boolean
}>()
const slots = props.slots
const shortcuts = injectReviewShortcuts()
const review = injectReviewContext()
const { heldTab, shortcutTab } = review
let heldTabKey: string | undefined

/** Panels only opened now and then, from the toolbar or the menus. */
const CLOSED_BY_DEFAULT: ProjectReviewSlot[] = ['settings']

function panelFor(slot: ProjectReviewSlot, region: WorkbenchHost, group?: string): WorkbenchPanel {
	return {
		closedByDefault: CLOSED_BY_DEFAULT.includes(slot),
		id: slot,
		title: reviewPanelLabels[slot],
		icon: SLOT_ICONS[slot],
		region,
		group,
		padded: true,
		scroll: !UNSCROLLED_SLOTS.includes(slot),
		minimumHeight: region === 'center' ? 120 : 80,
	}
}

const reviewPanels = computed<WorkbenchPanel[]>(() => [
	...props.sidePanels.map((panel) =>
		panelFor(panel, SIDE_PANEL_PLACEMENT[panel].region, SIDE_PANEL_PLACEMENT[panel].group),
	),
	...props.tabs.map((tab) => panelFor(tab, 'center')),
])

const filesSettings = useFilesWorkspaceSettings('project-review', { browsing: 'sidebar' })

/** Folders opened in tabs of their own trees leave no need for the files sidebar. */
const treeTabs = computed(() => filesSettings.store.get(filesSettings.browsing) === 'tree')

const archivePanels = useArchivePanels({
	include: () => ARCHIVE_PANELS.filter((panel) => panel !== 'tree' || !treeTabs.value),
	regions: { tree: 'left', details: 'left', logs: 'bottom' },
	closedByDefault: ['logs'],
})

/** The archive's panels whose content is laid out here, none while it's in a single panel. */
const archivePanelSlots = props.nestedArchive ? [] : ARCHIVE_PANELS

/** The panel `L` brings forward: the archive's files, or the archive's own panel. */
const filesPanel = props.nestedArchive ? ARCHIVE_PANEL : 'tree'

const fileBrowser = injectFileBrowserUI(null)

const panels = computed<WorkbenchPanel[]>(() => [
	...reviewPanels.value,
	...(props.nestedArchive
		? [
				{
					id: ARCHIVE_PANEL,
					title: 'Archive',
					icon: FolderArchiveIcon,
					region: 'center' as const,
					scroll: false,
					minimumHeight: 120,
				},
			]
		: archivePanels.value.map((panel) => ({
				...panel,
				group: panel.region === 'left' ? 'archive' : undefined,
			}))),
])

const workbench = createWorkbench({
	panels,
	storageKey: props.nestedArchive
		? 'moderation.project-review.single-archive'
		: 'moderation.project-review',
	documents: props.nestedArchive ? undefined : useFileDocuments(),
	mode: {
		...WORKBENCH_MODES.review,
		regions: ['left', 'right', 'bottom'],
		toolbar: true,
		movePanels: true,
		closePanels: true,
		headerless: [],
		popouts: {
			url: POPOUT_URL,
			docks: [...WORKBENCH_HOSTS],
			onWindow: (child) => {
				review.close()
				const unregister = shortcuts.registerWindow(child)
				return () => {
					review.close()
					unregister()
				}
			},
		},
	},
	variant: 'rounded',
	sizes: {
		left: { default: 400, minimum: 200 },
		right: { default: 400, minimum: 200 },
		bottom: { default: 220, minimum: 100 },
	},
	centerMinimum: 20,
	ready: () => props.ready,
})

/** Provided here as well as within the layout, for the settings while they float outside it. */
provideWorkbench(workbench)

const leftVisible = workbench.visible.left
const rightVisible = workbench.visible.right

function openTab(tab: ProjectReviewTab) {
	if (props.tabs.includes(tab)) workbench.revealPanel(tab)
}

/**
 * Brings a panel to the front of its group, showing its region (or its window, if popped out).
 * The archive's files, while they're browsed in tabs of trees rather than the sidebar, bring the
 * archive's active tab forward instead.
 */
function reveal(slot: ProjectReviewSlot | ArchivePanelId | typeof ARCHIVE_PANEL) {
	if (panels.value.some((panel) => panel.id === slot)) workbench.revealPanel(slot)
	else if (slot === 'tree' && fileBrowser) {
		fileBrowser.fileTabs.activateTab(fileBrowser.fileTabs.activeTabId.value)
	}
}

/**
 * Whether the project's own panel is shown, in which case the archive's details leave out the
 * project they already show.
 */
const projectShown = computed(() => {
	const project = panels.value.find((panel) => panel.id === 'project')
	if (!project || !workbench.isOpen('project')) return false
	const host = workbench.regionOf(project)
	return host === 'center' || workbench.visible[host].value
})

/** Whether the settings are shown in their floating window, rather than their panel or not at all. */
const settingsWindow = ref(false)

function toggleSlot(slot: ProjectReviewSlot) {
	if (workbench.isShown(slot)) workbench.closePanel(slot)
	else reveal(slot)
}

function toggleSettings() {
	if (workbench.isOpen('settings')) toggleSlot('settings')
	else settingsWindow.value = !settingsWindow.value
}

function settingsAsPanel() {
	settingsWindow.value = false
	reveal('settings')
}

function settingsAsWindow() {
	workbench.closePanel('settings')
	settingsWindow.value = true
}

watch(
	() => workbench.isOpen('settings'),
	(open) => {
		if (open) settingsWindow.value = false
	},
)

provideProjectReviewContext({
	slots,
	leftVisible,
	rightVisible,
	visibleRightWidth: workbench.regionExtents.right,
	toggleSidebar: (side) => workbench.toggleRegion(side),
	counts: computed(() => props.counts ?? {}),
	revealSlot: reveal,
	toggleSlot,
	toggleSettings,
	settingsAsPanel,
	settingsAsWindow,
	openTab,
	resetActiveTabs: workbench.resetActiveTabs,
	selectedTabs: workbench.selectedPanels,
	detachedTabs: workbench.detachedPanels,
	popoutTab: (tab) => void workbench.popoutPanel(tab),
	dockTab: workbench.dockPanel,
})

watchEffect(() => {
	document.body.style.setProperty(
		'--project-review-right-panel-width',
		`${workbench.regionExtents.right.value}px`,
	)
})
onScopeDispose(() => {
	document.body.style.removeProperty('--project-review-right-panel-width')
})

function releaseHeldTab() {
	heldTab.value = undefined
	heldTabKey = undefined
}

function clearShortcutTab() {
	releaseHeldTab()
	shortcutTab.value = undefined
}

watch(
	() => props.resetKey,
	(resetKey, previousResetKey) => {
		clearShortcutTab()
		if (resetKey && previousResetKey) workbench.resetActiveTabs()
	},
)

useReviewShortcut('toggle-left', () => workbench.toggleRegion('left'))
useReviewShortcut('toggle-right', () => workbench.toggleRegion('right'))
useReviewShortcut('reveal-right', () => workbench.setRegionVisible('right', true))
useReviewShortcut('reveal-rulings', () => reveal('rulings'))
useReviewShortcut('reveal-files', () => reveal(filesPanel))

const keybinds = useModerationKeybinds()
useReviewEventListener('keydown', (event) => {
	if (reviewShortcutBlocked(event)) return
	keybinds.value.handle(event, {
		scope: 'project-review',
		openTab: (tab) => {
			if (!props.tabs.includes(tab)) return
			openTab(tab)
			heldTab.value = tab
			shortcutTab.value = tab
			heldTabKey = event.code || event.key.toLowerCase()
		},
	})
})
useReviewEventListener('keyup', (event) => {
	if ((event.code || event.key.toLowerCase()) === heldTabKey) releaseHeldTab()
})
useReviewEventListener('blur', clearShortcutTab)
onScopeDispose(clearShortcutTab)
defineExpose({ reveal, workbench, filesPanel })
</script>

<style scoped>
.project-review {
	height: 100dvh;
	width: 100%;
}
</style>
