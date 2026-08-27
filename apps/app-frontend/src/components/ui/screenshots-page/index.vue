<script setup lang="ts">
import {
	DragDropProvider,
	type DragEndEvent,
	type DragMoveEvent,
	type DragOverEvent,
	DragOverlay,
	type DragStartEvent,
} from '@dnd-kit/vue'
import {
	CheckIcon,
	ClipboardCopyIcon,
	EditIcon,
	FileArchiveIcon,
	FolderOpenIcon,
	MinusIcon,
	SquarePlusIcon,
	TrashIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	type ButtonMenuOption,
	type ComboboxOption,
	commonMessages,
	ConfirmModal,
	ContextMenu,
	defineMessages,
	EmptyState,
	FloatingActionBar,
	IconButton,
	ImageViewerEditor,
	type ImageViewerEditorSavePayload,
	injectNotificationManager,
	ReadyTransition,
	useFormatDateTime,
	useReadyState,
	useScrollViewport,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { save } from '@tauri-apps/plugin-dialog'
import { readFile } from '@tauri-apps/plugin-fs'
import { useElementSize, useStorage, useWindowSize } from '@vueuse/core'
import dayjs from 'dayjs'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import {
	create_screenshot_group,
	delete_screenshot_group,
	delete_screenshots,
	export_screenshots,
	getInstanceIconUrl,
	import_screenshot_groups,
	type InstanceScreenshot,
	list_all_screenshots,
	move_screenshots,
	open_screenshot,
	rename_screenshot_group,
	save_edited_screenshot,
	type ScreenshotGroup,
	type ScreenshotGroupImport,
	type ScreenshotKey,
	set_screenshot_group_memberships,
} from '@/helpers/instance'
import { MAX_INSTANCE_GROUP_NAME_LENGTH } from '@/helpers/instance-groups'
import {
	instanceListQueryOptions,
	instanceScreenshotsQueryOptions,
	screenshotGroupsQueryOptions,
	screenshotKeys,
	syncedScreenshotsQueryOptions,
} from '@/pages/instance/query-options'

import ScreenshotDragGather from './drag-gather.vue'
import ScreenshotDragPreview from './drag-preview.vue'
import ScreenshotGroupSection from './group.vue'
import ScreenshotToolbar from './toolbar.vue'
import { type ActiveScreenshotDrag, useScreenshotDragGather } from './use-screenshot-drag-gather'

type ScreenshotSort = 'newest' | 'oldest' | 'name'
type ScreenshotGroupBy = 'custom' | 'instance' | 'date' | 'none'

type LegacyCustomScreenshotGrouping = {
	groups: ScreenshotGroup[]
	assignments: Record<string, string>
}

type ScreenshotGroupData = {
	id: string
	title: string
	screenshots: InstanceScreenshot[]
	dropInstanceId?: string
	customGroupId?: string | null
}

type ScreenshotDragData = {
	selectionKey: string
	instanceId: string
}

type ScreenshotDropData = {
	groupId: string
	instanceId?: string
	customGroupId?: string | null
}

type ScreenshotGroupLayout = {
	group: ScreenshotGroupData
	top: number
	height: number
	isOpen: boolean
	gridHeight: number
	gridTop: number
}

type VisibleScreenshotGroupLayout = ScreenshotGroupLayout & {
	renderedScreenshots: InstanceScreenshot[]
	virtualGridTop: number
}

const SCREENSHOT_GRID_GAP = 12
const SCREENSHOT_GROUP_SPACING = 12
const SCREENSHOT_GROUP_HEADER_HEIGHT = 40
const SCREENSHOT_GROUP_CONTENT_SPACING = 10
const SCREENSHOT_GROUP_OVERSCAN = 900
const SCREENSHOT_GRID_MIN_HEIGHT = 45
const FALLBACK_SCREENSHOT_CARD_WIDTH = 320

const props = withDefaults(
	defineProps<{
		instanceId?: string
		showHeading?: boolean
	}>(),
	{
		instanceId: undefined,
		showHeading: false,
	},
)

const isGlobal = computed(() => !props.instanceId)
const storageSuffix = props.instanceId ? 'instance' : 'global'
const search = ref('')
const sort = useStorage<ScreenshotSort>(`screenshots-sort-${storageSuffix}`, 'newest')
const groupBy = useStorage<ScreenshotGroupBy>(`screenshots-group-v2-${storageSuffix}`, 'date')
const sortModel = computed<string>({
	get: () => sort.value,
	set: (value) => {
		sort.value = value as ScreenshotSort
	},
})
const groupByModel = computed<string>({
	get: () => groupBy.value,
	set: (value) => {
		groupBy.value = value as ScreenshotGroupBy
	},
})
const collapsedGroups = useStorage<Record<string, boolean>>(
	`screenshots-collapsed-groups-${storageSuffix}`,
	{},
)
const legacyCustomGrouping = useStorage<LegacyCustomScreenshotGrouping>(
	'screenshots-custom-groups',
	{
		groups: [],
		assignments: {},
	},
)
const selectedKeys = ref(new Set<string>())
const copiedScreenshotIds = ref(new Set<string>())
const copiedResetTimeouts = new Map<string, ReturnType<typeof setTimeout>>()
const screenshotsPage = ref<HTMLElement>()
const regrouping = ref(false)
const screenshotsScrolling = ref(false)
const screenshotToDelete = ref<InstanceScreenshot | null>(null)
const deleteFromPreview = ref(false)
const activeDrag = ref<ActiveScreenshotDrag | null>(null)
const activeDropGroupId = ref<string | null>(null)
const imageViewer = ref<InstanceType<typeof ImageViewerEditor>>()
const screenshotOptionsMenu = ref<InstanceType<typeof ContextMenu>>()
const screenshotOptionsTarget = ref<InstanceScreenshot>()
const deleteModal = ref<InstanceType<typeof ConfirmModal>>()
const bulkDeleteModal = ref<InstanceType<typeof ConfirmModal>>()
const deleteGroupModal = ref<InstanceType<typeof ConfirmModal>>()
const customGroupToDelete = ref<ScreenshotGroup>()
const groupIdPendingNameEdit = ref<string>()
const migratingLegacyGroups = ref(false)
const creatingCustomGroup = ref(false)
const updatingCustomGroupMemberships = ref(false)
const revealedScreenshotId = ref<string>()
const highlightedScreenshotId = ref<string>()
const queryClient = useQueryClient()
const route = useRoute()
const router = useRouter()
const { formatMessage } = useVIntl()
const { addNotification, handleError } = injectNotificationManager()
let screenshotsScrollIdleTimeout: ReturnType<typeof setTimeout> | undefined
const {
	listContainer: screenshotListContainer,
	containerOffset: screenshotListOffset,
	relativeScrollTop: screenshotListScrollTop,
	scrollContainer: screenshotScrollContainer,
	viewportHeight: screenshotViewportHeight,
} = useScrollViewport({
	onScroll: () => {
		screenshotsScrolling.value = true
		if (screenshotsScrollIdleTimeout) clearTimeout(screenshotsScrollIdleTimeout)
		screenshotsScrollIdleTimeout = setTimeout(() => {
			screenshotsScrolling.value = false
			screenshotsScrollIdleTimeout = undefined
		}, 120)
	},
})
const { width: screenshotListWidth } = useElementSize(screenshotListContainer)
const { width: windowWidth } = useWindowSize()
const formatDateTime = useFormatDateTime({ dateStyle: 'long', timeStyle: 'short' })
const formatMonth = useFormatDateTime({ month: 'long', year: 'numeric' })
const messages = defineMessages({
	heading: { id: 'app.screenshots.heading', defaultMessage: 'Screenshots' },
	emptyHeading: { id: 'app.screenshots.empty-heading', defaultMessage: 'No screenshots yet' },
	emptyDescription: {
		id: 'app.screenshots.empty-description',
		defaultMessage: 'Screenshots you take in-game will appear here.',
	},
	noResultsHeading: {
		id: 'app.screenshots.no-results-heading',
		defaultMessage: 'No matching screenshots',
	},
	noResultsDescription: {
		id: 'app.screenshots.no-results-description',
		defaultMessage: 'Try a different search or filter.',
	},
	errorHeading: {
		id: 'app.screenshots.error-heading',
		defaultMessage: 'Failed to load screenshots',
	},
	newest: { id: 'app.screenshots.sort.newest', defaultMessage: 'Newest' },
	oldest: { id: 'app.screenshots.sort.oldest', defaultMessage: 'Oldest' },
	name: { id: 'app.screenshots.sort.name', defaultMessage: 'Name' },
	custom: { id: 'app.screenshots.group.custom', defaultMessage: 'Custom group' },
	instance: { id: 'app.screenshots.group.instance', defaultMessage: 'Instance' },
	date: { id: 'app.screenshots.group.date', defaultMessage: 'Date' },
	none: { id: 'app.screenshots.group.none', defaultMessage: 'No grouping' },
	today: { id: 'app.screenshots.group.today', defaultMessage: 'Today' },
	yesterday: { id: 'app.screenshots.group.yesterday', defaultMessage: 'Yesterday' },
	thisWeek: { id: 'app.screenshots.group.this-week', defaultMessage: 'This week' },
	thisMonth: { id: 'app.screenshots.group.this-month', defaultMessage: 'This month' },
	allScreenshots: {
		id: 'app.screenshots.group.all-screenshots',
		defaultMessage: 'All screenshots',
	},
	ungrouped: { id: 'app.screenshots.group.ungrouped', defaultMessage: 'Ungrouped' },
	editGroup: { id: 'app.screenshots.group.edit', defaultMessage: 'Edit group name' },
	deleteGroup: { id: 'app.screenshots.group.delete', defaultMessage: 'Delete group' },
	deleteGroupDescription: {
		id: 'app.screenshots.group.delete-description',
		defaultMessage: 'Screenshots in this group will become ungrouped.',
	},
	instanceAndDate: {
		id: 'app.screenshots.preview.instance-and-date',
		defaultMessage: '{instance} · {date}',
	},
	copy: { id: 'app.screenshots.copy', defaultMessage: 'Copy image' },
	copied: { id: 'app.screenshots.copied', defaultMessage: 'Copied' },
	edit: { id: 'app.screenshots.edit', defaultMessage: 'Edit screenshot' },
	showInFolder: { id: 'app.screenshots.show-in-folder', defaultMessage: 'Show in folder' },
	goToInstance: { id: 'app.screenshots.go-to-instance', defaultMessage: 'Go to instance' },
	deleteTitle: { id: 'app.screenshots.delete-title', defaultMessage: 'Delete screenshot' },
	deleteDescription: {
		id: 'app.screenshots.delete-description',
		defaultMessage: 'Permanently delete {name}? This action cannot be undone.',
	},
	deleteSuccess: { id: 'app.screenshots.delete-success', defaultMessage: 'Screenshot deleted' },
	selectionAriaLabel: {
		id: 'app.screenshots.selection.aria-label',
		defaultMessage: 'Selected screenshots',
	},
	selectedCount: {
		id: 'app.screenshots.selection.selected-count',
		defaultMessage: '{count} selected',
	},
	exportZip: { id: 'app.screenshots.selection.export-zip', defaultMessage: 'Export ZIP' },
	newGroup: { id: 'app.screenshots.group.new', defaultMessage: 'New group' },
	removeFromGroup: {
		id: 'app.screenshots.selection.remove-from-group',
		defaultMessage: 'Remove from group',
	},
	zipArchive: { id: 'app.screenshots.selection.zip-archive', defaultMessage: 'ZIP archive' },
	globalExportFilename: {
		id: 'app.screenshots.selection.global-export-filename',
		defaultMessage: 'Modrinth screenshots.zip',
	},
	instanceExportFilename: {
		id: 'app.screenshots.selection.instance-export-filename',
		defaultMessage: '{instance} screenshots.zip',
	},
	bulkDeleteTitle: {
		id: 'app.screenshots.selection.delete-title',
		defaultMessage: 'Delete selected screenshots',
	},
	bulkDeleteDescription: {
		id: 'app.screenshots.selection.delete-description',
		defaultMessage:
			'Delete {count, plural, one {# screenshot} other {# screenshots}}? This action cannot be undone.',
	},
	bulkDeleteSuccess: {
		id: 'app.screenshots.selection.delete-success',
		defaultMessage: '{count, plural, one {# screenshot deleted} other {# screenshots deleted}}',
	},
})

const screenshotsQuery = useQuery(
	computed(() =>
		props.instanceId
			? instanceScreenshotsQueryOptions(props.instanceId)
			: syncedScreenshotsQueryOptions(),
	),
)
const instancesQuery = useQuery(instanceListQueryOptions())
const screenshotGroupsQuery = useQuery(screenshotGroupsQueryOptions())
const screenshotsQueryPending = useReadyState(screenshotsQuery)
const screenshotGroupsQueryPending = useReadyState(screenshotGroupsQuery)
const screenshotsReadyPending = computed(
	() =>
		screenshotsQueryPending.value ||
		(groupBy.value === 'custom' &&
			(screenshotGroupsQueryPending.value || migratingLegacyGroups.value)),
)
const screenshots = computed(() => screenshotsQuery.data.value ?? [])
const customGroups = computed(() => screenshotGroupsQuery.data.value ?? [])
const screenshotOptionsInstance = computed(() =>
	(instancesQuery.data.value ?? []).find(
		(instance) => instance.id === screenshotOptionsTarget.value?.instance_id,
	),
)
const screenshotsError = computed(() => {
	const error =
		screenshotsQuery.error.value ||
		(groupBy.value === 'custom' ? screenshotGroupsQuery.error.value : null)
	return error instanceof Error ? error : error ? new Error(String(error)) : null
})
const selectionActive = computed(() => selectedKeys.value.size > 0)
const selectedScreenshots = computed(() =>
	screenshots.value.filter((screenshot) => selectedKeys.value.has(getSelectionKey(screenshot))),
)
const selectedGroupedScreenshots = computed(() =>
	selectedScreenshots.value.filter((screenshot) => screenshot.group_id),
)
const activeDraggedKeys = computed(() => new Set(activeDrag.value?.selectionKeys ?? []))
const activeDraggedScreenshots = computed(() => {
	const drag = activeDrag.value
	if (!drag) return []

	const screenshotsByKey = new Map(
		screenshots.value.map((screenshot) => [getSelectionKey(screenshot), screenshot]),
	)
	return drag.selectionKeys.flatMap((selectionKey) => {
		const screenshot = screenshotsByKey.get(selectionKey)
		return screenshot ? [screenshot] : []
	})
})
const activeDraggedScreenshot = computed(() =>
	activeDrag.value
		? screenshots.value.find(
				(screenshot) => getSelectionKey(screenshot) === activeDrag.value?.primarySelectionKey,
			)
		: undefined,
)
const {
	items: gatherItems,
	target: gatherTarget,
	isGathering,
	start: startGather,
	updateTarget: updateGatherTarget,
	clear: clearGather,
	finish: finishGather,
} = useScreenshotDragGather(screenshots)
const isNarrowingResults = computed(() => search.value.trim().length > 0)

const sortOptions = computed<ComboboxOption<string>[]>(() => [
	{ value: 'newest', label: formatMessage(messages.newest) },
	{ value: 'oldest', label: formatMessage(messages.oldest) },
	{ value: 'name', label: formatMessage(messages.name) },
])
const groupOptions = computed<ComboboxOption<string>[]>(() => [
	{ value: 'custom', label: formatMessage(messages.custom) },
	...(isGlobal.value ? [{ value: 'instance', label: formatMessage(messages.instance) }] : []),
	{ value: 'date', label: formatMessage(messages.date) },
	{ value: 'none', label: formatMessage(messages.none) },
])

const filteredScreenshots = computed(() => {
	const query = search.value.trim().toLocaleLowerCase()
	const filtered = screenshots.value.filter((screenshot) => {
		if (screenshot.id === revealedScreenshotId.value) return true
		return (
			!query ||
			screenshot.file_name.toLocaleLowerCase().includes(query) ||
			screenshot.instance_name.toLocaleLowerCase().includes(query)
		)
	})

	return filtered.sort((a, b) => {
		if (sort.value === 'name') return a.file_name.localeCompare(b.file_name)
		const difference = new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
		return sort.value === 'oldest' ? difference : -difference
	})
})

const groupedScreenshots = computed((): ScreenshotGroupData[] => {
	if (groupBy.value === 'none') {
		return [
			{
				id: 'all',
				title: formatMessage(messages.allScreenshots),
				screenshots: filteredScreenshots.value,
			},
		]
	}

	if (groupBy.value === 'instance') {
		const screenshotGroups = new Map<string, InstanceScreenshot[]>()
		for (const screenshot of filteredScreenshots.value) {
			const group = screenshotGroups.get(screenshot.instance_id) ?? []
			group.push(screenshot)
			screenshotGroups.set(screenshot.instance_id, group)
		}

		const syncedInstances = (instancesQuery.data.value ?? [])
			.filter((instance) => instance.synced_options.screenshots)
			.sort((a, b) => a.name.localeCompare(b.name))
		const groups = syncedInstances.flatMap((instance) => {
			const instanceScreenshots = screenshotGroups.get(instance.id)
			return instanceScreenshots
				? [
						{
							id: `instance:${instance.id}`,
							title: instance.name,
							screenshots: instanceScreenshots,
							dropInstanceId: instance.id,
						},
					]
				: []
		})

		for (const [instanceId, instanceScreenshots] of screenshotGroups) {
			if (groups.some((group) => group.dropInstanceId === instanceId)) continue
			groups.push({
				id: `instance:${instanceId}`,
				title: instanceScreenshots[0]?.instance_name ?? instanceId,
				screenshots: instanceScreenshots,
				dropInstanceId: instanceId,
			})
		}

		return groups
	}

	if (groupBy.value === 'custom') {
		const screenshotGroups = new Map<string, InstanceScreenshot[]>()
		const ungroupedScreenshots: InstanceScreenshot[] = []
		const validGroupIds = new Set(customGroups.value.map((group) => group.id))
		for (const screenshot of filteredScreenshots.value) {
			const customGroupId = screenshot.group_id
			if (!customGroupId || !validGroupIds.has(customGroupId)) {
				ungroupedScreenshots.push(screenshot)
				continue
			}
			const group = screenshotGroups.get(customGroupId) ?? []
			group.push(screenshot)
			screenshotGroups.set(customGroupId, group)
		}

		const groups: ScreenshotGroupData[] = customGroups.value.map((group) => ({
			id: `custom:${group.id}`,
			title: group.name,
			screenshots: screenshotGroups.get(group.id) ?? [],
			customGroupId: group.id,
		}))
		groups.push({
			id: 'custom:ungrouped',
			title: formatMessage(messages.ungrouped),
			screenshots: ungroupedScreenshots,
			customGroupId: null,
		})
		return isNarrowingResults.value
			? groups.filter(
					(group) =>
						group.screenshots.length > 0 || group.customGroupId === groupIdPendingNameEdit.value,
				)
			: groups
	}

	const groups = new Map<string, ScreenshotGroupData>()
	for (const screenshot of filteredScreenshots.value) {
		const dateGroup = getDateGroup(screenshot.created_at)
		const group = groups.get(dateGroup.id) ?? { ...dateGroup, screenshots: [] }
		group.screenshots.push(screenshot)
		groups.set(dateGroup.id, group)
	}
	return [...groups.values()].sort((a, b) => {
		const aTime = Math.max(
			...a.screenshots.map((screenshot) => new Date(screenshot.created_at).getTime()),
		)
		const bTime = Math.max(
			...b.screenshots.map((screenshot) => new Date(screenshot.created_at).getTime()),
		)
		return sort.value === 'oldest' ? aTime - bTime : bTime - aTime
	})
})

const screenshotColumnCount = computed(() => {
	if (windowWidth.value >= 1536) return 4
	if (windowWidth.value >= 640) return 2
	return 1
})

const screenshotCardWidth = computed(() => {
	if (screenshotListWidth.value <= 0) return FALLBACK_SCREENSHOT_CARD_WIDTH
	const gapsWidth = (screenshotColumnCount.value - 1) * SCREENSHOT_GRID_GAP
	return Math.max(0, (screenshotListWidth.value - gapsWidth) / screenshotColumnCount.value)
})

const screenshotCardHeight = computed(() => (screenshotCardWidth.value * 9) / 16)
const screenshotRowHeight = computed(() => screenshotCardHeight.value + SCREENSHOT_GRID_GAP)

const screenshotGroupLayouts = computed<ScreenshotGroupLayout[]>(() => {
	const layouts: ScreenshotGroupLayout[] = []
	let top = 0

	for (const group of groupedScreenshots.value) {
		const isHeaderHidden = groupBy.value === 'none'
		const isOpen = isHeaderHidden || search.value.length > 0 || !collapsedGroups.value[group.id]
		const rowCount = Math.ceil(group.screenshots.length / screenshotColumnCount.value)
		const gridHeight =
			rowCount === 0
				? SCREENSHOT_GRID_MIN_HEIGHT
				: rowCount * screenshotCardHeight.value + Math.max(0, rowCount - 1) * SCREENSHOT_GRID_GAP
		const headerHeight = isHeaderHidden ? 0 : SCREENSHOT_GROUP_HEADER_HEIGHT
		const gridTop = top + headerHeight + SCREENSHOT_GROUP_CONTENT_SPACING
		const height =
			headerHeight +
			(isOpen ? SCREENSHOT_GROUP_CONTENT_SPACING + gridHeight : 0) +
			SCREENSHOT_GROUP_SPACING

		layouts.push({ group, top, height, isOpen, gridHeight, gridTop })
		top += height
	}

	return layouts
})

const screenshotListHeight = computed(() => {
	const lastGroup = screenshotGroupLayouts.value[screenshotGroupLayouts.value.length - 1]
	return lastGroup ? lastGroup.top + lastGroup.height : 0
})

const visibleScreenshotGroups = computed<VisibleScreenshotGroupLayout[]>(() => {
	const hasViewport = Boolean(screenshotListContainer.value && screenshotScrollContainer.value)
	const viewportStart = hasViewport
		? Math.max(0, screenshotListScrollTop.value - SCREENSHOT_GROUP_OVERSCAN)
		: 0
	const viewportEnd = hasViewport
		? screenshotListScrollTop.value + screenshotViewportHeight.value + SCREENSHOT_GROUP_OVERSCAN
		: SCREENSHOT_GROUP_OVERSCAN

	return screenshotGroupLayouts.value
		.filter((layout) => layout.top + layout.height >= viewportStart && layout.top <= viewportEnd)
		.map((layout) => {
			if (!layout.isOpen || layout.group.screenshots.length === 0) {
				return { ...layout, renderedScreenshots: [], virtualGridTop: 0 }
			}

			const rowCount = Math.ceil(layout.group.screenshots.length / screenshotColumnCount.value)
			const firstRow = Math.min(
				rowCount,
				Math.max(0, Math.floor((viewportStart - layout.gridTop) / screenshotRowHeight.value)),
			)
			const lastRow = Math.min(
				rowCount,
				Math.max(
					firstRow,
					Math.ceil(
						(viewportEnd - layout.gridTop + SCREENSHOT_GRID_GAP) / screenshotRowHeight.value,
					),
				),
			)
			const firstScreenshot = firstRow * screenshotColumnCount.value
			const lastScreenshot = lastRow * screenshotColumnCount.value

			return {
				...layout,
				renderedScreenshots: layout.group.screenshots.slice(firstScreenshot, lastScreenshot),
				virtualGridTop: firstRow * screenshotRowHeight.value,
			}
		})
})

const previewItems = computed(() =>
	filteredScreenshots.value.map((screenshot) => ({
		id: getSelectionKey(screenshot),
		src: screenshot.url,
		alt: screenshot.file_name,
		title: screenshot.file_name,
		editorSource: {
			id: getSelectionKey(screenshot),
			path: screenshot.path,
		},
		description: isGlobal.value
			? formatMessage(messages.instanceAndDate, {
					instance: screenshot.instance_name,
					date: formatDateTime(screenshot.created_at),
				})
			: formatDateTime(screenshot.created_at),
	})),
)

const deleteMutation = useMutation({
	mutationFn: (keys: ScreenshotKey[]) => delete_screenshots(keys),
	onSuccess: async (_, keys) => {
		await invalidateScreenshots(keys.map((key) => key.instance_id))
	},
})
const exportMutation = useMutation({
	mutationFn: ({ keys, path }: { keys: ScreenshotKey[]; path: string }) =>
		export_screenshots(keys, path),
})
const moveMutation = useMutation({
	mutationFn: ({ keys, targetInstanceId }: { keys: ScreenshotKey[]; targetInstanceId: string }) =>
		move_screenshots(keys, targetInstanceId),
	onSuccess: async (_, variables) => {
		await invalidateScreenshots([
			...variables.keys.map((key) => key.instance_id),
			variables.targetInstanceId,
		])
		selectedKeys.value = new Set()
	},
})
const saveEditMutation = useMutation({
	mutationFn: (payload: {
		screenshot: InstanceScreenshot
		pngBytes: Uint8Array
		mode: 'create_copy' | 'replace_edit'
	}) =>
		save_edited_screenshot(getScreenshotKey(payload.screenshot), payload.pngBytes, payload.mode),
	onSuccess: async (saved) => {
		await invalidateScreenshots([saved.instance_id])
		await imageViewer.value?.markSavedAndView(getSelectionKey(saved))
		await revealScreenshot(saved.id)
	},
	onError: handleError,
})

function saveScreenshotEdit(payload: ImageViewerEditorSavePayload) {
	const screenshot = screenshotBySelectionKey(payload.item.id)
	if (!screenshot) return
	saveEditMutation.mutate({
		screenshot,
		pngBytes: payload.pngBytes,
		mode: payload.mode,
	})
}

const bulkBusy = computed(
	() =>
		deleteMutation.isPending.value ||
		exportMutation.isPending.value ||
		moveMutation.isPending.value ||
		creatingCustomGroup.value ||
		updatingCustomGroupMemberships.value,
)

watch(screenshots, (currentScreenshots) => {
	const currentKeys = new Set(currentScreenshots.map(getSelectionKey))
	selectedKeys.value = new Set([...selectedKeys.value].filter((key) => currentKeys.has(key)))
})

watch(groupBy, async (currentGroupBy, previousGroupBy) => {
	if (currentGroupBy !== 'custom') {
		groupIdPendingNameEdit.value = undefined
	}
	if (currentGroupBy === previousGroupBy) return

	const previousPositions = getScreenshotCardPositions()
	regrouping.value = true
	await nextTick()
	animateScreenshotCardsFrom(previousPositions)
	regrouping.value = false
})

let handledFocus: string | undefined

watch(
	[() => route.query.focus, screenshots],
	([focus]) => {
		if (typeof focus !== 'string') {
			handledFocus = undefined
			return
		}
		if (handledFocus === focus) return
		if (!screenshots.value.some((screenshot) => screenshot.id === focus)) return
		handledFocus = focus
		void revealScreenshot(focus)
	},
	{ immediate: true },
)

watch(
	() => screenshotGroupsQuery.isSuccess.value,
	(groupsLoaded) => {
		if (groupsLoaded) void migrateLegacyScreenshotGroups()
	},
	{ immediate: true },
)

useAppEvent('instance', (event) => {
	if (event.event !== 'screenshots_updated') return
	void invalidateScreenshots([event.instance_id])
})

function getSelectionKey(screenshot: InstanceScreenshot) {
	return JSON.stringify([screenshot.instance_id, screenshot.file_name])
}

function getScreenshotCardPositions() {
	const positions = new Map<string, DOMRect>()
	const cards = screenshotsPage.value?.querySelectorAll<HTMLElement>('[data-screenshot-card]') ?? []
	for (const card of cards) {
		const selectionKey = card.dataset.selectionKey
		if (selectionKey) positions.set(selectionKey, card.getBoundingClientRect())
	}
	return positions
}

function animateScreenshotCardsFrom(previousPositions: Map<string, DOMRect>) {
	if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return

	const cards = screenshotsPage.value?.querySelectorAll<HTMLElement>('[data-screenshot-card]') ?? []
	for (const card of cards) {
		const selectionKey = card.dataset.selectionKey
		const previousPosition = selectionKey ? previousPositions.get(selectionKey) : undefined
		if (!previousPosition) continue

		const currentPosition = card.getBoundingClientRect()
		const translateX = previousPosition.left - currentPosition.left
		const translateY = previousPosition.top - currentPosition.top
		if (translateX === 0 && translateY === 0) continue

		card.animate(
			[
				{ transform: `translate(${translateX}px, ${translateY}px)` },
				{ transform: 'translate(0, 0)' },
			],
			{ duration: 200, easing: 'ease-out' },
		)
	}
}

function getScreenshotKey(screenshot: InstanceScreenshot): ScreenshotKey {
	return {
		instance_id: screenshot.instance_id,
		file_name: screenshot.file_name,
	}
}

async function migrateLegacyScreenshotGroups() {
	const legacy = legacyCustomGrouping.value
	if (migratingLegacyGroups.value || legacy.groups.length === 0) return

	migratingLegacyGroups.value = true
	try {
		const allScreenshots = await list_all_screenshots()
		const screenshotIdsByLegacyKey = new Map(
			allScreenshots.map((screenshot) => [getSelectionKey(screenshot), screenshot.id]),
		)
		const screenshotIdsByGroupId = new Map<string, string[]>()
		for (const [legacyKey, groupId] of Object.entries(legacy.assignments)) {
			const screenshotId = screenshotIdsByLegacyKey.get(legacyKey)
			if (!screenshotId) continue
			const screenshotIds = screenshotIdsByGroupId.get(groupId) ?? []
			screenshotIds.push(screenshotId)
			screenshotIdsByGroupId.set(groupId, screenshotIds)
		}
		const groups: ScreenshotGroupImport[] = legacy.groups.map((group) => ({
			...group,
			screenshot_ids: screenshotIdsByGroupId.get(group.id) ?? [],
		}))
		await import_screenshot_groups(groups)
		legacyCustomGrouping.value = { groups: [], assignments: {} }
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: screenshotKeys.groups() }),
			invalidateScreenshots(allScreenshots.map((screenshot) => screenshot.instance_id)),
		])
	} catch (error) {
		handleError(error)
	} finally {
		migratingLegacyGroups.value = false
	}
}

