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
					@delete="$emit('delete', item)"
					@rename="$emit('rename', item)"
					@extract="$emit('extract', item)"
					@download="$emit('download', item)"
					@move="$emit('move', item)"
					@move-direct-to="$emit('moveDirectTo', $event)"
					@edit="$emit('edit', item)"
					@hover="$emit('hover', item)"
					@contextmenu="(x, y) => $emit('contextmenu', item, x, y)"
					@toggle-select="$emit('toggle-select', item.path)"
				/>
			</ul>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Kyros } from '@modrinth/api-client'
import { toRef } from 'vue'

import { useVirtualScroll } from '../../../../composables/virtual-scroll'
import FileItem from './FileItem.vue'

const props = defineProps<{
	items: Kyros.Files.v0.DirectoryItem[]
	selectedItems: Set<string>
}>()

const emit = defineEmits<{
	delete: [item: Kyros.Files.v0.DirectoryItem]
	rename: [item: Kyros.Files.v0.DirectoryItem]
	download: [item: Kyros.Files.v0.DirectoryItem]
	move: [item: Kyros.Files.v0.DirectoryItem]
	edit: [item: Kyros.Files.v0.DirectoryItem]
	moveDirectTo: [item: { name: string; type: string; path: string; destination: string }]
	extract: [item: Kyros.Files.v0.DirectoryItem]
	hover: [item: Kyros.Files.v0.DirectoryItem]
	contextmenu: [item: Kyros.Files.v0.DirectoryItem, x: number, y: number]
	loadMore: []
	'toggle-select': [path: string]
}>()

const { listContainer, totalHeight, visibleRange, visibleTop, visibleItems } = useVirtualScroll(
	toRef(props, 'items'),
	{
		itemHeight: 61,
		bufferSize: 5,
		onNearEnd: () => emit('loadMore'),
	},
)
</script>
