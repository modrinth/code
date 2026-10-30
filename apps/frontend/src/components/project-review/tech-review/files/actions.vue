<script setup lang="ts">
import {
	CoffeeIcon,
	DownloadIcon,
	ExternalIcon,
	MoreVerticalIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { ButtonLink, type ButtonMenuOption, TeleportOverflowMenu } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { injectReviewSlicer } from '~/providers/project-review/review-slicer'

import type { FlattenedFileReport } from '../types'

const props = defineProps<{
	file: FlattenedFileReport
	entryPath?: string
	inlineActions?: boolean
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

function onSlicerClick(event: MouseEvent) {
	if (
		event.button !== 0 ||
		event.ctrlKey ||
		event.metaKey ||
		event.shiftKey ||
		event.altKey ||
		!slicer
	)
		return
	event.preventDefault()
	if (!openingSlicer.value) openSlicer(event)
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
	<div v-if="inlineActions" class="flex shrink-0 items-center gap-1">
		<ButtonLink
			v-tooltip="'Open in Slicer (Ctrl/Cmd-click or middle-click to open externally)'"
			:href="slicerUrl"
			target="_blank"
			type="quiet"
			size="sm"
			icon-only
			aria-label="Open in Slicer"
			:aria-busy="openingSlicer"
			@click.stop="onSlicerClick"
		>
			<SpinnerIcon
				v-if="openingSlicer"
				class="animate-spin motion-reduce:animate-none"
				aria-hidden="true"
			/>
			<CoffeeIcon v-else aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="'Download archive'"
			:href="file.download_url"
			:download="file.file_name"
			type="quiet"
			size="sm"
			icon-only
			aria-label="Download archive"
		>
			<DownloadIcon aria-hidden="true" />
		</ButtonLink>
	</div>
	<TeleportOverflowMenu
		v-else
		type="quiet"
		size="sm"
		label="File options"
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
