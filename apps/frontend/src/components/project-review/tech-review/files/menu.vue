<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { Input, Tabs } from '@modrinth/ui'
import { useVirtualList } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { getFileDetailCount } from '../helpers'
import type { FlattenedFileReport } from '../types'
import { injectTechReviewDecisions } from '../verdict/use-decisions'
import FileItem from './item.vue'

const props = defineProps<{
	files: FlattenedFileReport[]
	selectedId: string
}>()
const emit = defineEmits<{ select: [file: FlattenedFileReport] }>()
const { getFileMarkedCount } = injectTechReviewDecisions()
const searches = ref({ pending: '', completed: '' })
const tab = ref<string | number>('pending')
const pending = computed(() =>
	props.files.filter((file) => getFileMarkedCount(file) < getFileDetailCount(file)),
)
const completed = computed(() =>
	props.files.filter((file) => getFileMarkedCount(file) >= getFileDetailCount(file)),
)
const tabs = computed(() => [
	{ value: 'pending', label: `Pending (${pending.value.length})` },
	{ value: 'completed', label: `Completed (${completed.value.length})` },
])
const activeFiles = computed(() => (tab.value === 'pending' ? pending.value : completed.value))
const search = computed({
	get: () => searches.value[tab.value === 'pending' ? 'pending' : 'completed'],
	set: (value: string) => {
		searches.value[tab.value === 'pending' ? 'pending' : 'completed'] = value
	},
})
const files = computed(() => {
	const term = activeFiles.value.length > 6 ? search.value.trim().toLowerCase() : ''
	return activeFiles.value.filter((file) =>
		`${file.file_name} ${file.version_number ?? ''}`.toLowerCase().includes(term),
	)
})
const { list, containerProps, wrapperProps, scrollTo } = useVirtualList(files, {
	itemHeight: 76,
	overscan: 3,
})
watch([search, tab], () => scrollTo(0))
watch(
	files,
	() => {
		const container = containerProps.ref.value
		if (!container) return
		const lastStart = Math.max(0, files.value.length - Math.ceil(container.clientHeight / 76))
		scrollTo(Math.min(Math.floor(container.scrollTop / 76), lastStart))
	},
	{ flush: 'post' },
)
</script>

<template>
	<section
		aria-label="Archives"
		class="review-scrollbars flex w-[30rem] max-w-[calc(100vw-2rem)] flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-1.5 shadow-2xl"
	>
		<Tabs v-model:value="tab" :tabs="tabs" />
		<Input
			v-if="activeFiles.length > 6"
			v-model="search"
			:icon="SearchIcon"
			placeholder="Search archives..."
			aria-label="Search archives"
		/>
		<div
			v-bind="containerProps"
			class="max-h-[60vh] overscroll-contain"
			:style="{ height: `${Math.min(files.length, 6) * 76}px` }"
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
		<p v-if="!files.length" class="m-0 p-3 text-sm text-secondary">No matching archives</p>
	</section>
</template>