function getDefaultCustomGroupName() {
	const existingNames = new Set(
		customGroups.value.map((group) => group.name.trim().toLocaleLowerCase()),
	)
	let groupNumber = customGroups.value.length + 1
	while (existingNames.has(`group ${groupNumber}`.toLocaleLowerCase())) {
		groupNumber += 1
	}
	return `Group ${groupNumber}`
}

async function createCustomGroup() {
	if (creatingCustomGroup.value) return
	creatingCustomGroup.value = true
	try {
		const screenshotsToGroup = [...selectedScreenshots.value]
		const group = await create_screenshot_group(
			getDefaultCustomGroupName(),
			screenshotsToGroup.map((screenshot) => screenshot.id),
		)
		queryClient.setQueryData<ScreenshotGroup[]>(screenshotKeys.groups(), (groups = []) => [
			group,
			...groups.filter((existingGroup) => existingGroup.id !== group.id),
		])
		await invalidateScreenshots([
			...new Set(screenshotsToGroup.map((screenshot) => screenshot.instance_id)),
		])
		groupBy.value = 'custom'
		selectedKeys.value = new Set()
		groupIdPendingNameEdit.value = group.id
	} catch (error) {
		handleError(error)
	} finally {
		creatingCustomGroup.value = false
	}
}

function validateCustomGroupName(value: string) {
	const normalizedGroupName = value.trim()
	return (
		normalizedGroupName.length > 0 && normalizedGroupName.length <= MAX_INSTANCE_GROUP_NAME_LENGTH
	)
}

