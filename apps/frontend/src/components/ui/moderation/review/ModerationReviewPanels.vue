<template>
	<div ref="rootEl" class="flex min-h-0 min-w-0 flex-1 flex-col bg-bg">
		<!-- Tab strip -->
		<div
			class="relative z-10 flex shrink-0 border-0 border-b border-solid border-divider bg-surface-2"
		>
			<div class="flex min-w-0 flex-1 items-stretch gap-px overflow-x-auto">
				<div
					v-for="id in dockState.tabs"
					:key="id"
					class="group relative flex shrink-0 cursor-pointer select-none items-center gap-1.5 border-0 border-r border-solid border-divider px-3 py-2 text-sm"
					:class="[
						dockState.active.includes(id)
							? 'bg-bg font-semibold text-contrast'
							: 'bg-surface-2 text-secondary hover:bg-surface-1',
						{ 'opacity-40': draggingId === id },
					]"
					:title="isPip ? tabDef(id).label : undefined"
					draggable="true"
					@click="onTabClick(id, $event)"
					@contextmenu.prevent="openContextMenu(id, $event)"
					@dragstart="onTabDragStart(id, $event)"
					@dragenter.prevent
					@dragover.prevent="onTabDragOver(id, $event)"
					@drop.prevent="onTabDrop(id)"
					@dragend="clearDrag"
				>
					<div
						v-if="dragOverId === id"
						class="pointer-events-none absolute inset-y-0 z-10 w-0.5 bg-brand"
						:class="dropBefore ? 'left-0' : 'right-0'"
					/>
					<component :is="tabDef(id).icon" class="size-4 shrink-0" />
					<span>{{ tabDef(id).label }}</span>
					<button
						title="Close tab"
						class="rounded p-0.5 text-secondary hover:bg-button-bg hover:text-contrast"
						aria-label="Close tab"
						@click.stop="layout.closeTab(id)"
					>
						<XIcon class="size-3.5" />
					</button>
				</div>
			</div>

			<div class="relative flex shrink-0 items-center gap-0.5 px-1">
				<button
					v-if="activeOrdered.length > 1"
					:title="splitDir === 'column' ? 'Arrange side by side' : 'Arrange stacked'"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Flip split direction"
					@click="layout.toggleSplitDirection(dock)"
				>
					<SplitIcon class="size-4" :class="{ 'rotate-90': splitDir === 'column' }" />
				</button>
				<button
					v-if="activeOrdered.length > 1"
					v-tooltip="'Save this split as a tab group'"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Save as tab group"
					@click="openSaveGroupPrompt"
				>
					<SaveIcon class="size-4" />
				</button>
				<div v-if="saveGroupPrompt" class="fixed inset-0 z-40" @click="saveGroupPrompt = null" />
				<div
					v-if="saveGroupPrompt"
					class="absolute right-1 top-full z-50 mt-1 flex w-56 flex-col gap-2 rounded-lg border border-solid border-divider bg-bg-raised p-2 shadow-lg"
				>
					<input
						ref="groupNameInput"
						v-model="saveGroupPrompt.label"
						type="text"
						placeholder="Group name"
						class="w-full rounded border border-solid border-divider bg-bg px-2 py-1 text-sm text-primary"
						@keydown.enter="confirmSaveGroup"
						@keydown.escape="saveGroupPrompt = null"
					/>
					<Button size="xs" @click="confirmSaveGroup">Save group</Button>
				</div>
				<button
					v-tooltip="'Tab groups'"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast"
					:class="{ 'text-contrast': groupsMenuOpen }"
					aria-label="Tab groups"
					@click="toggleGroupsMenu"
				>
					<LayersIcon class="size-4" />
				</button>
				<div v-if="groupsMenuOpen" class="fixed inset-0 z-40" @click="groupsMenuOpen = false" />
				<div
					v-if="groupsMenuOpen"
					class="absolute right-1 top-full z-50 mt-1 flex min-w-48 flex-col rounded-lg border border-solid border-divider bg-bg-raised p-1 shadow-lg"
				>
					<div v-for="group in layout.tabGroups.value" :key="group.id" class="flex">
						<button
							class="group/item flex items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-primary hover:bg-button-bg"
							@click="activateGroup(group.id)"
						>
							<LayersIcon class="size-4 shrink-0 text-secondary" />
							<span class="min-w-0 flex-1 truncate">{{ group.label }}</span>
							<span class="shrink-0 text-xs text-secondary">{{ group.tabs.length }}</span>
						</button>
						<button
							v-tooltip="'Delete group'"
							class="shrink-0 rounded p-0.5 text-secondary hover:bg-button-bg hover:text-red group-hover/item:opacity-100"
							aria-label="Delete group"
							@click.stop="layout.deleteTabGroup(group.id)"
						>
							<XIcon class="size-4" />
						</button>
					</div>

					<p
						v-if="layout.tabGroups.value.length === 0"
						class="m-0 px-2 py-1.5 text-sm text-secondary"
					>
						No saved groups yet — split two or more tabs, then save them as a group.
					</p>
				</div>
				<button
					v-if="dock === 'main' && (shiftHeld || layout.pipOpen.value)"
					:title="
						!pipSupported
							? 'PiP requires a Chromium-based browser'
							: layout.pipOpen.value
								? 'Close PiP window'
								: 'Open PiP window'
					"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast disabled:opacity-40"
					:class="layout.pipOpen.value ? 'text-contrast' : ''"
					:disabled="!pipSupported"
					aria-label="Toggle PiP window"
					@click="togglePip"
				>
					<WindowIcon class="size-4" />
				</button>
				<button
					title="Open section"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast"
					aria-label="Open section"
					@click="toggleAddMenu"
				>
					<PlusIcon class="size-4" />
				</button>
				<button
					v-tooltip="topBarCollapsed ? 'Expand Topbar' : 'Collapse Topbar'"
					class="rounded p-1.5 text-secondary hover:bg-button-bg hover:text-contrast"
					:aria-label="topBarCollapsed ? 'Expand Topbar' : 'Collapse Topbar'"
					@click="topBarCollapsed = !topBarCollapsed"
				>
					<ChevronDownIcon v-if="topBarCollapsed" class="size-4" />
					<ChevronUpIcon v-else class="size-4" />
				</button>
				<div v-if="addMenuOpen" class="fixed inset-0 z-40" @click="addMenuOpen = false" />
				<div
					v-if="addMenuOpen"
					class="absolute right-1 top-full z-50 mt-1 flex min-w-40 flex-col rounded-lg border border-solid border-divider bg-bg-raised p-1 shadow-lg"
				>
					<button
						v-for="id in closedTabs"
						:key="id"
						class="flex items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-primary hover:bg-button-bg"
						@click="openFromMenu(id)"
					>
						<component :is="tabDef(id).icon" class="size-4" />
						{{ tabDef(id).label }}
					</button>
					<p v-if="closedTabs.length === 0" class="m-0 px-2 py-1.5 text-sm text-secondary">
						All sections open
					</p>
				</div>
			</div>
		</div>

		<!-- Panel bodies -->
		<div
			v-if="dockState.active.length > 0"
			ref="panelsContainer"
			class="flex min-h-0 min-w-0 flex-1"
			:class="[splitDir === 'column' ? 'flex-col' : 'flex-row', { 'select-none': resizing }]"
		>
			<template v-for="(id, index) in activeOrdered" :key="id">
				<div
					v-if="index > 0"
					class="group flex shrink-0 items-center justify-center"
					:class="splitDir === 'column' ? 'h-2 w-full cursor-row-resize' : 'w-2 cursor-col-resize'"
					role="separator"
					:aria-orientation="splitDir === 'column' ? 'horizontal' : 'vertical'"
					@pointerdown="startResize(index, $event)"
				>
					<div
						class="transition-colors"
						:class="[
							splitDir === 'column'
								? 'h-px w-full group-hover:h-0.5'
								: 'h-full w-px group-hover:w-0.5',
							resizing === index
								? `bg-brand ${splitDir === 'column' ? 'h-0.5' : 'w-0.5'}`
								: 'bg-divider group-hover:bg-brand',
						]"
					/>
				</div>
				<ModerationElementFrame
					:element-key="elementKeyForTab(id)"
					:auto-bars="id !== 'description'"
					class="min-h-0 min-w-0"
					:style="{ flexGrow: weightOf(id), flexShrink: 1, flexBasis: '0' }"
				>
					<div class="p-4">
						<Suspense>
							<component :is="tabDef(id).component" />
							<template #fallback>
								<div class="flex items-center justify-center gap-2 py-12 text-secondary">
									<SpinnerIcon class="size-5 animate-spin" /> Loading {{ tabDef(id).label }}…
								</div>
							</template>
						</Suspense>
					</div>
				</ModerationElementFrame>
			</template>
		</div>
		<div v-else class="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center">
			<p class="m-0 text-secondary">No sections open in this panel.</p>
			<div class="flex flex-wrap justify-center gap-2">
				<Button v-for="id in selectableTabs" :key="id" @click="layout.openTab(id, { dock })">
					<component :is="tabDef(id).icon" />
					{{ tabDef(id).label }}
				</Button>
			</div>
		</div>

		<!-- Tab context menu -->
		<template v-if="contextMenu">
			<div
				class="fixed inset-0 z-[60]"
				@click="contextMenu = null"
				@contextmenu.prevent="contextMenu = null"
			/>
			<div
				class="fixed z-[61] flex min-w-[12rem] flex-col rounded-lg border border-solid border-divider bg-bg-raised p-1 text-sm shadow-lg"
				:style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
			>
				<button
					v-if="dock === 'pip'"
					class="ctx-item"
					@click="runContext(() => layout.moveTab(menuTargetId, 'main'))"
				>
					<ExternalIcon class="size-4" /> Move to main window
				</button>
				<button
					v-else-if="pipSupported"
					class="ctx-item"
					@click="runContext(() => layout.moveTab(menuTargetId, 'pip'))"
				>
					<WindowIcon class="size-4" /> Open in PiP window
				</button>

				<button class="ctx-item" @click="runContext(() => layout.toggleActive(menuTargetId, dock))">
					<SplitIcon class="size-4" />
					{{ isInSplit(menuTargetId) ? 'Remove from split view' : 'Add to split view' }}
				</button>

				<!-- Future: "Open right" / "Open down" for multi-tab section layouts -->

				<div class="my-1 h-px bg-divider" />
				<button
					class="ctx-item ctx-item--danger"
					@click="runContext(() => layout.closeTab(menuTargetId))"
				>
					<XIcon class="size-4" /> Close tab
				</button>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	ChevronDownIcon,
	ChevronUpIcon,
	ExternalIcon,
	LayersIcon,
	PlusIcon,
	SaveIcon,
	SpinnerIcon,
	SplitIcon,
	WindowIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, injectPageContext, injectProjectPageContext } from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import type { ChecklistElementKey } from '~/components/ui/moderation/checklist/checklist-context'
