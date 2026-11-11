<script setup lang="ts">
import { ChevronDownIcon, LoaderCircleIcon } from '@modrinth/assets'
import { IconButton } from '@modrinth/ui'
import { StorageSerializers, useLocalStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { slicerEntry } from '../archive-path'
import CodeViewer from '../code-viewer/index.vue'
import FileActions from '../file-actions.vue'
import HeightDivider from '../height-divider.vue'
import { severityOrder, truncateMiddle } from '../helpers'
import PanelDivider from '../panel-divider.vue'
import type { SourceLinkTarget, SourceView } from '../source-link'
import { isUsableDecompiledSource } from '../source-status'
import TraceList from '../trace-list/index.vue'
import type { ClassGroup, FlattenedFileReport, TraceVerdictEvent } from '../types'
import { injectTechReviewDecisions } from '../use-tech-review-decisions'
import { useTracesNotificationOffset } from '../use-traces-notification-offset'

const props = defineProps<{
	file: FlattenedFileReport
	classItem: ClassGroup
	expanded: boolean
	splitView?: boolean
	focusedTraceId?: string | null
	sourceLink?: SourceLinkTarget
	loadingIssues: Set<string>
	decompiledSources: Map<string, string>
}>()

const tracesWidth = defineModel<number>('tracesWidth', { default: 360 })
const classPane = ref<HTMLElement>()
const tracesPane = ref<HTMLElement>()
useTracesNotificationOffset(tracesPane, () => !!props.splitView, classPane)

const tracesHeight = useLocalStorage<number | null>('moderation-tech-review-traces-height', null, {
	serializer: StorageSerializers.number,
})

const emit = defineEmits<{
	toggle: []
	verdict: [event: TraceVerdictEvent]
	sourceView: [view: SourceView]
	sourceLinkHandled: [target: SourceLinkTarget]
}>()

const { getMarkedFlagsCount } = injectTechReviewDecisions()
const codeViewer = ref<InstanceType<typeof CodeViewer>>()

const showHiddenTraces = ref(false)

const visibleFlags = computed(() =>
	props.classItem.flags
		.filter((flag) => showHiddenTraces.value || flag.detail.severity !== 'hidden')
		.sort((a, b) => severityOrder[b.detail.severity] - severityOrder[a.detail.severity]),
)

watch(
	() => props.focusedTraceId,
	(id) => {
		const flag = props.classItem.flags.find((flag) => String(flag.detail.id) === id)
		if (flag?.detail.severity === 'hidden') showHiddenTraces.value = true
	},
	{ immediate: true },
)

const truncatedPath = computed(() => truncateMiddle(props.classItem.filePath))
const pathTooltip = computed(() =>
	truncatedPath.value !== props.classItem.filePath ? props.classItem.filePath : undefined,
)

const allFlagsMarked = computed(
	() => getMarkedFlagsCount(props.classItem.flags) === props.classItem.flags.length,
)

const isLoadingSource = computed(() =>
	props.classItem.flags.some((flag) => props.loadingIssues.has(flag.issueId)),
)

const decompiledSource = computed(() => {
	const sources = props.classItem.flags.map((flag) => props.decompiledSources.get(flag.detail.id))
	return sources.find(isUsableDecompiledSource) ?? sources.find((source) => source?.trim())
})

const highlightData = computed(() => visibleFlags.value.map((flag) => flag.detail.data))

function canHighlight(value: string) {
	return codeViewer.value?.canHighlight(value) ?? false
}

function selectSourceValue(value: string) {
	return codeViewer.value?.selectSourceValue(value)
}

defineExpose({
	navigateHighlight: (direction: number) => codeViewer.value?.navigateHighlight(direction) ?? false,
	canNavigateHighlight: () => codeViewer.value?.canNavigateHighlight() ?? false,
})
</script>

<template>
	<div
		ref="classPane"
		:class="
			splitView
				? 'file-review-panes grid min-h-0 min-w-0 gap-y-1'
				: 'flex min-h-0 min-w-0 flex-col gap-1'
		"
		:style="splitView ? { '--traces-width': `${tracesWidth}px` } : undefined"
	>
		<div
			class="flex min-w-0 shrink-0 items-center justify-between gap-2"
			@click="!splitView && emit('toggle')"
		>
			<div class="flex min-w-0 grow items-center gap-2 pl-4">
				<IconButton
					v-if="!splitView"
					type="quiet"
					label="Toggle details"
					class="transition-transform"
					:class="{ 'rotate-180': expanded }"
				>
					<ChevronDownIcon class="h-5 w-5 text-contrast" />
				</IconButton>

				<div class="group/file-actions flex min-w-0 items-center gap-1">
					<span
						v-tooltip="pathTooltip"
						class="min-w-0 truncate font-mono text-sm font-semibold text-primary"
						:class="{ 'opacity-50': allFlagsMarked }"
					>
						{{ truncatedPath }}
					</span>
					<FileActions :file="file" :entry-path="slicerEntry(file.download_url, classItem)" />
				</div>

				<div class="ml-auto flex items-center gap-2">
					<Transition name="fade">
						<div v-if="isLoadingSource" class="text-secondary">
							<span class="flex items-center gap-1.5 text-sm font-medium text-secondary">
								<LoaderCircleIcon class="size-4 animate-spin" />
								Loading source...
							</span>
						</div>
					</Transition>
				</div>
			</div>
		</div>

		<div v-if="expanded" class="contents">
			<aside
				ref="tracesPane"
				:class="
					splitView
						? 'col-start-3 row-span-2 row-start-1 flex min-h-0 min-w-0 flex-col pb-1 pl-1'
						: 'contents'
				"
				:aria-label="splitView ? 'Traces' : undefined"
			>
				<div
					:class="splitView ? 'flex min-h-0 flex-col overflow-hidden' : 'contents'"
					:style="
						splitView && $slots.conversation
							? {
									height: tracesHeight === null ? 'calc(50% - 0.5rem)' : `${tracesHeight}px`,
									maxHeight: 'calc(50% - 0.5rem)',
									flexShrink: 0,
								}
							: { flex: 1 }
					"
				>
					<TraceList
						v-model:show-hidden-traces="showHiddenTraces"
						:flags="classItem.flags"
						:focused-trace-id="focusedTraceId"
						:scrollable="splitView"
						:can-highlight="canHighlight"
						@verdict="emit('verdict', $event)"
						@select-source-value="selectSourceValue"
					>
						<template #actions="{ flags }">
							<slot name="trace-actions" :flags="flags" />
						</template>
					</TraceList>
				</div>
				<HeightDivider v-if="$slots.conversation" v-model="tracesHeight" />
				<div v-if="$slots.conversation" class="flex min-h-0 flex-1 flex-col gap-2.5">
					<slot name="conversation" />
				</div>
			</aside>
			<PanelDivider
				v-if="splitView"
				v-model="tracesWidth"
				class="col-start-2 row-span-2 row-start-1"
				label="Traces width"
				width-property="--traces-width"
				side="right"
				:min="220"
				:max="720"
				:reserve="200"
				:default-width="360"
			/>
		</div>
		<CodeViewer
			ref="codeViewer"
			:report-id="file.id"
			:file-path="classItem.filePath"
			:jar="classItem.jar"
			:source="decompiledSource"
			:loading="isLoadingSource"
			:expanded="expanded"
			:scrollable="splitView"
			:source-link="sourceLink"
			:highlight-data="highlightData"
			@source-view="emit('sourceView', $event)"
			@source-link-handled="emit('sourceLinkHandled', $event)"
		/>
	</div>
</template>

<style scoped>
@property --traces-width {
	syntax: '<length>';
	inherits: false;
	initial-value: 360px;
}

.file-review-panes {
	grid-template-rows: auto minmax(0, 1fr);
	grid-template-columns: minmax(0, 1fr) 8px min(var(--traces-width), max(220px, calc(100% - 208px)));
}

.fade-enter-active {
	transition: opacity 0.3s ease-in;
	transition-delay: 0.2s;
}

.fade-leave-active {
	transition: opacity 0.15s ease-out;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>