async function renameCustomGroup(groupId: string | null | undefined, name: string) {
	if (!groupId) return false
	try {
		const group = await rename_screenshot_group(groupId, name)
		queryClient.setQueryData<ScreenshotGroup[]>(screenshotKeys.groups(), (groups = []) =>
			groups.map((existingGroup) => (existingGroup.id === group.id ? group : existingGroup)),
		)
		return true
	} catch {
		return false
	}
}

async function requestCustomGroupDeletion(groupId: string) {
	const group = customGroups.value.find((candidate) => candidate.id === groupId)
	if (!group) return

	customGroupToDelete.value = group
	try {
		const allScreenshots = await list_all_screenshots()
		if (allScreenshots.some((screenshot) => screenshot.group_id === groupId)) {
			deleteGroupModal.value?.show()
		} else {
			await deleteCustomGroup()
		}
	} catch (error) {
		handleError(error)
	}
}

async function deleteCustomGroup() {
	const group = customGroupToDelete.value
	if (!group) return
	try {
		await delete_screenshot_group(group.id)
		queryClient.setQueryData<ScreenshotGroup[]>(screenshotKeys.groups(), (groups = []) =>
			groups.filter((candidate) => candidate.id !== group.id),
		)
		await invalidateScreenshots(screenshots.value.map((screenshot) => screenshot.instance_id))
		if (groupIdPendingNameEdit.value === group.id) {
			groupIdPendingNameEdit.value = undefined
		}
		customGroupToDelete.value = undefined
	} catch (error) {
		handleError(error)
	}
}

