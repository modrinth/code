<script setup lang="ts">
import {
	ChevronRightIcon,
	ChevronsDownUpIcon,
	ChevronsUpDownIcon,
	FileArchiveIcon,
	FileIcon,
	FilterIcon,
	FolderIcon,
	FolderOpenIcon,
	ListChevronsUpDownIcon,
	SearchIcon,
	ShieldAlertIcon,
	XIcon,
} from '@modrinth/assets'
import {
	type ButtonMenuOption,
	Checkbox,
	FloatingMenu,
	Input,
	MultiSelect,
	OverlayScrollArea,
	TagItem,
	TeleportOverflowMenu,
	Tooltip,
	truncatedTooltip,
} from '@modrinth/ui'
import { dismissFloatingMenus } from '@modrinth/ui/src/providers/floating-menu'
import { computed, nextTick, onBeforeUnmount, provide, ref, watch } from 'vue'

import { getHighestSeverity, severityOrder } from '../helpers'
import { folderMenuLockKey } from '../menus'
import TraceName from '../traces/name.vue'
import { severityPresentation, traceName } from '../traces/presentation'
import type { ClassGroup, FlagItem } from '../types'
import { injectTechReviewDecisions } from '../verdict/use-decisions'
import FolderMenu from './folder-menu.vue'
import ProjectCard from './project-card.vue'

const props = defineProps<{
	classes: ClassGroup[]
	traceTypes: string[]
	unavailableSources: Set<string>
	selectedKey?: string
	fileName: string
	globallyPassedCount: number
	hiddenCount: number
}>()
const showGloballyPassed = defineModel<boolean>('showGloballyPassed', {
	default: false,
})
const showHidden = defineModel<boolean>('showHidden', { default: false })
const treeElement = ref<HTMLElement>()
provide(folderMenuLockKey, ref<symbol>())

const emit = defineEmits<{ select: [classItem: ClassGroup] }>()
const { getDetailDecision, getMarkedFlagsCount } = injectTechReviewDecisions()

interface TreeNode {
	key: string
	name: string
	kind: 'folder' | 'archive' | 'file'
	children: Map<string, TreeNode>
	classItem?: ClassGroup
}

const previews = computed(
	() =>
		new Map(
			props.classes.map((classItem) => {
				const severity = getHighestSeverity(classItem.flags.map((flag) => flag.detail))
				const presentation = severityPresentation[severity]
				const unsafeCount = classItem.flags.filter(
					({ detail }) => getDetailDecision(detail.id, detail.status) === 'malware',
				).length
				const traceTypes = new Map<
					string,
					{
						issueType: string
						severity: FlagItem['detail']['severity']
						count: number
					}
				>()
				for (const flag of classItem.flags) {
					const trace = traceTypes.get(flag.issueType)
					if (trace) {
						trace.count++
						if (severityOrder[flag.detail.severity] > severityOrder[trace.severity]) {
							trace.severity = flag.detail.severity
						}
					} else {
						traceTypes.set(flag.issueType, {
							issueType: flag.issueType,
							severity: flag.detail.severity,
							count: 1,
						})
					}
				}
				return [
					classItem.key,
					{
						severityRank: severityOrder[severity],
						unsafeCount,
						unsafeLabel: `${unsafeCount} traces marked unsafe`,
						tagClass: severity === 'low' ? '!border-surface-5' : undefined,
						tagStyle: {
							'--_bg-color': presentation.bgColor,
							'--_color': presentation.color,
						},
						traceTypes: [...traceTypes.values()].sort(
							(a, b) => severityOrder[b.severity] - severityOrder[a.severity],
						),
					},
				]
			}),
		),
)

const hoveredName = ref<HTMLElement>()
const hoveredNameText = ref<string>()
let nameTooltipTimer: ReturnType<typeof setTimeout> | undefined

