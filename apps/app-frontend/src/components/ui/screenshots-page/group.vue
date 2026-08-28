<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

import ScreenshotCard from './card.vue'
import ScreenshotSection from './section.vue'

const props = defineProps<{
	id: string
	title: string
	screenshots: InstanceScreenshot[]
	renderedScreenshots?: InstanceScreenshot[]
	virtualGridHeight?: number
	virtualGridTop?: number
	selectedKeys: ReadonlySet<string>
	selectionActive: boolean
	activeDraggedKeys: ReadonlySet<string>
	showDropOutline: boolean
	canDrag: boolean
	dropInstanceId?: string
	dropCustomGroup?: boolean
	dropCustomGroupId?: string
	showInstanceName: boolean
	highlightedScreenshotId?: string
	copiedScreenshotIds: ReadonlySet<string>
	forceOpen: boolean
	animateEntry: boolean
	hideHeader?: boolean
	editableTitle?: boolean
	startEditingTitle?: boolean
	maxTitleLength?: number
	validateTitle?: (value: string) => boolean
	onTitleChange?: (value: string) => boolean | void | Promise<boolean | void>
}>()

const collapsed = defineModel<boolean>('collapsed', { required: true })
const dropTarget = ref<HTMLElement>()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	emptyGroup: {
		id: 'app.screenshots.group.empty',
		defaultMessage: 'Drag and drop to add screenshots.',
	},
})

const shouldShowGrid = computed(() =>
	Boolean(props.hideHeader || props.forceOpen || !collapsed.value),
)
const visibleScreenshots = ref<InstanceScreenshot[]>(props.renderedScreenshots ?? props.screenshots)
const renderGrid = ref(shouldShowGrid.value)
const virtualGridStyle = computed(() =>
	props.virtualGridHeight === undefined ? undefined : { height: `${props.virtualGridHeight}px` },
)
const visibleGridStyle = computed(() =>
	props.virtualGridTop === undefined
		? undefined
		: { transform: `translateY(${props.virtualGridTop}px)` },
)
let unmountGridTimeout: ReturnType<typeof setTimeout> | undefined

watch(
	() => props.renderedScreenshots ?? props.screenshots,
	(screenshots) => {
		if (shouldShowGrid.value) visibleScreenshots.value = screenshots
	},
)

watch(
	shouldShowGrid,
	(showGrid, previouslyShown) => {
		if (unmountGridTimeout) clearTimeout(unmountGridTimeout)
		if (showGrid) {
			visibleScreenshots.value = props.renderedScreenshots ?? props.screenshots
			renderGrid.value = true
			return
		}
		if (!previouslyShown) {
			renderGrid.value = false
			return
		}
		unmountGridTimeout = setTimeout(() => {
			renderGrid.value = false
			unmountGridTimeout = undefined
		}, 300)
	},
	{ flush: 'post' },
)

onBeforeUnmount(() => {
	if (unmountGridTimeout) clearTimeout(unmountGridTimeout)
})

const emit = defineEmits<{
	(e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent | KeyboardEvent): void
	(e: 'toggle-selection' | 'copy' | 'edit', screenshot: InstanceScreenshot): void
	(e: 'more', screenshot: InstanceScreenshot, event: MouseEvent): void
}>()

useDroppable({
	id: computed(() => `screenshot-group:${props.id}`),
	element: dropTarget,
	disabled: computed(() => !props.dropInstanceId && !props.dropCustomGroup),
	data: computed(() =>
		props.dropCustomGroup
			? { groupId: props.id, customGroupId: props.dropCustomGroupId ?? null }
			: { groupId: props.id, instanceId: props.dropInstanceId },
	),
})

function getSelectionKey(screenshot: InstanceScreenshot) {
	return JSON.stringify([screenshot.instance_id, screenshot.file_name])
}
</script>

<template>
	<div
		ref="dropTarget"
		class="group/instance-container relative select-none pb-3 transition-colors"
	>
		<Transition
			enter-active-class="transition-opacity duration-150 ease-out"
			enter-from-class="!opacity-0"
			enter-to-class="opacity-100"
			leave-active-class="transition-opacity duration-150 ease-in"
			leave-from-class="opacity-100"
			leave-to-class="!opacity-0"
		>
			<div
				v-if="showDropOutline"
				class="pointer-events-none absolute -inset-2 inset-y-0 z-20 rounded-xl border-2 border-dashed border-contrast bg-transparent opacity-40"
			/>
		</Transition>
		<ScreenshotSection
			v-model:collapsed="collapsed"
			:title="title"
			:count="screenshots.length"
			:force-open="forceOpen"
			:hide-header="hideHeader"
			:editable="editableTitle"
			:start-editing="startEditingTitle"
			:max-title-length="maxTitleLength"
			:validate-title="validateTitle"
			:on-title-change="onTitleChange"
		>
			<template #actions="{ startEditing }">
				<slot name="actions" :start-editing="startEditing" />
			</template>
			<div v-if="renderGrid" class="relative min-h-[45px] w-full" :style="virtualGridStyle">
				<TransitionGroup
					tag="div"
					class="grid min-h-[45px] w-full grid-cols-1 gap-3 sm:grid-cols-2 2xl:grid-cols-4"
					:class="{ 'absolute inset-x-0 top-0': virtualGridHeight !== undefined }"
					:style="visibleGridStyle"
					move-class="transition-transform duration-200 ease-out motion-reduce:transition-none"
					:enter-active-class="
						animateEntry
							? 'transition-[opacity,transform] duration-[150ms] ease-out motion-reduce:transition-none'
							: ''
					"
					:enter-from-class="animateEntry ? 'opacity-0' : ''"
					enter-to-class="opacity-100 scale-100"
				>
					<ScreenshotCard
						v-for="screenshot in visibleScreenshots"
						:key="getSelectionKey(screenshot)"
						:screenshot="screenshot"
						:selection-key="getSelectionKey(screenshot)"
						:selected="selectedKeys.has(getSelectionKey(screenshot))"
						:selection-active="selectionActive"
						:active-dragged="activeDraggedKeys.has(getSelectionKey(screenshot))"
						:can-drag="canDrag"
						:show-instance-name="showInstanceName"
						:highlighted="highlightedScreenshotId === screenshot.id"
						:copied="copiedScreenshotIds.has(screenshot.id)"
						@activate="(event) => emit('activate', screenshot, event)"
						@toggle-selection="emit('toggle-selection', screenshot)"
						@copy="emit('copy', screenshot)"
						@edit="emit('edit', screenshot)"
						@more="(event) => emit('more', screenshot, event)"
					/>
					<p
						v-if="screenshots.length === 0"
						key="empty-group"
						class="col-span-full m-0 pl-0.5 pt-1 text-base font-base text-secondary opacity-80"
					>
						{{ formatMessage(messages.emptyGroup) }}
					</p>
				</TransitionGroup>
			</div>
		</ScreenshotSection>
	</div>
</template>
