<template>
	<FilePageLayout advanced-view :search-loaded-only-notice="!fullyLoaded" />
</template>

<script setup lang="ts">
import { watch } from 'vue'

import type { RootZipTreeNode, ZipHandler } from '#ui/helpers/toolbox/zip'
import FilePageLayout from '#ui/layouts/shared/files-tab/layout.vue'
import { provideFileManager } from '#ui/layouts/shared/files-tab/providers/file-manager'

import { useArchiveFileManager } from '../composables/archive-file-manager'
import type { ArchiveSource } from '../types'

const props = defineProps<{
	root: RootZipTreeNode
	handler: ZipHandler
	source: ArchiveSource
	/** Path within the archive, without a leading slash. */
	path?: string | null
}>()

const emit = defineEmits<{
	navigate: [path: string]
}>()

const { context, location, open, fullyLoaded } = useArchiveFileManager({
	root: props.root,
	handler: props.handler,
	source: props.source,
	initialPath: props.path,
})

provideFileManager(context)

watch(location, (value) => emit('navigate', value.path.replace(/^\/+/, '')))

watch(
	() => props.path,
	(path) => open(path ?? ''),
)
</script>
