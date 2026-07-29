<template>
	<ConfirmModal
		ref="deleteModal"
		:markdown="false"
		:title="deleteTitle"
		:description="formatMessage(analyticsMessages.breakdownGroupsDeleteDescription)"
		:proceed-label="formatMessage(analyticsMessages.breakdownGroupsDelete)"
		@proceed="confirmDelete"
	/>
	<div ref="optionContent" class="group flex min-w-0 flex-1 items-center gap-2">
		<slot></slot>
		<ButtonStyled type="outlined" size="small">
			<button
				class="!shadow-none transition-opacity"
				:class="
					activeGroup
						? undefined
						: 'md:opacity-0 md:group-focus-within:opacity-100 md:group-hover:opacity-100'
				"
				type="button"
				:aria-label="formatMessage(analyticsMessages.breakdownGroupsButton)"
				:aria-expanded="isMenuOpen"
				aria-haspopup="menu"
				@click.stop="toggleMenu"
				@keydown.enter.stop
				@keydown.space.stop
			>
				<span class="max-w-32 truncate">
					{{ activeGroup?.name ?? formatMessage(analyticsMessages.breakdownGroupsButton) }}
				</span>
				<ChevronRightIcon />
			</button>
		</ButtonStyled>
	</div>

	<Teleport to="#teleports">
		<Transition
			enter-active-class="transition-opacity duration-150"
			leave-active-class="transition-opacity duration-150"
			enter-from-class="opacity-0"
			leave-to-class="opacity-0"
		>
			<div
				v-if="isMenuOpen"
				ref="submenu"
				data-multi-select-submenu
				class="fixed z-[10000] w-64 overflow-hidden rounded-xl border border-solid border-surface-5 bg-surface-4 text-primary shadow-2xl"
				:style="submenuStyle"
				role="menu"
				@mousedown.stop
				@click.stop
				@keydown.esc.stop="closeMenu"
			>
				<button
					type="button"
					class="menu-row"
					:class="{ 'text-brand': isNoGroupingActive }"
					@click="selectGroup(null)"
				>
					<span class="min-w-0 flex-1 text-left">
						{{ formatMessage(analyticsMessages.breakdownGroupsNoGrouping) }}
					</span>
					<CheckIcon v-if="isNoGroupingActive" />
				</button>
				<div
					v-for="group in groups"
					:key="group.id"
					class="group/menu-item flex items-center hover:bg-surface-5"
				>
					<button
						type="button"
						class="menu-row min-w-0 flex-1"
						:class="{ 'text-brand': isGroupActive(group) }"
						@click="selectGroup(group)"
					>
						<span class="min-w-0 flex-1 truncate text-left">{{ group.name }}</span>
						<CheckIcon v-if="isGroupActive(group)" class="shrink-0" />
					</button>
					<button
						type="button"
						class="icon-action"
						:aria-label="formatMessage(analyticsMessages.breakdownGroupsEdit)"
						@click.stop="editGroup(group, $event)"
					>
						<PencilIcon />
					</button>
					<button
						type="button"
						class="icon-action hover:!text-red"
						:aria-label="formatMessage(analyticsMessages.breakdownGroupsDelete)"
						@click.stop="requestDelete(group, $event)"
					>
						<TrashIcon />
					</button>
				</div>
				<button
					type="button"
					class="menu-row border-0 border-t border-solid border-surface-5 text-brand"
					@click="createGroup"
				>
					<PlusIcon />
					<span>{{ formatMessage(analyticsMessages.breakdownGroupsCreate) }}</span>
				</button>
			</div>
		</Transition>
	</Teleport>
</template>

