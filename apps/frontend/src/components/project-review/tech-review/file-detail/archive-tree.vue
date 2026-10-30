<script setup lang="ts">
import {
	ChevronRightIcon,
	FileIcon,
	FolderIcon,
	FolderOpenIcon,
	SearchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	Checkbox,
	FloatingMenu,
	Input,
	OverlayScrollArea,
	TagItem,
	Tooltip,
	truncatedTooltip,
} from '@modrinth/ui'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import { getHighestSeverity, severityOrder } from '../helpers'
import TraceName from '../traces/name.vue'
import { severityPresentation } from '../traces/presentation'
import type { ClassGroup, FlagItem } from '../types'
import { injectTechReviewDecisions } from '../verdict/use-decisions'
import ProjectCard from './project-card.vue'

const props = defineProps<{
	classes: ClassGroup[]
	unavailableSources: Set<string>
	selectedKey?: string
	fileName: string
	globallyPassedCount: number
	hiddenCount: number
}>()
const showGloballyPassed = defineModel<boolean>('showGloballyPassed', { default: false })
const showHidden = defineModel<boolean>('showHidden', { default: false })
const treeElement = ref<HTMLElement>()

const emit = defineEmits<{ select: [classItem: ClassGroup] }>()
const { getMarkedFlagsCount } = injectTechReviewDecisions()

interface TreeNode {
	key: string
	name: string
	children: Map<string, TreeNode>
	classItem?: ClassGroup
}

const previews = computed(
	() =>
		new Map(
			props.classes.map((classItem) => {
				const severity = getHighestSeverity(classItem.flags.map((flag) => flag.detail))
				const presentation = severityPresentation[severity]
				const traceTypes = new Map<
					string,
					{ issueType: string; severity: FlagItem['detail']['severity']; count: number }
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
						tagClass: severity === 'low' ? '!border-surface-5' : undefined,
						tagStyle: { '--_bg-color': presentation.bgColor, '--_color': presentation.color },
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
const collapsed = ref(new Set<string>())

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
	return [...archives, classItem.filePath].flatMap((path) => path.split('/').filter(Boolean))
}

const tree = computed(() => {
	const root: TreeNode = { key: '', name: '', children: new Map() }
	const query = search.value.trim().toLowerCase()
	for (const classItem of props.classes) {
		const segments = archiveSegments(classItem)
		if (query && !segments.join('/').toLowerCase().includes(query)) continue
		let node = root
		for (const [index, name] of segments.entries()) {
			const isFile = index === segments.length - 1
			const key = isFile ? `file:${classItem.key}` : `${node.key}/${name}`
			if (!node.children.has(key)) {
				node.children.set(key, { key, name, children: new Map() })
			}
			node = node.children.get(key)!
			if (isFile) node.classItem = classItem
		}
	}
	return root
})

const rows = computed(() => {
	const result: { node: TreeNode; depth: number }[] = []
	function visit(parent: TreeNode, depth: number) {
		const children = [...parent.children.values()].sort((a, b) =>
			Boolean(a.classItem) !== Boolean(b.classItem)
				? a.classItem
					? 1
					: -1
				: a.name.localeCompare(b.name, 'en', { numeric: true }),
		)
		for (let node of children) {
			let name = node.name
			while (!node.classItem && node.children.size === 1) {
				const child = [...node.children.values()][0]
				if (child.classItem) break
				name += `/${child.name}`
				node = child
			}
			result.push({ node: { ...node, name }, depth })
			if (!node.classItem && (search.value.trim() || !collapsed.value.has(node.key))) {
				visit(node, depth + 1)
			}
		}
	}
	visit(tree.value, 0)
	return result
})

function select(node: TreeNode) {
	if (node.classItem) {
		emit('select', node.classItem)
	} else if (collapsed.value.has(node.key)) {
		collapsed.value.delete(node.key)
	} else {
		collapsed.value.add(node.key)
	}
}

function navigateFile(direction: number) {
	const files = rows.value.flatMap(({ node }) => (node.classItem ? [node.classItem] : []))
	const index = files.findIndex((item) => item.key === props.selectedKey)
	const next = files[index < 0 ? (direction > 0 ? 0 : files.length - 1) : index + direction]
	if (!next) return false
	emit('select', next)
	void nextTick(() => {
		treeElement.value?.querySelector('[aria-current="true"]')?.scrollIntoView({ block: 'nearest' })
	})
	return true
}

defineExpose({ navigateFile })

watch(
	() => props.selectedKey,
	() => {
		const selected = props.classes.find((item) => item.key === props.selectedKey)
		if (!selected) return
		let key = ''
		for (const segment of archiveSegments(selected).slice(0, -1)) {
			key += `/${segment}`
			collapsed.value.delete(key)
		}
	},
)
</script>

<template>
	<nav
		ref="treeElement"
		aria-label="Archive files"
		class="review-scrollbars flex min-h-0 min-w-0 flex-col"
	>
		<div class="flex shrink-0 items-center gap-1.5 pr-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				aria-label="Filter archive files"
				placeholder="Filter files..."
				size="small"
				wrapper-class="min-w-0 flex-1"
			/>
		</div>
		<OverlayScrollArea :auto-hide="false" class="flex-1">
			<div class="py-2 pr-2.5">
				<p v-if="!rows.length" class="m-0 p-2 text-sm text-secondary">No matching files</p>
				<button
					v-for="{ node, depth } in rows"
					:key="node.key"
					type="button"
					:aria-current="node.classItem && node.classItem.key === selectedKey ? 'true' : undefined"
					:aria-expanded="node.classItem ? undefined : !!search.trim() || !collapsed.has(node.key)"
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
							'rotate-90': !node.classItem && (!!search.trim() || !collapsed.has(node.key)),
						}"
						aria-hidden="true"
					/>
					<component
						:is="
							node.classItem
								? FileIcon
								: !!search.trim() || !collapsed.has(node.key)
									? FolderOpenIcon
									: FolderIcon
						"
						class="size-4 shrink-0"
						:class="{ 'text-yellow-500': !node.classItem }"
						aria-hidden="true"
					/>
					<span
						data-tree-name
						class="min-w-0 truncate text-xs"
						@mouseenter="showNameTooltip"
						@mouseleave="hideNameTooltip"
						>{{ node.name }}</span
					>
					<FloatingMenu
						v-if="node.classItem"
						trigger="hover"
						:show-delay="250"
						class="ml-auto shrink-0 items-center gap-1 self-stretch px-1.5"
						placement="right-start"
						:arrow="false"
						bare
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
				</button>
			</div>
		</OverlayScrollArea>
		<div class="flex shrink-0 flex-col gap-1 py-3 pr-2">
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