function showNameTooltip(event: MouseEvent | FocusEvent) {
	hideNameTooltip()
	const target = event.currentTarget as HTMLElement
	const name = target.hasAttribute('data-tree-name')
		? target
		: target.querySelector<HTMLElement>('[data-tree-name]')
	if (!name) return
	nameTooltipTimer = setTimeout(() => {
		const text = truncatedTooltip(name, name.textContent ?? '')
		hoveredName.value = text ? name : undefined
		hoveredNameText.value = text
	}, 750)
}

function hideNameTooltip() {
	clearTimeout(nameTooltipTimer)
	hoveredName.value = undefined
	hoveredNameText.value = undefined
}

onBeforeUnmount(hideNameTooltip)

const search = ref('')
const selectedTraceTypes = ref<string[]>([])
const selectedTraceTypeSet = computed(() => new Set(selectedTraceTypes.value))
const traceTypeCounts = computed(() => {
	const counts = new Map<string, number>()
	for (const classItem of props.classes) {
		for (const flag of classItem.flags) {
			counts.set(flag.issueType, (counts.get(flag.issueType) ?? 0) + 1)
		}
	}
	return counts
})
const traceTypeOptions = computed(() =>
	[...new Set(props.traceTypes)]
		.map((type) => ({ value: type, label: traceName(type) }))
		.sort((a, b) => a.label.localeCompare(b.label)),
)
const isFiltering = computed(() => !!search.value.trim() || selectedTraceTypes.value.length > 0)
const expanded = ref(new Map<string, boolean>())
const manuallyToggled = new Set<string>()
const expandFilterMatches = ref(true)
const filteredExpansion = ref(new Map<string, boolean>())
let automaticSelectionKey: string | undefined

watch([search, selectedTraceTypes], () => {
	expandFilterMatches.value = true
	filteredExpansion.value.clear()
})

function isExpanded(node: TreeNode) {
	if (isFiltering.value && expandFilterMatches.value) {
		return filteredExpansion.value.get(node.key) ?? true
	}
	return expanded.value.get(node.key) ?? !completedFolders.value.has(node.key)
}

function segmentKey(parentKey: string, segment: Pick<TreeNode, 'name' | 'kind'>) {
	return `${parentKey}/${segment.name}${segment.kind === 'archive' ? '#' : ''}`
}

function archiveSegments(classItem: ClassGroup) {
	const archives = (classItem.jar ?? '')
		.split('#')
		.filter(Boolean)
		.map((segment) => {
			try {
				return decodeURIComponent(segment)
			} catch {
				return segment
			}
		})
	if (archives[0] === props.fileName) archives.shift()
	return [...archives, classItem.filePath].flatMap((path, pathIndex) => {
		const names = path.split('/').filter(Boolean)
		return names.map<Pick<TreeNode, 'name' | 'kind'>>((name, index) => ({
			name,
			kind: index < names.length - 1 ? 'folder' : pathIndex < archives.length ? 'archive' : 'file',
		}))
	})
}

const folderCounts = computed(() => {
	const counts = new Map<
		string,
		{ total: number; marked: number; unsafe: number; flags: FlagItem[] }
	>()
	for (const classItem of props.classes) {
		const total = classItem.flags.length
		const marked = getMarkedFlagsCount(classItem.flags)
		const unsafe = previews.value.get(classItem.key)?.unsafeCount ?? 0
		let key = ''
		for (const segment of archiveSegments(classItem).slice(0, -1)) {
			key = segmentKey(key, segment)
			const count = counts.get(key) ?? {
				total: 0,
				marked: 0,
				unsafe: 0,
				flags: [],
			}
			count.total += total
			count.marked += marked
			count.unsafe += unsafe
			count.flags.push(...classItem.flags)
			counts.set(key, count)
		}
	}
	return counts
})

const completedFolders = computed(
	() =>
		new Set(
			[...folderCounts.value].flatMap(([key, { total, marked, unsafe }]) =>
				total > 0 && marked === total && unsafe === 0 ? [key] : [],
			),
		),
)

watch(completedFolders, (completed, previous) => {
	for (const key of previous) {
		if (!completed.has(key) && !manuallyToggled.has(key)) {
			expanded.value.delete(key)
		}
	}
	for (const key of completed) {
		if (!previous.has(key) && !manuallyToggled.has(key)) {
			expanded.value.set(key, false)
		}
	}
})

