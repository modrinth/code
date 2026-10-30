<script setup lang="ts">
import { ChevronDownIcon, SearchIcon } from '@modrinth/assets'
import { FloatingMenu, Input } from '@modrinth/ui'
import { useVirtualList } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { getHighestSeverity, severityOrder } from '../helpers'
import { injectTechReviewMenus } from '../menus'
import type { ClassGroup, FlagItem } from '../types'
import FilePath from './file-path.vue'
import TraceName from './name.vue'
import { traceName } from './presentation'

const props = defineProps<{ flags: FlagItem[]; label: string }>()
const emit = defineEmits<{ select: [fileKey: string] }>()
const menu = ref<InstanceType<typeof FloatingMenu>>()
const activateMenu = injectTechReviewMenus()
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
type Row =
	| { kind: 'group'; key: string; group: (typeof groups.value)[number] }
	| { kind: 'file'; key: string; file: ClassGroup }
const rows = computed<Row[]>(() =>
	groups.value.flatMap((group) => [
		{ kind: 'group' as const, key: group.issueType, group },
		...(!collapsed.value.has(group.issueType)
			? group.files.map((file) => ({
					kind: 'file' as const,
					key: `${group.issueType}:${file.key}`,
					file,
				}))
			: []),
	]),
)
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(rows, {
	itemHeight: 40,
	overscan: 5,
})
watch(search, () => scrollTo(0))
watch(
	rows,
	() => {
		const container = containerProps.ref.value
		if (!container) return
		const lastStart = Math.max(0, rows.value.length - Math.ceil(container.clientHeight / 40))
		scrollTo(Math.min(Math.floor(container.scrollTop / 40), lastStart))
	},
	{ flush: 'post' },
)
function toggle(issueType: string) {
	if (collapsed.value.has(issueType)) collapsed.value.delete(issueType)
	else collapsed.value.add(issueType)
}
function select(fileKey: string) {
	menu.value?.hide()
	emit('select', fileKey)
}
</script>

<template>
	<FloatingMenu ref="menu" placement="bottom-end" :arrow="false" bare @open="activateMenu(menu)">
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
					v-bind="containerProps"
					class="max-h-[60vh] overscroll-contain"
					:style="{ height: `${Math.min(rows.length, 10) * 40}px` }"
				>
					<div v-bind="wrapperProps">
						<div v-for="item in list" :key="item.data.key" class="h-10">
							<button
								v-if="item.data.kind === 'group'"
								type="button"
								class="custom-focus-indicator flex h-full w-full cursor-pointer items-center gap-2 rounded-lg border-0 bg-surface-3 px-3 text-left hover:bg-surface-4"
								:aria-expanded="!collapsed.has(item.data.group.issueType)"
								@click="toggle(item.data.group.issueType)"
							>
								<ChevronDownIcon
									class="size-4 shrink-0"
									:class="{
										'-rotate-90': collapsed.has(item.data.group.issueType),
									}"
									aria-hidden="true"
								/>
								<TraceName
									:issue-type="item.data.group.issueType"
									:severity="item.data.group.severity"
									colored
								/>
								<span class="ml-auto shrink-0 text-xs text-secondary"
									>{{ item.data.group.files.length }}
									{{ item.data.group.files.length === 1 ? 'file' : 'files' }}</span
								>
							</button>
							<button
								v-else
								type="button"
								class="custom-focus-indicator flex h-full w-full cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent pl-9 pr-3 text-left text-sm text-primary hover:bg-surface-3"
								:title="item.data.file.filePath"
								:aria-label="item.data.file.filePath"
								@click="select(item.data.file.key)"
							>
								<FilePath :path="item.data.file.filePath" />
							</button>
						</div>
					</div>
				</div>
				<p v-if="!groups.length" class="m-0 p-3 text-sm text-secondary">No matching traces</p>
			</section>
		</template>
	</FloatingMenu>
</template>
