<template>
	<div ref="listContainer" class="relative w-full">
		<div
			:style="{
				position: 'relative',
				minHeight: `${totalHeight}px`,
			}"
		>
			<ul
				class="list-none"
				:style="{
					position: 'absolute',
					top: `${visibleTop}px`,
					width: '100%',
					margin: 0,
					padding: 0,
				}"
			>
				<FileItem
					v-for="(item, idx) in visibleItems"
					:key="item.path"
					:count="item.count"
					:created="item.created"
					:modified="item.modified"
					:name="item.name"
					:path="item.path"
					:type="item.type"
					:size="item.size"
					:index="visibleRange.start + idx"
					:is-last="visibleRange.start + idx === props.items.length - 1"
					:selected="selectedItems.has(item.path)"
					:write-disabled="writeDisabled"
					:write-disabled-tooltip="writeDisabledTooltip"
					@delete="$emit('delete', item)"
					@rename="$emit('rename', item)"
					@extract="$emit('extract', item)"
					@download="$emit('download', item)"
					@move="$emit('move', item)"
					@move-direct-to="$emit('moveDirectTo', $event)"
					@edit="$emit('edit', item)"
					@navigate="$emit('navigate', item)"
					@hover="$emit('hover', item)"
					@contextmenu="(x, y) => $emit('contextmenu', item, x, y)"
					@toggle-select="$emit('toggle-select', item.path)"
				/>
			</ul>
		</div>
	</div>
</template>

<script setup lang="ts">
import { toRef } from 'vue'

import { useVirtualScroll } from '#ui/composables/virtual-scroll'

import type { FileItem as FileItemType } from '../types'
import FileItem from './FileTableRow.vue'

const props = defineProps<{
	items: FileItemType[]
	selectedItems: Set<string>
	writeDisabled?: boolean
	writeDisabledTooltip?: string
}>()

const emit = defineEmits<{
	delete: [item: FileItemType]
	rename: [item: FileItemType]
	download: [item: FileItemType]
	move: [item: FileItemType]
	edit: [item: FileItemType]
	navigate: [item: FileItemType]
	moveDirectTo: [item: { name: string; type: string; path: string; destination: string }]
	extract: [item: FileItemType]
	hover: [item: FileItemType]
	contextmenu: [item: FileItemType, x: number, y: number]
	loadMore: []
	'toggle-select': [path: string]
}>()

const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems } = useVirtualScroll(
	toRef(props, 'items'),
	{
		itemHeight: 48,
		bufferSize: 5,
		onNearEnd: () => emit('loadMore'),
	},
)
</script>