function setFolderExpansion(action: 'collapse-resolved' | 'collapse-all' | 'expand-all') {
	expandFilterMatches.value = false
	for (const key of folderCounts.value.keys()) {
		if (action === 'collapse-resolved' && !completedFolders.value.has(key)) continue
		manuallyToggled.add(key)
		expanded.value.set(key, action === 'expand-all')
	}
}

const folderActions: ButtonMenuOption[] = [
	{
		id: 'expand-all',
		label: 'Expand all',
		icon: ChevronsUpDownIcon,
		action: () => setFolderExpansion('expand-all'),
	},
	{
		id: 'collapse-all',
		label: 'Collapse all',
		icon: ChevronsDownUpIcon,
		action: () => setFolderExpansion('collapse-all'),
	},
	{
		id: 'collapse-resolved',
		label: 'Collapse resolved',
		icon: ChevronsDownUpIcon,
		action: () => setFolderExpansion('collapse-resolved'),
	},
]

const tree = computed(() => {
	const root: TreeNode = {
		key: '',
		name: '',
		kind: 'archive',
		children: new Map(),
	}
	const query = search.value.trim().toLowerCase()
	for (const classItem of props.classes) {
		if (
			selectedTraceTypeSet.value.size > 0 &&
			!classItem.flags.some((flag) => selectedTraceTypeSet.value.has(flag.issueType))
		)
			continue
		const segments = archiveSegments(classItem)
		if (
			query &&
			!segments
				.map((segment) => segment.name)
				.join('/')
				.toLowerCase()
				.includes(query)
		)
			continue
		let node = root
		for (const segment of segments) {
			const isFile = segment.kind === 'file'
			const key = isFile ? `file:${classItem.key}` : segmentKey(node.key, segment)
			if (!node.children.has(key)) {
				node.children.set(key, { key, ...segment, children: new Map() })
			}
			node = node.children.get(key)!
			if (isFile) node.classItem = classItem
		}
	}
	return root
})

function sortPriority(node: TreeNode, parent: TreeNode) {
	if (node.kind !== 'folder') return 4
	if (parent.kind !== 'archive') return 1
	if (node.name === 'META-INF') return 0
	if (node.name === 'assets') return 2
	if (node.name === 'data') return 3
	return 1
}

function sortedChildren(parent: TreeNode) {
	const children = [...parent.children.values()].sort(
		(a, b) =>
			sortPriority(a, parent) - sortPriority(b, parent) ||
			a.name.localeCompare(b.name, 'en', { numeric: true }),
	)
	const files = children
		.filter((node) => node.classItem)
		.sort(
			(a, b) =>
				(previews.value.get(b.classItem!.key)?.severityRank ?? -1) -
					(previews.value.get(a.classItem!.key)?.severityRank ?? -1) ||
				b.classItem!.flags.length - a.classItem!.flags.length ||
				a.name.localeCompare(b.name, 'en', { numeric: true }),
		)
	let fileIndex = 0
	return children.map((node) => (node.classItem ? files[fileIndex++] : node))
}

function firstFile(parent: TreeNode, unresolvedOnly = false): ClassGroup | undefined {
	for (const child of sortedChildren(parent)) {
		if (child.classItem) {
			if (
				!unresolvedOnly ||
				getMarkedFlagsCount(child.classItem.flags) < child.classItem.flags.length
			)
				return child.classItem
			continue
		}
		const file = firstFile(child, unresolvedOnly)
		if (file) return file
	}
}

watch(
	[() => props.classes, () => props.selectedKey],
	([classes, selectedKey]) => {
		if (classes.some((item) => item.key === selectedKey)) return
		const first = firstFile(tree.value, true) ?? firstFile(tree.value)
		if (first) {
			automaticSelectionKey = first.key
			emit('select', first)
		}
	},
	{ immediate: true },
)