async function assignCustomGroup(
	screenshotsToMove: InstanceScreenshot[],
	customGroupId: string | null,
) {
	const movedScreenshots = screenshotsToMove.filter(
		(screenshot) => (screenshot.group_id ?? null) !== customGroupId,
	)
	if (movedScreenshots.length === 0 || updatingCustomGroupMemberships.value) return
	updatingCustomGroupMemberships.value = true
	try {
		await set_screenshot_group_memberships(
			movedScreenshots.map((screenshot) => ({
				screenshot_id: screenshot.id,
				group_id: customGroupId,
			})),
		)
		await invalidateScreenshots(movedScreenshots.map((screenshot) => screenshot.instance_id))
		selectedKeys.value = new Set()
	} catch (error) {
		handleError(error)
	} finally {
		updatingCustomGroupMemberships.value = false
	}
}

function removeSelectedScreenshotsFromGroups() {
	if (bulkBusy.value || selectedGroupedScreenshots.value.length === 0) return
	void assignCustomGroup(selectedGroupedScreenshots.value, null)
}

function getDateGroup(createdAt: string): Omit<ScreenshotGroupData, 'screenshots'> {
	const created = dayjs(createdAt)
	const now = dayjs()
	if (created.isSame(now, 'day')) {
		return { id: 'date:today', title: formatMessage(messages.today) }
	}
	if (created.isSame(now.subtract(1, 'day'), 'day')) {
		return { id: 'date:yesterday', title: formatMessage(messages.yesterday) }
	}
	if (created.isSame(now, 'week')) {
		return {
			id: `date:week:${created.startOf('week').format('YYYY-MM-DD')}`,
			title: formatMessage(messages.thisWeek),
		}
	}
	if (created.isSame(now, 'month')) {
		return {
			id: `date:month:${created.format('YYYY-MM')}`,
			title: formatMessage(messages.thisMonth),
		}
	}
	return {
		id: `date:month:${created.format('YYYY-MM')}`,
		title: formatMonth(created.toDate()),
	}
}

