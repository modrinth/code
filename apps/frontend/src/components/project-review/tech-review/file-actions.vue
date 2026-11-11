<script setup lang="ts">
import { CoffeeIcon, DownloadIcon, SpinnerIcon } from '@modrinth/assets'
import { ButtonLink } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectReviewSlicer } from '~/providers/project-review/review-slicer'

import type { FlattenedFileReport } from './types'

const props = defineProps<{
	file: FlattenedFileReport
	entryPath?: string
}>()

const slicer = injectReviewSlicer(null)
const slicerRequest = ref<number>()
const openingSlicer = computed(
	() => !!slicer?.loadingFiles.value && slicerRequest.value === slicer.openRequests.value,
)
const slicerUrl = computed(() => {
	const url = `https://slicer.run/?url=${encodeURIComponent(props.file.download_url)}`
	return props.entryPath ? `${url}&file=${encodeURIComponent(props.entryPath)}` : url
})

function openSlicer(event: MouseEvent) {
	if (event.ctrlKey || event.metaKey || event.shiftKey || event.altKey || event.button !== 0) return
	if (!slicer) return
	event.preventDefault()
	if (openingSlicer.value) return
	slicer.open([props.file.download_url], props.entryPath)
	slicerRequest.value = slicer.openRequests.value
}
</script>

<template>
	<div class="flex shrink-0 items-center gap-1">
		<ButtonLink
			v-tooltip="'Open in Slicer'"
			type="quiet"
			size="sm"
			icon-only
			circular
			aria-label="Open in Slicer"
			:href="slicerUrl"
			target="_blank"
			:aria-busy="openingSlicer || undefined"
			class="shrink-0"
			@click.stop="openSlicer"
			@auxclick.stop
		>
			<SpinnerIcon
				v-if="openingSlicer"
				class="animate-spin motion-reduce:animate-none"
				aria-hidden="true"
			/>
			<CoffeeIcon v-else aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-if="!entryPath"
			v-tooltip="'Download archive'"
			type="quiet"
			size="sm"
			icon-only
			circular
			aria-label="Download archive"
			:href="file.download_url"
			:download="file.file_name"
			class="shrink-0"
			@click.stop
			@auxclick.stop
		>
			<DownloadIcon aria-hidden="true" />
		</ButtonLink>
	</div>
</template>
