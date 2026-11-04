<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { Input } from '@modrinth/ui'
import { useVirtualList } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import type { FlattenedFileReport } from '../types'
import FileItem from './item.vue'

const props = defineProps<{
	files: FlattenedFileReport[]
	selectedId: string
	label?: string
}>()
const emit = defineEmits<{ select: [file: FlattenedFileReport] }>()
const search = ref('')
const filteredFiles = computed(() => {
	const term = props.files.length > 6 ? search.value.trim().toLowerCase() : ''
	return props.files.filter((file) =>
		`${file.file_name} ${file.version_number ?? ''}`.toLowerCase().includes(term),
	)
})
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(filteredFiles, {
	itemHeight: 76,
	overscan: 3,
})
watch(search, () => scrollTo(0))
watch(
	filteredFiles,
	() => {
		const container = containerProps.ref.value
		if (!container) return
		const lastStart = Math.max(
			0,
			filteredFiles.value.length - Math.ceil(container.clientHeight / 76),
		)
		scrollTo(Math.min(Math.floor(container.scrollTop / 76), lastStart))
	},
	{ flush: 'post' },
)
</script>

<template>
	<section
		:aria-label="label ?? 'All archives'"
		class="review-scrollbars flex w-[34rem] max-w-[calc(100vw-2rem)] flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
	>
		<Input
			v-if="files.length > 6"
			v-model="search"
			:icon="SearchIcon"
			placeholder="Search archives..."
			aria-label="Search archives"
		/>
		<div
			v-bind="containerProps"
			class="max-h-[60vh] overscroll-contain"
			:style="{ height: `${Math.min(filteredFiles.length, 6) * 76}px` }"
		>
			<div v-bind="wrapperProps">
				<div v-for="item in list" :key="item.data.id" class="h-[76px] pb-1">
					<FileItem
						:file="item.data"
						:selected="item.data.id === selectedId"
						@view-flags="emit('select', $event)"
					/>
				</div>
			</div>
		</div>
		<p v-if="!filteredFiles.length" class="m-0 p-3 text-sm text-secondary">No matching archives</p>
	</section>
</template>