async function invalidateScreenshots(instanceIds: string[]) {
	const uniqueInstanceIds = [...new Set(instanceIds)]
	await Promise.all([
		queryClient.invalidateQueries({ queryKey: screenshotKeys.global() }),
		...uniqueInstanceIds.map((instanceId) =>
			queryClient.invalidateQueries({ queryKey: screenshotKeys.instance(instanceId) }),
		),
	])
}

function toggleScreenshotSelection(screenshot: InstanceScreenshot) {
	if (bulkBusy.value) return
	const key = getSelectionKey(screenshot)
	const next = new Set(selectedKeys.value)
	if (next.has(key)) {
		next.delete(key)
	} else {
		next.add(key)
	}
	selectedKeys.value = next
}

function activateScreenshot(screenshot: InstanceScreenshot, event: MouseEvent | KeyboardEvent) {
	if (selectionActive.value || event.shiftKey) {
		toggleScreenshotSelection(screenshot)
		return
	}
	const index = filteredScreenshots.value.findIndex(
		(candidate) => getSelectionKey(candidate) === getSelectionKey(screenshot),
	)
	if (index >= 0) {
		screenshotOptionsMenu.value?.close()
		imageViewer.value?.show(index)
	}
}

function clearSelection() {
	if (!bulkBusy.value) selectedKeys.value = new Set()
}

function requestDelete(screenshot: InstanceScreenshot, fromPreview = false) {
	deleteFromPreview.value = fromPreview
	screenshotToDelete.value = screenshot
	deleteModal.value?.show()
}