import {
	REVIEW_TAB_ORDER,
	type ReviewDockId,
	type ReviewTabId,
	useModerationReviewLayout,
} from '~/services/moderation/review-layout'

import ModerationElementFrame from './ModerationElementFrame.vue'
import { pipSupported } from './pip-window'
import { reviewTab, selectableReviewTabs } from './review-tabs'

const pageContext = injectPageContext()
const topBarCollapsed = pageContext.topBarCollapsed

const props = defineProps<{ dock: ReviewDockId }>()

const ELEMENT_TABS = new Set<ReviewTabId>([
	'description',
	'gallery',
	'versions',
	'tags',
	'license',
	'links',
	'disclosures',
	'permissions',
])

function elementKeyForTab(id: ReviewTabId): ChecklistElementKey | null {
	return ELEMENT_TABS.has(id) ? (id as ChecklistElementKey) : null
}

const layout = useModerationReviewLayout()
const { loadVersions, projectV3 } = injectProjectPageContext()

const isModpack = computed(() => (projectV3.value?.project_types ?? []).includes('modpack'))
const selectableTabs = computed(() => selectableReviewTabs(REVIEW_TAB_ORDER, isModpack.value))

const isPip = computed(() => props.dock === 'pip')

// This instance renders inside the review's Picture-in-Picture window when `dock === 'pip'` —
// that window shares the main window's JS realm, so the bare `window` global here always means
// the *main* window. A dock's own keyboard/blur handling (closing its own menus on Escape, in
// particular) has to listen on whichever window this instance actually lives in.
const rootEl = ref<HTMLElement | null>(null)
const ownerWindow = computed<Window>(() => rootEl.value?.ownerDocument?.defaultView ?? window)

