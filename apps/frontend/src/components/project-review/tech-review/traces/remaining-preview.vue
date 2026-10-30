<script setup lang="ts">
import { ChevronDownIcon, SearchIcon } from '@modrinth/assets'
import { Accordion, FloatingMenu, Input } from '@modrinth/ui'
import { useElementSize, useResizeObserver, useScroll } from '@vueuse/core'
import { computed, inject, nextTick, onBeforeUnmount, ref, watch } from 'vue'

import { getHighestSeverity, severityOrder } from '../helpers'
import { injectTechReviewMenus, registerFolderMenuChildKey } from '../menus'
import type { ClassGroup, FlagItem } from '../types'
import FilePath from './file-path.vue'
import TraceName from './name.vue'
import { traceName } from './presentation'

const props = defineProps<{ flags: FlagItem[]; label: string }>()
const emit = defineEmits<{ select: [fileKey: string] }>()
const menu = ref<InstanceType<typeof FloatingMenu>>()
const activateMenu = injectTechReviewMenus()
const registerWithFolder = inject(registerFolderMenuChildKey, undefined)
let unregisterFromFolder: (() => void) | undefined

function onOpen() {
	activateMenu(menu.value)
	unregisterFromFolder = registerWithFolder?.()
}

function onClose() {
	unregisterFromFolder?.()
	unregisterFromFolder = undefined
}

onBeforeUnmount(onClose)
const search = ref('')
const collapsed = ref(new Set<string>())
const groups = computed(() => {
	const types = new Map<string, FlagItem[]>()
	for (const flag of props.flags) {
		const flags = types.get(flag.issueType) ?? []
		flags.push(flag)
		types.set(flag.issueType, flags)
	}
	const term = search.value.trim().toLowerCase()
	return [...types]
		.map(([issueType, flags]) => {
			const files = new Map<string, ClassGroup>()
			for (const flag of flags) {
				if (
					!`${traceName(issueType)} ${issueType} ${flag.detail.file_path} ${flag.detail.jar ?? ''}`
						.toLowerCase()
						.includes(term)
				)
					continue
				const key = `${flag.detail.jar ?? ''}::${flag.detail.file_path}`
				let file = files.get(key)
				if (!file) {
					file = {
						key,
						jar: flag.detail.jar ?? null,
						filePath: flag.detail.file_path,
						flags: [],
					}
					files.set(key, file)
				}
				file.flags.push(flag)
			}
			return {
				issueType,
				severity: getHighestSeverity(flags.map((flag) => flag.detail)),
				files: [...files.values()],
			}
		})
		.filter((group) => group.files.length)
		.sort(
			(a, b) =>
				severityOrder[b.severity] - severityOrder[a.severity] ||
				a.issueType.localeCompare(b.issueType),
		)
})
const container = ref<HTMLElement>()
const { y: scrollTop, measure: measureScroll } = useScroll(container)
const { height: viewportHeight } = useElementSize(container)
const content = ref<HTMLElement>()
const { height: contentHeight } = useElementSize(content)
const groupElements = new Map<string, HTMLElement>()
const groupOffsets = ref(new Map<string, number>())

function measureGroups() {
	const viewport = container.value
	if (!viewport) return
	const offsets = new Map<string, number>()
	for (const [key, element] of groupElements) {
		let top = 0
		let current: HTMLElement | null = element
		while (current && current !== content.value) {
			top += current.offsetTop
			current = current.offsetParent as HTMLElement | null
		}
		offsets.set(key, top)
	}
	if (
		offsets.size !== groupOffsets.value.size ||
		[...offsets].some(([key, value]) => groupOffsets.value.get(key) !== value)
	) {
		groupOffsets.value = offsets
	}
}

function setGroupContent(key: string, element: HTMLElement | null) {
	if (groupElements.get(key) === element) return
	if (element) groupElements.set(key, element)
	else groupElements.delete(key)
	nextTick(measureGroups)
}

function syncLayout() {
	measureScroll()
	measureGroups()
}