function showScreenshotOptions(screenshot: InstanceScreenshot, event: MouseEvent) {
	screenshotOptionsTarget.value = screenshot
	const options: ButtonMenuOption[] = [
		...(event.type === 'contextmenu'
			? [
					{
						id: 'edit',
						label: formatMessage(messages.edit),
						icon: EditIcon,
						action: () => editScreenshot(screenshot),
					},
					{
						id: 'copy',
						label: formatMessage(messages.copy),
						icon: ClipboardCopyIcon,
						action: () => void copyScreenshot(screenshot),
					},
				]
			: []),
		{
			id: 'open',
			label: formatMessage(messages.showInFolder),
			icon: FolderOpenIcon,
			action: () => void openScreenshot(screenshot),
		},
		{
			id: 'go-to-instance',
			label: formatMessage(messages.goToInstance),
			action: () => void goToInstance(screenshot),
		},
		{ type: 'divider' },
		{
			id: 'delete',
			label: formatMessage(commonMessages.deleteLabel),
			icon: TrashIcon,
			tone: 'red',
			action: () => requestDelete(screenshot),
		},
	]
	screenshotOptionsMenu.value?.open(event, options)
}

function goToInstance(screenshot: InstanceScreenshot) {
	return router.push(`/instance/${encodeURIComponent(screenshot.instance_id)}`)
}

function screenshotBySelectionKey(selectionKey: string) {
	return screenshots.value.find((screenshot) => getSelectionKey(screenshot) === selectionKey)
}

function copyScreenshotBySelectionKey(selectionKey: string) {
	const screenshot = screenshotBySelectionKey(selectionKey)
	if (screenshot) void copyScreenshot(screenshot)
}

function isScreenshotCopiedBySelectionKey(selectionKey: string) {
	const screenshot = screenshotBySelectionKey(selectionKey)
	return screenshot ? copiedScreenshotIds.value.has(screenshot.id) : false
}

function openScreenshotBySelectionKey(selectionKey: string) {
	const screenshot = screenshotBySelectionKey(selectionKey)
	if (screenshot) void openScreenshot(screenshot)
}

function requestPreviewDelete(selectionKey: string) {
	const screenshot = screenshotBySelectionKey(selectionKey)
	if (screenshot) requestDelete(screenshot, true)
}

function editScreenshot(screenshot: InstanceScreenshot) {
	const index = filteredScreenshots.value.findIndex(
		(candidate) => getSelectionKey(candidate) === getSelectionKey(screenshot),
	)
	if (index >= 0) {
		screenshotOptionsMenu.value?.close()
		void imageViewer.value?.edit(index)
	}
}

let revealTimeout: ReturnType<typeof setTimeout> | undefined

async function revealScreenshot(id: string) {
	if (revealTimeout) clearTimeout(revealTimeout)
	revealedScreenshotId.value = id
	await nextTick()

	const group = groupedScreenshots.value.find((candidate) =>
		candidate.screenshots.some((screenshot) => screenshot.id === id),
	)
	if (group) setGroupCollapsed(group.id, false)

	await nextTick()
	await waitForScreenshotViewport()
	highlightedScreenshotId.value = id

	const layout = screenshotGroupLayouts.value.find((candidate) => candidate.group.id === group?.id)
	const screenshotIndex = layout?.group.screenshots.findIndex((screenshot) => screenshot.id === id)
	const scrollTarget = screenshotScrollContainer.value
	if (layout && screenshotIndex !== undefined && screenshotIndex >= 0 && scrollTarget) {
		const row = Math.floor(screenshotIndex / screenshotColumnCount.value)
		const top = Math.max(
			0,
			screenshotListOffset.value +
				layout.gridTop +
				row * screenshotRowHeight.value +
				screenshotCardHeight.value / 2 -
				screenshotViewportHeight.value / 2,
		)
		scrollTarget.scrollTo({ top, behavior: 'smooth' })
	}

	const card = await waitForScreenshotCard(id)
	card?.focus({ preventScroll: true })
	revealTimeout = setTimeout(() => {
		if (highlightedScreenshotId.value === id) highlightedScreenshotId.value = undefined
		if (revealedScreenshotId.value === id) revealedScreenshotId.value = undefined
		revealTimeout = undefined
	}, 2400)
}

async function waitForScreenshotViewport() {
	for (let frame = 0; frame < 10; frame++) {
		if (screenshotListContainer.value && screenshotScrollContainer.value) return
		await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
	}
}

async function waitForScreenshotCard(id: string) {
	const selector = `[data-screenshot-id="${CSS.escape(id)}"]`
	for (let frame = 0; frame < 60; frame++) {
		const card = screenshotListContainer.value?.querySelector<HTMLElement>(selector)
		if (card) return card
		await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()))
	}
	return undefined
}

async function confirmDelete() {
	const screenshot = screenshotToDelete.value
	if (!screenshot) return
	try {
		await deleteMutation.mutateAsync([getScreenshotKey(screenshot)])
		if (deleteFromPreview.value) imageViewer.value?.hide()
		addNotification({ type: 'success', title: formatMessage(messages.deleteSuccess) })
	} catch (error) {
		handleError(error)
	} finally {
		screenshotToDelete.value = null
		deleteFromPreview.value = false
	}
}

async function deleteSelected() {
	const selected = selectedScreenshots.value
	if (bulkBusy.value || selected.length === 0) return
	try {
		await deleteMutation.mutateAsync(selected.map(getScreenshotKey))
		selectedKeys.value = new Set()
		addNotification({
			type: 'success',
			title: formatMessage(messages.bulkDeleteSuccess, { count: selected.length }),
		})
	} catch (error) {
		handleError(error)
	}
}