// The PiP toggle only appears while Shift is held (or while a PiP window is already open).
const shiftHeld = ref(false)
function onKeyDown(e: KeyboardEvent) {
	if (e.key === 'Shift') shiftHeld.value = true
	if (e.key === 'Escape') {
		contextMenu.value = null
		closeMenus()
	}
}
function onKeyUp(e: KeyboardEvent) {
	if (e.key === 'Shift') shiftHeld.value = false
}
function onWindowBlur() {
	shiftHeld.value = false
}
/** The window `onKeyDown`/`onKeyUp`/`onWindowBlur` were registered against — matched on cleanup. */
let listenerWindow: Window = window
onMounted(() => {
	listenerWindow = ownerWindow.value
	listenerWindow.addEventListener('keydown', onKeyDown)
	listenerWindow.addEventListener('keyup', onKeyUp)
	listenerWindow.addEventListener('blur', onWindowBlur)
})
onBeforeUnmount(() => {
	listenerWindow.removeEventListener('keydown', onKeyDown)
	listenerWindow.removeEventListener('keyup', onKeyUp)
	listenerWindow.removeEventListener('blur', onWindowBlur)
})

function togglePip() {
	if (layout.pipOpen.value) layout.reclaimPipTabs()
	else layout.openPip()
}

const dockState = computed(() =>
	props.dock === 'main' ? layout.mainDock.value : layout.pipDock.value,
)
const closedTabs = computed(() =>
	selectableTabs.value.filter((id) => !dockState.value.tabs.includes(id)),
)
/** Active tabs in tab-strip order (the split panels follow the strip, not REVIEW_TAB_ORDER). */
const activeOrdered = computed(() =>
	dockState.value.tabs.filter((id) => dockState.value.active.includes(id)),
)
const splitDir = computed(() => dockState.value.splitDirection ?? 'row')

