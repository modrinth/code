<script setup lang="ts">
import {
	CircleIcon,
	CropIcon,
	EraserIcon,
	EyeOffIcon,
	HighlighterIcon,
	MousePointer2Icon,
	MoveUpRightIcon,
	PencilIcon,
	SquareIcon,
	TypeIcon,
} from '@modrinth/assets'

import IconButton from '#ui/components/base/buttons/IconButton.vue'
import { useVIntl } from '#ui/composables/i18n'

import { imageViewerEditorMessages as messages } from './image-viewer-editor-messages'
import type { ScreenshotEditorTool } from './image-viewer-editor-types'

const props = defineProps<{
	tool: ScreenshotEditorTool
}>()

const emit = defineEmits<{
	select: [tool: ScreenshotEditorTool]
}>()

const { formatMessage } = useVIntl()

const toolGroups = [
	[
		{ id: 'select', message: messages.select, icon: MousePointer2Icon, shortcut: 'V' },
		{ id: 'crop', message: messages.crop, icon: CropIcon, shortcut: 'K' },
	],
	[
		{ id: 'pen', message: messages.pen, icon: PencilIcon, shortcut: 'P' },
		{ id: 'highlight', message: messages.highlight, icon: HighlighterIcon, shortcut: 'H' },
		{ id: 'eraser', message: messages.eraser, icon: EraserIcon, shortcut: 'E' },
	],
	[
		{ id: 'arrow', message: messages.arrow, icon: MoveUpRightIcon, shortcut: 'A' },
		{ id: 'rectangle', message: messages.rectangle, icon: SquareIcon, shortcut: 'R' },
		{ id: 'ellipse', message: messages.ellipse, icon: CircleIcon, shortcut: 'O' },
	],
	[
		{ id: 'text', message: messages.text, icon: TypeIcon, shortcut: 'T' },
		{ id: 'censor', message: messages.censor, icon: EyeOffIcon, shortcut: 'C' },
	],
] satisfies Array<
	Array<{
		id: ScreenshotEditorTool
		message: (typeof messages)[keyof typeof messages]
		icon: typeof MousePointer2Icon
		shortcut: string
	}>
>

function tooltip(groupIndex: number, toolIndex: number) {
	const option = toolGroups[groupIndex]?.[toolIndex]
	return option ? `${formatMessage(option.message)} (${option.shortcut})` : ''
}
</script>

<template>
	<div
		class="absolute left-6 top-1/2 z-10 flex -translate-y-1/2 flex-col items-center gap-2 rounded-[20px] border border-solid border-white/10 bg-surface-3 px-3 py-2.5 shadow-[0_1rem_3rem_rgb(0_0_0_/_32%)] max-[900px]:bottom-[5.25rem] max-[900px]:left-1/2 max-[900px]:top-auto max-[900px]:-translate-x-1/2 max-[900px]:translate-y-0 max-[900px]:flex-row"
		role="toolbar"
		:aria-label="formatMessage(messages.annotationTools)"
		@click.stop
	>
		<template v-for="(group, groupIndex) in toolGroups" :key="group[0]?.id">
			<div v-if="groupIndex > 0" class="h-px w-6 bg-white/10 max-[900px]:h-6 max-[900px]:w-px" />
			<IconButton
				v-for="(option, toolIndex) in group"
				:key="option.id"
				v-tooltip="tooltip(groupIndex, toolIndex)"
				:label="formatMessage(option.message)"
				type="quiet"
				:color="props.tool === option.id ? 'green' : undefined"
				:class="{
					'!bg-highlight-green shadow-[inset_0_0_0_1px_var(--color-green)]':
						props.tool === option.id,
				}"
				:aria-pressed="props.tool === option.id"
				@click="emit('select', option.id)"
			>
				<component :is="option.icon" />
			</IconButton>
		</template>
	</div>
</template>