async function exportSelected() {
	const selected = selectedScreenshots.value
	if (bulkBusy.value || selected.length === 0) return
	const instanceName =
		selected[0]?.instance_name.replace(/[\\/:*?"<>|]/g, '-') ?? formatMessage(messages.heading)
	const outputPath = await save({
		defaultPath: isGlobal.value
			? formatMessage(messages.globalExportFilename)
			: formatMessage(messages.instanceExportFilename, { instance: instanceName }),
		filters: [{ name: formatMessage(messages.zipArchive), extensions: ['zip'] }],
	})
	if (!outputPath) return
	try {
		await exportMutation.mutateAsync({
			keys: selected.map(getScreenshotKey),
			path: outputPath,
		})
	} catch (error) {
		handleError(error)
	}
}

async function copyScreenshot(screenshot: InstanceScreenshot) {
	try {
		const png = readFile(screenshot.path).then((bytes) => new Blob([bytes], { type: 'image/png' }))
		await navigator.clipboard.write([new ClipboardItem({ 'image/png': png })])
		markScreenshotCopied(screenshot.id)
	} catch (error) {
		handleError(error)
	}
}

function markScreenshotCopied(id: string) {
	copiedScreenshotIds.value = new Set([...copiedScreenshotIds.value, id])
	const existingTimeout = copiedResetTimeouts.get(id)
	if (existingTimeout) clearTimeout(existingTimeout)
	copiedResetTimeouts.set(
		id,
		setTimeout(() => {
			const nextCopiedScreenshotIds = new Set(copiedScreenshotIds.value)
			nextCopiedScreenshotIds.delete(id)
			copiedScreenshotIds.value = nextCopiedScreenshotIds
			copiedResetTimeouts.delete(id)
		}, 2000),
	)
}

async function openScreenshot(screenshot: InstanceScreenshot) {
	try {
		await open_screenshot(getScreenshotKey(screenshot))
	} catch (error) {
		handleError(error)
	}
}

function handleDragStart(event: DragStartEvent) {
	const source = event.operation.source?.data as ScreenshotDragData | undefined
	if (!source) return

	const selectedDragKeys = selectedKeys.value.has(source.selectionKey)
		? selectedScreenshots.value.map(getSelectionKey)
		: []
	const selectionKeys = [
		source.selectionKey,
		...selectedDragKeys.filter((selectionKey) => selectionKey !== source.selectionKey),
	]
	activeDrag.value = {
		primarySelectionKey: source.selectionKey,
		selectionKeys,
	}
	startGather(activeDrag.value, event.operation.position.current)
}

function handleDragMove(event: DragMoveEvent) {
	if (gatherItems.value.length === 0) return
	updateGatherTarget(event.to ?? event.operation.position.current)
}

function handleDragOver(event: DragOverEvent) {
	const target = event.operation.target?.data as ScreenshotDropData | undefined
	activeDropGroupId.value = target?.groupId ?? null
}

function canDropScreenshotsOnTarget(target: ScreenshotDropData) {
	if (!activeDrag.value || activeDraggedScreenshots.value.length === 0) return false

	if ('customGroupId' in target) {
		const targetGroupId = target.customGroupId ?? null
		return activeDraggedScreenshots.value.some(
			(screenshot) => (screenshot.group_id ?? null) !== targetGroupId,
		)
	}

	return Boolean(
		target.instanceId &&
		activeDraggedScreenshots.value.some(
			(screenshot) => screenshot.instance_id !== target.instanceId,
		),
	)
}

function canDropScreenshotsOnGroup(group: ScreenshotGroupData) {
	if (groupBy.value === 'custom') {
		return canDropScreenshotsOnTarget({
			groupId: group.id,
			customGroupId: group.customGroupId ?? null,
		})
	}
	if (groupBy.value === 'instance' && group.dropInstanceId) {
		return canDropScreenshotsOnTarget({
			groupId: group.id,
			instanceId: group.dropInstanceId,
		})
	}
	return false
}

async function handleDragEnd(event: DragEndEvent) {
	const drag = activeDrag.value
	const target = event.operation.target?.data as ScreenshotDropData | undefined
	clearGroupHoverOpenTimeout()
	if (!event.canceled && drag && target && canDropScreenshotsOnTarget(target)) {
		if ('customGroupId' in target) {
			await assignCustomGroup(activeDraggedScreenshots.value, target.customGroupId ?? null)
			await nextTick()
		} else if (target.instanceId) {
			const keys = activeDraggedScreenshots.value.map(getScreenshotKey)
			const movableKeys = keys.filter((key) => key.instance_id !== target.instanceId)
			if (movableKeys.length > 0) {
				moveMutation.mutate({ keys: movableKeys, targetInstanceId: target.instanceId })
			}
		}
	}

	clearGather()
	activeDrag.value = null
	activeDropGroupId.value = null
}

function setGroupCollapsed(groupId: string, collapsed: boolean) {
	collapsedGroups.value = { ...collapsedGroups.value, [groupId]: collapsed }
}

const GROUP_HOVER_OPEN_DELAY = 750
let groupHoverOpenTimeout: ReturnType<typeof setTimeout> | undefined

function clearGroupHoverOpenTimeout() {
	if (groupHoverOpenTimeout !== undefined) {
		clearTimeout(groupHoverOpenTimeout)
	}
	groupHoverOpenTimeout = undefined
}

watch(activeDropGroupId, (groupId) => {
	clearGroupHoverOpenTimeout()
	if (!groupId || !collapsedGroups.value[groupId]) return

	const group = groupedScreenshots.value.find((candidate) => candidate.id === groupId)
	if (!group || !canDropScreenshotsOnGroup(group)) return

	groupHoverOpenTimeout = setTimeout(() => {
		groupHoverOpenTimeout = undefined
		if (activeDropGroupId.value === groupId) {
			setGroupCollapsed(groupId, false)
		}
	}, GROUP_HOVER_OPEN_DELAY)
})

onBeforeUnmount(() => {
	clearGroupHoverOpenTimeout()
	if (revealTimeout) clearTimeout(revealTimeout)
	if (screenshotsScrollIdleTimeout) clearTimeout(screenshotsScrollIdleTimeout)
	for (const timeout of copiedResetTimeouts.values()) clearTimeout(timeout)
	copiedResetTimeouts.clear()
})
</script>

<template>
	<ConfirmModal
		ref="deleteModal"
		:title="formatMessage(messages.deleteTitle)"
		:description="
			formatMessage(messages.deleteDescription, {
				name: screenshotToDelete?.file_name ?? '',
			})
		"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		:markdown="false"
		@proceed="confirmDelete"
	/>
	<ConfirmModal
		ref="bulkDeleteModal"
		:title="formatMessage(messages.bulkDeleteTitle)"
		:description="formatMessage(messages.bulkDeleteDescription, { count: selectedKeys.size })"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		:markdown="false"
		@proceed="deleteSelected"
	/>
	<ConfirmModal
		ref="deleteGroupModal"
		:title="formatMessage(messages.deleteGroup)"
		:description="formatMessage(messages.deleteGroupDescription)"
		:proceed-label="formatMessage(commonMessages.deleteLabel)"
		:markdown="false"
		@proceed="deleteCustomGroup"
	/>
	<ContextMenu ref="screenshotOptionsMenu" :label="formatMessage(commonMessages.actionsLabel)">
		<template #go-to-instance>
			<Avatar
				:src="getInstanceIconUrl(screenshotOptionsInstance?.icon_path)"
				:tint-by="screenshotOptionsTarget?.instance_id"
				alt=""
				size="1rem"
				class="shrink-0"
			/>
			{{ formatMessage(messages.goToInstance) }}
		</template>
	</ContextMenu>
	<ImageViewerEditor
		ref="imageViewer"
		:items="previewItems"
		editor="enabled"
		:saving="saveEditMutation.isPending.value"
		@save="saveScreenshotEdit"
	>
		<template #actions="{ item }">
			<IconButton
				v-tooltip="
					formatMessage(isScreenshotCopiedBySelectionKey(item.id) ? messages.copied : messages.copy)
				"
				:label="
					formatMessage(isScreenshotCopiedBySelectionKey(item.id) ? messages.copied : messages.copy)
				"
				type="quiet"
				@click="copyScreenshotBySelectionKey(item.id)"
			>
				<CheckIcon v-if="isScreenshotCopiedBySelectionKey(item.id)" class="text-green" />
				<ClipboardCopyIcon v-else />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(messages.showInFolder)"
				:label="formatMessage(messages.showInFolder)"
				type="quiet"
				@click="openScreenshotBySelectionKey(item.id)"
			>
				<FolderOpenIcon />
			</IconButton>
			<IconButton
				v-tooltip="formatMessage(commonMessages.deleteLabel)"
				:label="formatMessage(commonMessages.deleteLabel)"
				type="quiet"
				class="hover:!bg-red focus-visible:!bg-red hover:!text-[var(--color-accent-contrast)] focus-visible:!text-[var(--color-accent-contrast)]"
				@click="requestPreviewDelete(item.id)"
			>
				<TrashIcon />
			</IconButton>
		</template>
	</ImageViewerEditor>

	<div
		ref="screenshotsPage"
		class="flex h-full w-full flex-col gap-3"
		:class="{ 'justify-center': screenshots.length === 0 && !groupIdPendingNameEdit }"
	>
		<template v-if="screenshots.length > 0 || groupIdPendingNameEdit">
			<h1 v-if="showHeading" class="m-0 text-2xl font-bold text-contrast">
				{{ formatMessage(messages.heading) }}
			</h1>

			<ScreenshotToolbar
				v-model:search="search"
				v-model:sort="sortModel"
				v-model:group="groupByModel"
				:sort-options="sortOptions"
				:group-options="groupOptions"
				@new-group="createCustomGroup"
			/>
		</template>

		<ReadyTransition :pending="screenshotsReadyPending">
			<EmptyState
				v-if="screenshotsError"
				type="error"
				:heading="formatMessage(messages.errorHeading)"
				:description="screenshotsError.message"
			>
				<template #actions>
					<Button type="outlined" @click="screenshotsQuery.refetch()">
						{{ formatMessage(commonMessages.retryButton) }}
					</Button>
				</template>
			</EmptyState>

			<EmptyState
				v-else-if="screenshots.length === 0 && !groupIdPendingNameEdit"
				type="no-images"
				:heading="formatMessage(messages.emptyHeading)"
				:description="formatMessage(messages.emptyDescription)"
			/>
			<EmptyState
				v-else-if="filteredScreenshots.length === 0 && !groupIdPendingNameEdit"
				type="no-images"
				:heading="formatMessage(messages.noResultsHeading)"
				:description="formatMessage(messages.noResultsDescription)"
			/>

			<DragDropProvider
				v-else
				@drag-start="handleDragStart"
				@drag-move="handleDragMove"
				@drag-over="handleDragOver"
				@drag-end="handleDragEnd"
			>
				<div
					ref="screenshotListContainer"
					class="relative w-full"
					:style="{ height: `${screenshotListHeight}px`, overflowAnchor: 'none' }"
				>
					<div
						v-for="{
							group,
							top,
							gridHeight,
							renderedScreenshots,
							virtualGridTop,
						} in visibleScreenshotGroups"
						:key="group.id"
						class="absolute inset-x-0 transition-transform duration-300 ease-in-out will-change-transform motion-reduce:transition-none"
						:style="{ transform: `translateY(${top}px)` }"
					>
						<ScreenshotGroupSection
							:id="group.id"
							:title="group.title"
							:screenshots="group.screenshots"
							:rendered-screenshots="renderedScreenshots"
							:virtual-grid-height="gridHeight"
							:virtual-grid-top="virtualGridTop"
							:selected-keys="selectedKeys"
							:selection-active="selectionActive"
							:active-dragged-keys="activeDraggedKeys"
							:show-drop-outline="
								activeDropGroupId === group.id && canDropScreenshotsOnGroup(group)
							"
							:can-drag="groupBy === 'custom' || (isGlobal && groupBy === 'instance')"
							:drop-instance-id="groupBy === 'instance' ? group.dropInstanceId : undefined"
							:drop-custom-group="groupBy === 'custom'"
							:drop-custom-group-id="group.customGroupId ?? undefined"
							:show-instance-name="isGlobal && groupBy !== 'instance'"
							:highlighted-screenshot-id="highlightedScreenshotId"
							:copied-screenshot-ids="copiedScreenshotIds"
							:animate-entry="!regrouping && !screenshotsScrolling"
							:force-open="search.length > 0"
							:hide-header="groupBy === 'none'"
							:editable-title="Boolean(group.customGroupId)"
							:start-editing-title="groupIdPendingNameEdit === group.customGroupId"
							:max-title-length="MAX_INSTANCE_GROUP_NAME_LENGTH"
							:validate-title="validateCustomGroupName"
							:on-title-change="(name: string) => renameCustomGroup(group.customGroupId, name)"
							:collapsed="Boolean(collapsedGroups[group.id])"
							@update:collapsed="(value) => setGroupCollapsed(group.id, value)"
							@activate="activateScreenshot"
							@toggle-selection="toggleScreenshotSelection"
							@copy="copyScreenshot"
							@edit="editScreenshot"
							@more="showScreenshotOptions"
						>
							<template v-if="group.customGroupId" #actions="{ startEditing }">
								<div
									class="flex shrink-0 items-center opacity-0 transition-opacity duration-250 group-hover/header:opacity-100 focus-within:opacity-100"
								>
									<IconButton
										v-tooltip="formatMessage(messages.editGroup)"
										:label="formatMessage(messages.editGroup)"
										type="quiet"
										size="sm"
										@click.stop="startEditing"
									>
										<EditIcon />
									</IconButton>
									<IconButton
										v-tooltip="formatMessage(messages.deleteGroup)"
										:label="formatMessage(messages.deleteGroup)"
										type="quiet"
										size="sm"
										@click.stop="requestCustomGroupDeletion(group.customGroupId)"
									>
										<TrashIcon />
									</IconButton>
								</div>
							</template>
						</ScreenshotGroupSection>
					</div>
				</div>
				<Teleport to="body">
					<div class="pointer-events-none fixed inset-0 z-[9999]">
						<DragOverlay :drop-animation="null">
							<div
								v-if="activeDraggedScreenshot"
								class="w-full transition-all duration-150 ease-out"
								:class="isGathering ? 'scale-[0.975]' : 'scale-100'"
							>
								<ScreenshotDragPreview
									:screenshot="activeDraggedScreenshot"
									:count="activeDraggedScreenshots.length"
								/>
							</div>
						</DragOverlay>
					</div>
				</Teleport>
			</DragDropProvider>
			<ScreenshotDragGather
				v-if="gatherItems.length > 0"
				:items="gatherItems"
				:target="gatherTarget"
				@complete="finishGather"
			/>
		</ReadyTransition>
	</div>

	<FloatingActionBar
		:shown="selectionActive"
		:aria-label="formatMessage(messages.selectionAriaLabel)"
		hide-when-modal-open
	>
		<div class="flex items-center gap-0.5">
			<span class="px-4 py-2.5 text-base font-semibold tabular-nums text-contrast">
				{{ formatMessage(messages.selectedCount, { count: selectedKeys.size }) }}
			</span>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<Button
				v-tooltip="formatMessage(commonMessages.clearButton)"
				type="quiet"
				:aria-label="formatMessage(commonMessages.clearButton)"
				:disabled="bulkBusy"
				@click="clearSelection"
			>
				<XIcon class="hidden cq-show-icon" />
				<span class="bar-label">{{ formatMessage(commonMessages.clearButton) }}</span>
			</Button>
		</div>
		<div class="ml-auto flex items-center gap-0.5">
			<Button
				v-tooltip="formatMessage(messages.newGroup)"
				type="quiet"
				:aria-label="formatMessage(messages.newGroup)"
				:disabled="bulkBusy"
				@click="createCustomGroup"
			>
				<SquarePlusIcon />
				<span class="bar-label">{{ formatMessage(messages.newGroup) }}</span>
			</Button>
			<Button
				v-if="selectedGroupedScreenshots.length > 0"
				v-tooltip="formatMessage(messages.removeFromGroup)"
				type="quiet"
				:aria-label="formatMessage(messages.removeFromGroup)"
				:disabled="bulkBusy"
				@click="removeSelectedScreenshotsFromGroups"
			>
				<MinusIcon />
				<span class="bar-label">{{ formatMessage(messages.removeFromGroup) }}</span>
			</Button>
			<Button
				v-tooltip="formatMessage(messages.exportZip)"
				type="quiet"
				:aria-label="formatMessage(messages.exportZip)"
				:disabled="bulkBusy"
				@click="exportSelected"
			>
				<FileArchiveIcon />
				<span class="bar-label">{{ formatMessage(messages.exportZip) }}</span>
			</Button>
			<div class="mx-1 h-6 w-px bg-surface-5" />
			<Button
				v-tooltip="formatMessage(commonMessages.deleteLabel)"
				type="quiet"
				color="red"
				interaction="filled"
				:aria-label="formatMessage(commonMessages.deleteLabel)"
				:disabled="bulkBusy"
				@click="bulkDeleteModal?.show()"
			>
				<TrashIcon />
				<span class="bar-label">{{ formatMessage(commonMessages.deleteLabel) }}</span>
			</Button>
		</div>
	</FloatingActionBar>
</template>