const rows = computed(() => {
	const result: { node: TreeNode; depth: number }[] = []
	function visit(parent: TreeNode, depth: number) {
		const children = sortedChildren(parent)
		for (let node of children) {
			let name = node.name
			while (node.kind === 'folder' && node.children.size === 1) {
				const child = [...node.children.values()][0]
				if (child.kind !== 'folder') break
				name += `/${child.name}`
				node = child
			}
			result.push({ node: { ...node, name }, depth })
			if (!node.classItem && isExpanded(node)) {
				visit(node, depth + 1)
			}
		}
	}
	visit(tree.value, 0)
	return result
})

const orderedFiles = computed(() => {
	const files: ClassGroup[] = []
	function visit(parent: TreeNode) {
		for (const node of sortedChildren(parent)) {
			if (node.classItem) files.push(node.classItem)
			else visit(node)
		}
	}
	visit(tree.value)
	return files
})

function select(node: TreeNode) {
	dismissFloatingMenus()
	if (node.classItem) {
		automaticSelectionKey = undefined
		emit('select', node.classItem)
	} else {
		const nextExpanded = !isExpanded(node)
		if (isFiltering.value && expandFilterMatches.value) {
			filteredExpansion.value.set(node.key, nextExpanded)
		}
		manuallyToggled.add(node.key)
		expanded.value.set(node.key, nextExpanded)
	}
}

function getNextFile(direction: number) {
	const visibleKeys = new Set(
		rows.value.flatMap(({ node }) => (node.classItem ? [node.classItem.key] : [])),
	)
	const files = orderedFiles.value
	const index = files.findIndex((item) => item.key === props.selectedKey)
	const step = direction > 0 ? 1 : -1
	const start = index < 0 ? (step > 0 ? 0 : files.length - 1) : index + step
	for (let nextIndex = start; nextIndex >= 0 && nextIndex < files.length; nextIndex += step) {
		if (visibleKeys.has(files[nextIndex].key)) return files[nextIndex]
	}
}

function navigateFile(direction: number) {
	const next = getNextFile(direction)
	if (!next) return false
	emit('select', next)
	void nextTick(() => {
		treeElement.value?.querySelector('[aria-current="true"]')?.scrollIntoView({ block: 'nearest' })
	})
	return true
}

defineExpose({
	navigateFile,
	canNavigateFile: (direction: number) => !!getNextFile(direction),
})

watch(
	() => props.selectedKey,
	() => {
		dismissFloatingMenus()
		const selected = props.classes.find((item) => item.key === props.selectedKey)
		if (!selected) return
		if (
			selected.key === automaticSelectionKey &&
			getMarkedFlagsCount(selected.flags) === selected.flags.length
		)
			return
		let key = ''
		for (const segment of archiveSegments(selected).slice(0, -1)) {
			key = segmentKey(key, segment)
			filteredExpansion.value.set(key, true)
			expanded.value.set(key, true)
		}
	},
	{ immediate: true },
)
</script>

