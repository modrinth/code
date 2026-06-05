<template>
	<div
		class="grid grid-cols-[min-content_auto_min-content_min-content] items-center gap-2 rounded-2xl border-[1px] border-divider bg-bg p-2"
	>
		<VersionChannelIndicator :channel="version.version_type" />
		<div class="flex min-w-0 flex-col gap-1">
			<h1 class="my-0 truncate text-nowrap text-base font-extrabold leading-none text-contrast">
				{{ version.version_number }}
			</h1>
			<p class="m-0 truncate text-nowrap text-xs font-semibold text-secondary">
				{{ version.name }}
			</p>
		</div>
		<ButtonStyled color="brand">
			<a
				:href="downloadUrl"
				:download="primaryFilename"
				class="min-w-0"
				@click="emit('onDownload')"
			>
				<DownloadIcon aria-hidden="true" /> Download
			</a>
		</ButtonStyled>
		<ButtonStyled circular>
			<button
				class="min-w-0"
				aria-label="View version"
				@click="
					emit('onNavigate', `/project/${props.version.project_id}/version/${props.version.id}`)
				"
			>
				<ExternalIcon aria-hidden="true" />
			</button>
		</ButtonStyled>
	</div>
</template>

<script setup lang="ts">
import { DownloadIcon, ExternalIcon } from '@icarus/assets'
import type { Version, VersionFile } from '@icarus/utils'
import { computed } from 'vue'

import { ButtonStyled, VersionChannelIndicator } from '../index'

const props = defineProps<{
	version: Version
}>()

const primaryFile = computed<VersionFile>(
	() => props.version.files.find((x) => x.primary) || props.version.files[0],
)

const downloadUrl = computed(() => {
	return primaryFile.value.url
})

const primaryFilename = computed(() => primaryFile.value.filename)

const emit = defineEmits<{
	onDownload: []
	onNavigate: [url: string]
}>()
</script>