<script setup lang="ts">
import { CheckIcon, ChevronRightIcon, PencilIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import { ButtonStyled, ConfirmModal, useVIntl } from '@modrinth/ui'

import type {
	AnalyticsActiveBreakdownGroup,
	AnalyticsBreakdownGroup,
	AnalyticsBreakdownPreset,
} from '~/providers/analytics/analytics'
import { injectAnalyticsDashboardContext } from '~/providers/analytics/analytics'

import { analyticsMessages } from '../analytics-messages'

type Breakdown = Exclude<AnalyticsBreakdownPreset, 'none'>

const SUBMENU_GAP = 8
const VIEWPORT_MARGIN = 8
const SUBMENU_WIDTH = 256
const OPEN_SUBMENU_EVENT = 'analytics-breakdown-group-submenu-open'

const props = defineProps<{
	breakdown: Breakdown
	selected: boolean
	activeGroup: AnalyticsActiveBreakdownGroup | null
}>()
const emit = defineEmits<{
	activate: [breakdown: Breakdown]
	selectGroup: [group: AnalyticsBreakdownGroup | null]
	create: [breakdown: Breakdown, event?: MouseEvent]
	edit: [group: AnalyticsBreakdownGroup, event?: MouseEvent]
}>()

const { breakdownGroups, deleteBreakdownGroup } = injectAnalyticsDashboardContext()
const { formatMessage } = useVIntl()
const optionContent = ref<HTMLElement | null>(null)
const submenu = ref<HTMLElement | null>(null)
const isMenuOpen = ref(false)
const submenuStyle = ref({ left: '0px', top: '0px' })
const deleteModal = ref<InstanceType<typeof ConfirmModal> | null>(null)
const pendingDeleteGroup = ref<AnalyticsBreakdownGroup | null>(null)
const menuId = Symbol('breakdown-group-submenu')

const groups = computed(() =>
	breakdownGroups.value.filter((group) => group.breakdown === props.breakdown),
)
const activeGroup = computed(() => {
	if (!props.selected || props.activeGroup?.breakdown !== props.breakdown) return null
	return groups.value.find((group) => group.id === props.activeGroup?.groupId) ?? null
})
const isNoGroupingActive = computed(() => props.selected && activeGroup.value === null)
const deleteTitle = computed(() =>
	formatMessage(analyticsMessages.breakdownGroupsDeleteTitle, {
		name: pendingDeleteGroup.value?.name ?? '',
	}),
)

function isGroupActive(group: AnalyticsBreakdownGroup) {
	return activeGroup.value?.id === group.id
}

function selectGroup(group: AnalyticsBreakdownGroup | null) {
	emit('selectGroup', group)
	closeMenu()
}

function openMenu() {
	if (!isMenuOpen.value) {
		window.dispatchEvent(new CustomEvent(OPEN_SUBMENU_EVENT, { detail: menuId }))
		isMenuOpen.value = true
	}
	updateMenuPosition()
	nextTick(updateMenuPosition)
}

function closeMenu() {
	isMenuOpen.value = false
}

function toggleMenu() {
	if (isMenuOpen.value) {
		closeMenu()
	} else {
		openMenu()
	}
}

function updateMenuPosition() {
	if (!isMenuOpen.value || typeof window === 'undefined') return

	const anchor = optionContent.value?.closest<HTMLElement>('[role="option"]') ?? optionContent.value
	if (!anchor) return

	const anchorRect = anchor.getBoundingClientRect()
	const submenuRect = submenu.value?.getBoundingClientRect()
	const submenuWidth = submenuRect?.width ?? SUBMENU_WIDTH
	const submenuHeight = submenuRect?.height ?? 320
	const viewport = window.visualViewport
	const viewportLeft = viewport?.offsetLeft ?? 0
	const viewportTop = viewport?.offsetTop ?? 0
	const viewportRight = viewportLeft + (viewport?.width ?? window.innerWidth)
	const viewportBottom = viewportTop + (viewport?.height ?? window.innerHeight)
	const anchorLeft = anchorRect.left + viewportLeft
	const anchorRight = anchorRect.right + viewportLeft
	const anchorTop = anchorRect.top + viewportTop
	const opensRight =
		anchorRight + SUBMENU_GAP + submenuWidth + VIEWPORT_MARGIN <= viewportRight ||
		anchorLeft - submenuWidth - SUBMENU_GAP < viewportLeft + VIEWPORT_MARGIN
	const preferredLeft = opensRight
		? anchorRight + SUBMENU_GAP
		: anchorLeft - submenuWidth - SUBMENU_GAP
	const maxLeft = Math.max(
		viewportLeft + VIEWPORT_MARGIN,
		viewportRight - submenuWidth - VIEWPORT_MARGIN,
	)
	const maxTop = Math.max(
		viewportTop + VIEWPORT_MARGIN,
		viewportBottom - submenuHeight - VIEWPORT_MARGIN,
	)

	submenuStyle.value = {
		left: `${Math.min(Math.max(viewportLeft + VIEWPORT_MARGIN, preferredLeft), maxLeft)}px`,
		top: `${Math.min(Math.max(viewportTop + VIEWPORT_MARGIN, anchorTop), maxTop)}px`,
	}
}

function handleOtherSubmenuOpen(event: Event) {
	if ((event as CustomEvent<symbol>).detail !== menuId) closeMenu()
}

function handleDocumentMouseDown(event: MouseEvent) {
	const target = event.target
	if (
		isMenuOpen.value &&
		target instanceof Node &&
		!optionContent.value?.contains(target) &&
		!submenu.value?.contains(target)
	) {
		closeMenu()
	}
}

function createGroup(event: MouseEvent) {
	closeMenu()
	emit('activate', props.breakdown)
	emit('create', props.breakdown, event)
}

function editGroup(group: AnalyticsBreakdownGroup, event: MouseEvent) {
	closeMenu()
	emit('edit', group, event)
}

function requestDelete(group: AnalyticsBreakdownGroup, event: MouseEvent) {
	pendingDeleteGroup.value = group
	closeMenu()
	deleteModal.value?.show(event)
}

function confirmDelete() {
	if (pendingDeleteGroup.value) {
		if (isGroupActive(pendingDeleteGroup.value)) emit('selectGroup', null)
		deleteBreakdownGroup(pendingDeleteGroup.value.id)
	}
	pendingDeleteGroup.value = null
}

onMounted(() => {
	window.addEventListener(OPEN_SUBMENU_EVENT, handleOtherSubmenuOpen)
	window.addEventListener('resize', updateMenuPosition)
	window.addEventListener('scroll', updateMenuPosition, true)
	document.addEventListener('mousedown', handleDocumentMouseDown)
})

onBeforeUnmount(() => {
	window.removeEventListener(OPEN_SUBMENU_EVENT, handleOtherSubmenuOpen)
	window.removeEventListener('resize', updateMenuPosition)
	window.removeEventListener('scroll', updateMenuPosition, true)
	document.removeEventListener('mousedown', handleDocumentMouseDown)
})
</script>

<style scoped>
.menu-row {
	display: flex;
	width: 100%;
	align-items: center;
	gap: 0.625rem;
	border: 0;
	background: transparent;
	padding: 0.75rem 1rem;
	color: inherit;
	box-shadow: none;
}

.menu-row:hover,
.menu-row:focus-visible {
	background: var(--surface-5);
}

.menu-row svg,
.icon-action svg {
	width: 1.25rem;
	height: 1.25rem;
}

.icon-action {
	display: flex;
	flex-shrink: 0;
	align-items: center;
	justify-content: center;
	border: 0;
	background: transparent;
	padding: 0.5rem;
	color: var(--color-secondary);
	box-shadow: none;
}

.icon-action:hover,
.icon-action:focus-visible {
	color: var(--color-contrast);
}
</style>
