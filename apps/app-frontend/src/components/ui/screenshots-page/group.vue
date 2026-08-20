<script setup lang="ts">
import { useDroppable } from '@dnd-kit/vue'
import { computed, ref } from 'vue'

import type { InstanceScreenshot } from '@/helpers/instance'

import ScreenshotCard from './card.vue'
import ScreenshotSection from './section.vue'

const props = defineProps<{
	id: string
	title: string
	screenshots: InstanceScreenshot[]
	selectedKeys: ReadonlySet<string>
	selectionActive: boolean
	activeDraggedKeys: ReadonlySet<string>
	showDropOutline: boolean
	canDrag: boolean
	dropInstanceId?: string
	dropCustomGroup?: boolean
	dropCustomGroupId?: string
	showInstanceName: boolean
	forceOpen: boolean
	hideHeader?: boolean
	editableTitle?: boolean
	startEditingTitle?: boolean
	maxTitleLength?: number
	validateTitle?: (value: string) => boolean
	onTitleChange?: (value: string) => boolean | void | Promise<boolean | void>
}>()

const collapsed = defineModel<boolean>('collapsed', { required: true })
const dropTarget = ref<HTMLElement>()

const emit = defineEmits<{
	(e: 'activate', screenshot: InstanceScreenshot, event: MouseEvent | KeyboardEvent): void
	(e: 'toggle-selection' | 'copy' | 'open' | 'delete', screenshot: InstanceScreenshot): void
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
			<TransitionGroup
				tag="div"
				class="grid min-h-[45px] w-full grid-cols-[repeat(auto-fill,minmax(min(15rem,100%),1fr))] gap-3 overflow-y-auto scroll-smooth"
				move-class="transition-transform duration-200 ease-out motion-reduce:transition-none"
				enter-active-class="transition-[opacity,transform] duration-[150ms] ease-out motion-reduce:transition-none"
				enter-from-class="opacity-0"
				enter-to-class="opacity-100 scale-100"
			>
				<ScreenshotCard
					v-for="screenshot in screenshots"
					:key="getSelectionKey(screenshot)"
					:screenshot="screenshot"
					:selection-key="getSelectionKey(screenshot)"
					:selected="selectedKeys.has(getSelectionKey(screenshot))"
					:selection-active="selectionActive"
					:active-dragged="activeDraggedKeys.has(getSelectionKey(screenshot))"
					:can-drag="canDrag"
					:show-instance-name="showInstanceName"
					@activate="(event) => emit('activate', screenshot, event)"
					@toggle-selection="emit('toggle-selection', screenshot)"
					@copy="emit('copy', screenshot)"
					@open="emit('open', screenshot)"
					@delete="emit('delete', screenshot)"
				/>
			</TransitionGroup>
		</ScreenshotSection>
	</div>
</template>