watch(
	[container, content],
	([viewport, body]) => {
		if (!viewport || !body) {
			groupOffsets.value = new Map()
			return
		}
		syncLayout()
	},
	{ flush: 'post' },
)
useResizeObserver(content, syncLayout)
const visibleGroups = computed(() => {
	let offset = 0
	return groups.value.map((group) => {
		const contentTop = groupOffsets.value.get(group.issueType) ?? offset + 40
		const open = !collapsed.value.has(group.issueType)
		offset = contentTop + (open ? group.files.length * 40 : 0)
		const start = Math.min(
			group.files.length,
			Math.max(0, Math.floor((scrollTop.value - contentTop) / 40) - 5),
		)
		const end = Math.min(
			group.files.length,
			Math.max(start, Math.ceil((scrollTop.value + viewportHeight.value - contentTop) / 40) + 5),
		)
		return {
			...group,
			visibleFiles: group.files.slice(start, end),
			paddingTop: start * 40,
			paddingBottom: (group.files.length - end) * 40,
		}
	})
})
watch(search, () => {
	if (container.value) container.value.scrollTop = 0
})
function select(fileKey: string) {
	menu.value?.hide()
	emit('select', fileKey)
}
</script>

<template>
	<FloatingMenu
		ref="menu"
		placement="bottom-end"
		:arrow="false"
		bare
		@open="onOpen"
		@close="onClose"
	>
		<button
			type="button"
			class="custom-focus-indicator h-8 cursor-pointer whitespace-nowrap border-0 bg-transparent px-3 text-sm font-semibold text-secondary hover:bg-surface-4 hover:text-primary"
		>
			{{ label }}
		</button>
		<template #popper>
			<section
				aria-label="Remaining traces"
				class="review-scrollbars flex w-[500px] max-w-[calc(100vw-2rem)] flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
			>
				<Input
					v-model="search"
					:icon="SearchIcon"
					placeholder="Search files or issue types..."
					aria-label="Search files or issue types"
				/>
				<div
					ref="container"
					class="max-h-[60vh] overflow-y-auto overscroll-contain"
					:style="{ height: `${Math.min(contentHeight, 400)}px` }"
				>
					<div ref="content" class="relative">
						<Accordion
							v-for="group in visibleGroups"
							:key="group.issueType"
							:open-by-default="!collapsed.has(group.issueType)"
							button-class="custom-focus-indicator flex h-8 mt-2 w-full cursor-pointer items-center gap-2 border-0 bg-transparent py-0 pl-2 pr-2.5 text-left"
							@on-open="collapsed.delete(group.issueType)"
							@on-close="collapsed.add(group.issueType)"
						>
							<template #button="{ open }">
								<TraceName
									class="min-w-0 [&>span:last-child]:truncate"
									:title="traceName(group.issueType)"
									:issue-type="group.issueType"
									:severity="group.severity"
									colored
								/>
								<ChevronDownIcon
									class="size-4 shrink-0 text-secondary transition-transform"
									:class="{ '-rotate-90': !open }"
									aria-hidden="true"
								/>
								<span class="ml-auto min-w-16 shrink-0 text-right text-xs text-secondary"
									>{{ group.files.length }} {{ group.files.length === 1 ? 'file' : 'files' }}</span
								>
							</template>
							<div
								:ref="(element) => setGroupContent(group.issueType, element as HTMLElement | null)"
								:style="{
									paddingTop: `${group.paddingTop}px`,
									paddingBottom: `${group.paddingBottom}px`,
								}"
							>
								<button
									v-for="file in group.visibleFiles"
									:key="file.key"
									type="button"
									class="custom-focus-indicator flex h-10 w-full cursor-pointer items-center rounded-lg border-0 bg-transparent p-0 pl-2 text-left text-sm text-primary hover:bg-surface-3"
									:title="file.filePath"
									:aria-label="file.filePath"
									@click="select(file.key)"
								>
									<FilePath :path="file.filePath" />
								</button>
							</div>
						</Accordion>
					</div>
				</div>
				<p v-if="!groups.length" class="m-0 p-3 text-sm text-secondary">No matching traces</p>
			</section>
		</template>
	</FloatingMenu>
</template>