const addMenuOpen = ref(false)
const groupsMenuOpen = ref(false)
const saveGroupPrompt = ref<{ label: string } | null>(null)
const groupNameInput = ref<HTMLInputElement | null>(null)

function tabDef(id: ReviewTabId) {
	return reviewTab(id)
}

function closeMenus() {
	addMenuOpen.value = false
	groupsMenuOpen.value = false
	saveGroupPrompt.value = null
}

function toggleAddMenu() {
	const next = !addMenuOpen.value
	closeMenus()
	addMenuOpen.value = next
}

function toggleGroupsMenu() {
	const next = !groupsMenuOpen.value
	closeMenus()
	groupsMenuOpen.value = next
}

function openSaveGroupPrompt() {
	const label = activeOrdered.value.map((id) => tabDef(id).label).join(' + ')
	closeMenus()
	saveGroupPrompt.value = { label }
	nextTick(() => groupNameInput.value?.focus())
}

function confirmSaveGroup() {
	if (!saveGroupPrompt.value) return
	layout.saveTabGroup(props.dock, saveGroupPrompt.value.label)
	saveGroupPrompt.value = null
}

function activateGroup(groupId: string) {
	layout.activateTabGroup(props.dock, groupId)
	groupsMenuOpen.value = false
}

function isInSplit(id: ReviewTabId) {
	return dockState.value.active.includes(id) && dockState.value.active.length > 1
}

function onTabClick(id: ReviewTabId, event: MouseEvent) {
	if (event.ctrlKey || event.metaKey) {
		layout.toggleActive(id, props.dock)
	} else {
		layout.focusTab(id, props.dock)
	}
}

const contextMenu = ref<{ id: ReviewTabId; x: number; y: number } | null>(null)
const menuTargetId = computed<ReviewTabId>(() => contextMenu.value?.id ?? REVIEW_TAB_ORDER[0])

function openContextMenu(id: ReviewTabId, event: MouseEvent) {
	closeMenus()
	const view = (event.view as Window | null) ?? window
	const x = Math.min(event.clientX, view.innerWidth - 208)
	const y = Math.min(event.clientY, view.innerHeight - 176)
	contextMenu.value = { id, x: Math.max(4, x), y: Math.max(4, y) }
}

function runContext(action: () => void) {
	action()
	contextMenu.value = null
}

function openFromMenu(id: ReviewTabId) {
	layout.openTab(id, { dock: props.dock })
	addMenuOpen.value = false
}

// --- Tab reordering (drag and drop) -------------------------------------------------

