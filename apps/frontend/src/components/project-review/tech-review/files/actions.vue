<script setup lang="ts">
import {
	CoffeeIcon,
	DownloadIcon,
	ExternalIcon,
	MoreVerticalIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { type ButtonMenuOption, TeleportOverflowMenu } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectReviewSlicer } from '~/providers/project-review/review-slicer'

import type { FlattenedFileReport } from '../types'

const props = defineProps<{
	file: FlattenedFileReport
	entryPath?: string
}>()

const slicer = injectReviewSlicer(null)
const slicerRequest = ref<number>()
const menuOpen = ref(false)
const openingSlicer = computed(
	() => !!slicer?.loadingFiles.value && slicerRequest.value === slicer.openRequests.value,
)
const slicerUrl = computed(() => {
	const url = `https://slicer.run/?url=${encodeURIComponent(props.file.download_url)}`
	return props.entryPath ? `${url}&file=${encodeURIComponent(props.entryPath)}` : url
})

function openExternal(event: MouseEvent) {
	const element = event.currentTarget as HTMLElement
	element.ownerDocument.defaultView?.open(slicerUrl.value, '_blank', 'noopener,noreferrer')
}

function openSlicer(event: MouseEvent) {
	if (slicer) {
		slicer.open([props.file.download_url], props.entryPath)
		slicerRequest.value = slicer.openRequests.value
	} else {
		openExternal(event)
	}
}

const options = computed<ButtonMenuOption[]>(() => [
	{
		id: 'open-slicer',
		label: 'Open in Slicer',
		icon: CoffeeIcon,
		disabled: openingSlicer.value,
		action: openSlicer,
		trailingAction: {
			label: 'Open in Slicer in a new tab',
			icon: ExternalIcon,
			action: openExternal,
		},
	},
	{
		id: 'download',
		type: 'link',
		label: 'Download',
		icon: DownloadIcon,
		href: props.file.download_url,
		download: props.file.file_name,
	},
])
</script>

<template>
	<TeleportOverflowMenu
		type="quiet"
		size="sm"
		label="File options"
		placement="right-start"
		:aria-label="entryPath ? 'Class file options' : 'Archive options'"
		circular
		class="shrink-0 transition-opacity"
		:class="
			menuOpen
				? 'opacity-100'
				: 'opacity-0 focus:opacity-100 group-focus-within/file-actions:opacity-100 group-hover/file-actions:opacity-100 [@media(hover:none)]:opacity-100'
		"
		@open="menuOpen = true"
		@close="menuOpen = false"
		:options="options"
		@click.stop
	>
		<template #open-slicer="{ option }">
			<SpinnerIcon
				v-if="openingSlicer"
				class="animate-spin motion-reduce:animate-none"
				aria-hidden="true"
			/>
			<CoffeeIcon v-else aria-hidden="true" />
			{{ option.label }}
		</template>
		<MoreVerticalIcon />
	</TeleportOverflowMenu>
</template>