<template>
	<nav
		ref="treeElement"
		aria-label="Archive files"
		class="review-scrollbars flex min-h-0 min-w-0 flex-col"
	>
		<div class="flex shrink-0 items-center gap-0.5 pr-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				aria-label="Filter archive files"
				placeholder="Filter files..."
				size="small"
				wrapper-class="min-w-0 flex-1"
			/>
			<MultiSelect
				v-model="selectedTraceTypes"
				:options="traceTypeOptions"
				:searchable="traceTypeOptions.length > 6"
				search-placeholder="Search trace types..."
				show-selection-actions
				:show-chevron="false"
				:clearable="false"
				:dropdown-width="280"
				dropdown-align="right"
				trigger-type="quiet"
				trigger-size="sm"
				fit-content
				:trigger-class="`!size-8 !p-0 ${selectedTraceTypes.length ? 'text-brand' : ''}`"
			>
				<template #input-content>
					<FilterIcon class="size-4 shrink-0" aria-hidden="true" />
					<span class="sr-only">Filter by trace type</span>
					<span
						v-if="selectedTraceTypes.length"
						class="absolute right-0 top-0 text-[10px] leading-3"
						>{{ selectedTraceTypes.length }}</span
					>
				</template>
				<template #option="{ item, selected }">
					<div class="flex min-w-0 flex-1 items-center justify-between gap-3 text-sm">
						<span
							class="min-w-0 truncate font-semibold leading-tight"
							:class="selected ? 'text-contrast' : 'text-primary'"
							>{{ item.label }}</span
						>
						<span class="tabular-nums text-secondary">{{
							traceTypeCounts.get(item.value) ?? 0
						}}</span>
					</div>
				</template>
			</MultiSelect>
			<TeleportOverflowMenu
				type="quiet"
				size="sm"
				class="size-8 shrink-0"
				label="Folder actions"
				:circular="false"
				aria-label="Folder actions"
				placement="bottom-end"
				:options="folderActions"
			>
				<ListChevronsUpDownIcon class="size-4" aria-hidden="true" />
			</TeleportOverflowMenu>
		</div>
		<OverlayScrollArea :auto-hide="false" class="flex-1">
			<div class="py-2 pr-2.5">
				<p v-if="!rows.length" class="m-0 p-2 text-sm text-secondary">No matching files</p>
				<FolderMenu
					v-for="{ node, depth } in rows"
					:key="node.key"
					:show-delay="250"
					:disabled="!!node.classItem"
					class="!block min-w-0"
				>
					<button
						type="button"
						:aria-current="
							node.classItem && node.classItem.key === selectedKey ? 'true' : undefined
						"
						:aria-expanded="node.classItem ? undefined : isExpanded(node)"
						class="flex h-8 w-full min-w-0 items-center gap-1.5 rounded-md border-0 py-0 text-left text-sm hover:bg-surface-3 focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
						:class="
							node.classItem && node.classItem.key === selectedKey
								? 'bg-surface-3 text-contrast'
								: 'bg-transparent text-secondary'
						"
						:style="{ paddingLeft: `${depth * 12 + 6}px` }"
						@click="select(node)"
						@focus="showNameTooltip"
						@blur="hideNameTooltip"
					>
						<ChevronRightIcon
							class="size-3 shrink-0"
							:class="{
								invisible: node.classItem,
								'rotate-90': !node.classItem && isExpanded(node),
							}"
							aria-hidden="true"
						/>
						<component
							:is="
								node.classItem
									? FileIcon
									: node.kind === 'archive'
										? FileArchiveIcon
										: isExpanded(node)
											? FolderOpenIcon
											: FolderIcon
							"
							class="size-4 shrink-0"
							:class="{
								'text-yellow-500': node.kind === 'folder',
								'text-blue': node.kind === 'archive',
							}"
							aria-hidden="true"
						/>
						<span
							data-tree-name
							class="min-w-0 truncate text-xs"
							@mouseenter="showNameTooltip"
							@mouseleave="hideNameTooltip"
							>{{ node.name }}</span
						>
						<div
							v-if="node.classItem"
							class="ml-auto flex shrink-0 items-center gap-1 self-stretch px-1.5"
						>
							<span
								v-if="unavailableSources.has(node.classItem.key)"
								title="Source code unavailable or failed to decompile"
								class="inline-flex shrink-0"
								role="img"
								aria-label="Source code unavailable or failed to decompile"
							>
								<XIcon class="size-3 text-red" aria-hidden="true" />
							</span>
							<span
								v-if="previews.get(node.classItem.key)?.unsafeCount"
								v-tooltip="previews.get(node.classItem.key)?.unsafeLabel"
								class="inline-flex shrink-0"
								role="img"
								:aria-label="previews.get(node.classItem.key)?.unsafeLabel"
							>
								<ShieldAlertIcon class="size-[18px] text-red" aria-hidden="true" />
							</span>
							<FloatingMenu
								trigger="hover"
								:show-delay="250"
								class="items-center self-stretch"
								placement="right-start"
								:arrow="false"
								bare
							>
								<TagItem
									class="pointer-events-none h-[22px] min-w-[18px] shrink-0 justify-center !px-1 !py-0 !text-xs !font-medium !leading-none"
									:class="previews.get(node.classItem.key)?.tagClass"
									:style="previews.get(node.classItem.key)?.tagStyle"
								>
									<span class="[text-box:trim-both_cap_alphabetic]"
										>{{ getMarkedFlagsCount(node.classItem.flags) }}/{{
											node.classItem.flags.length
										}}</span
									>
								</TagItem>
								<template #popper>
									<div
										v-if="node.classItem"
										class="review-scrollbars w-max max-w-[calc(100vw-2rem)] rounded-lg border border-solid border-surface-4 bg-surface-2 p-3 text-sm shadow-lg"
									>
										<ul
											class="m-0 flex max-h-[min(60vh,24rem)] list-none flex-col gap-2 overflow-y-auto p-0"
											aria-label="Trace preview"
										>
											<li
												v-for="trace in previews.get(node.classItem.key)?.traceTypes"
												:key="trace.issueType"
												class="flex items-center gap-1"
											>
												<TraceName
													:issue-type="trace.issueType"
													:severity="trace.severity"
													colored
												/>
												<span
													v-if="trace.count > 1"
													class="font-medium"
													:class="severityPresentation[trace.severity].textClass"
													>({{ trace.count }})</span
												>
											</li>
										</ul>
									</div>
								</template>
							</FloatingMenu>
						</div>
					</button>
					<template v-if="!node.classItem" #popper>
						<section
							:aria-label="`Trace actions for ${node.name}`"
							class="review-scrollbars flex max-w-[calc(100vw-2rem)] flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3 shadow-2xl"
						>
							<div class="flex min-w-0 items-end justify-between gap-3">
								<h4 class="m-0 min-w-0 truncate text-sm font-semibold text-contrast">
									{{ node.name }}
								</h4>
								<span class="shrink-0 text-xs font-medium text-secondary">
									{{ folderCounts.get(node.key)?.total ?? 0 }} traces
								</span>
							</div>
							<slot name="folder-actions" :flags="folderCounts.get(node.key)?.flags ?? []" />
						</section>
					</template>
				</FolderMenu>
			</div>
		</OverlayScrollArea>
		<div class="flex shrink-0 flex-col gap-1 py-1.5 pl-0.5 pr-2">
			<Checkbox
				v-model="showGloballyPassed"
				class="!gap-1.5 !text-sm !text-primary [&>span:first-child]:!size-4 [&>span:first-child]:!rounded"
				:label="`Show ${globallyPassedCount} globally passed`"
			/>
			<Checkbox
				v-model="showHidden"
				class="!gap-1.5 !text-sm !text-primary [&>span:first-child]:!size-4 [&>span:first-child]:!rounded"
				:label="`Show ${hiddenCount} hidden`"
			/>
		</div>
		<ProjectCard />
		<Tooltip :reference="hoveredName ?? null" :text="hoveredNameText" />
	</nav>
</template>

<style scoped>
@property --review-scrollbar-color {
	syntax: '<color>';
	inherits: true;
	initial-value: transparent;
}

.review-scrollbars {
	--review-scrollbar-color: transparent;
	--overlay-scrollbar-color: var(--review-scrollbar-color);
	transition: --review-scrollbar-color 1750ms ease;
}

.review-scrollbars:hover {
	--review-scrollbar-color: var(--color-scrollbar);
	transition-duration: 200ms;
}

.review-scrollbars,
.review-scrollbars :deep(*) {
	scrollbar-color: var(--review-scrollbar-color) transparent;
}

.review-scrollbars::-webkit-scrollbar-thumb,
.review-scrollbars :deep(*::-webkit-scrollbar-thumb) {
	background: var(--review-scrollbar-color);
}

@media (hover: none) {
	.review-scrollbars {
		--review-scrollbar-color: var(--color-scrollbar);
	}
}

@media (prefers-reduced-motion: reduce) {
	.review-scrollbars {
		transition: none;
	}
}
</style>