const draggingId = ref<ReviewTabId | null>(null)
const dragOverId = ref<ReviewTabId | null>(null)
const dropBefore = ref(true)

function onTabDragStart(id: ReviewTabId, event: DragEvent) {
	draggingId.value = id
	if (event.dataTransfer) {
		event.dataTransfer.effectAllowed = 'move'
		event.dataTransfer.setData('text/plain', id)
	}
}

function onTabDragOver(id: ReviewTabId, event: DragEvent) {
	if (!draggingId.value) return
	if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
	const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
	dropBefore.value = event.clientX - rect.left < rect.width / 2
	dragOverId.value = id
}

function onTabDrop(targetId: ReviewTabId) {
	const dragged = draggingId.value
	if (dragged && dragged !== targetId) {
		const order = dockState.value.tabs.filter((t) => t !== dragged)
		const targetIndex = order.indexOf(targetId)
		order.splice(dropBefore.value ? targetIndex : targetIndex + 1, 0, dragged)
		layout.reorderTabs(props.dock, order)
	}
	clearDrag()
}

function clearDrag() {
	draggingId.value = null
	dragOverId.value = null
}

// --- Split view resizing -------------------------------------------------------------

const panelsContainer = ref<HTMLElement | null>(null)
/** Divider index (1-based, matching the panel it precedes) currently being dragged. */
const resizing = ref<number | null>(null)
/** Live weights during a drag; committed to the store on pointer up. */
const dragWeights = ref<Partial<Record<ReviewTabId, number>> | null>(null)

const MIN_PANEL_FRACTION = 0.08

function weightOf(id: ReviewTabId): number {
	return dragWeights.value?.[id] ?? layout.panelWeight(props.dock, id)
}

function startResize(dividerIndex: number, event: PointerEvent) {
	const container = panelsContainer.value
	if (!container || dividerIndex < 1) return
	event.preventDefault()

	const active = activeOrdered.value
	const rect = container.getBoundingClientRect()
	const win = (event.view as Window | null) ?? window
	const isColumn = splitDir.value === 'column'
	const axisStart = isColumn ? rect.top : rect.left
	const axisSize = isColumn ? rect.height : rect.width

	const weights = Object.fromEntries(
		active.map((id) => [id, layout.panelWeight(props.dock, id)]),
	) as Record<ReviewTabId, number>
	const total = active.reduce((sum, id) => sum + weights[id], 0)

	const leftId = active[dividerIndex - 1]
	const rightId = active[dividerIndex]
	const before = active.slice(0, dividerIndex - 1).reduce((s, id) => s + weights[id], 0)
	const pairWeight = weights[leftId] + weights[rightId]

	resizing.value = dividerIndex
	dragWeights.value = { ...weights }

	function onMove(e: PointerEvent) {
		const fraction = ((isColumn ? e.clientY : e.clientX) - axisStart) / axisSize
		const min = before / total + MIN_PANEL_FRACTION
		const max = (before + pairWeight) / total - MIN_PANEL_FRACTION
		if (max <= min) return
		const clamped = Math.min(Math.max(fraction, min), max)
		const leftWeight = (clamped - before / total) * total
		dragWeights.value = {
			...weights,
			[leftId]: leftWeight,
			[rightId]: pairWeight - leftWeight,
		}
	}

	function onUp() {
		win.removeEventListener('pointermove', onMove)
		win.removeEventListener('pointerup', onUp)
		if (dragWeights.value) layout.setPanelWeights(props.dock, dragWeights.value)
		dragWeights.value = null
		resizing.value = null
	}

	win.addEventListener('pointermove', onMove)
	win.addEventListener('pointerup', onUp)
}

// Lazily load version data whenever a version-backed tab is visible in this dock.
watch(
	() => dockState.value.tabs.map((id) => reviewTab(id).requiresVersions).some(Boolean),
	(needsVersions) => {
		if (needsVersions) loadVersions()
	},
	{ immediate: true },
)
</script>

<style scoped>
.ctx-item {
	display: flex;
	width: 100%;
	align-items: center;
	gap: 0.5rem;
	border-radius: 0.25rem;
	padding: 0.375rem 0.5rem;
	text-align: left;
	color: var(--color-base);
}

.ctx-item:hover {
	background-color: var(--color-button-bg);
}

.ctx-item--danger {
	color: var(--color-red);
}
</style>
