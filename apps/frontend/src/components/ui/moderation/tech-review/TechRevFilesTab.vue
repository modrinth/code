<script setup lang="ts">
import { CheckIcon, DownloadIcon, ExternalIcon, VersionIcon } from '@modrinth/assets'
import { ButtonLink, useFormatBytes } from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { computed } from 'vue'

import {
	getFileDetailCount,
	getFileHighestSeverity,
	getSeverityBadgeColor,
	getVersionLabel,
	getVersionPageHref,
	truncateMiddle,
} from './helpers'
import type { FlattenedFileReport } from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	reports: FlattenedFileReport[]
	project: {
		id: string
		slug?: string
		project_types: string[]
	}
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const formatBytes = useFormatBytes()
const { getFileMarkedCount } = injectTechReviewDecisions()

const allFiles = computed(() => {
	return [...props.reports].sort((a, b) => {
		const aComplete = getFileMarkedCount(a) === getFileDetailCount(a)
		const bComplete = getFileMarkedCount(b) === getFileDetailCount(b)
		return aComplete === bComplete ? 0 : aComplete ? 1 : -1
	})
})
</script>

<template>
	<div
		v-for="(file, idx) in allFiles"
		:key="idx"
		class="flex items-center justify-between border-0 border-x border-b border-solid border-surface-3 bg-surface-2 px-4 py-3"
		:class="{
			'rounded-bl-2xl rounded-br-2xl': idx === allFiles.length - 1,
			'bg-[#E8E8E8] dark:bg-[#1A1C20]': idx % 2 === 1,
		}"
	>
		<div class="flex items-center gap-3">
			<span
				v-tooltip="file.file_name"
				class="py-2 font-medium text-contrast"
				:aria-label="`View flags for ${file.file_name}`"
				tabindex="0"
				:class="{ 'cursor-pointer hover:underline': getFileDetailCount(file) > 0 }"
				@click="getFileDetailCount(file) > 0 && emit('viewFlags', file)"
			>
				{{ truncateMiddle(file.file_name, 50) }}
			</span>
			<div class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1">
				<span class="text-sm font-medium text-secondary">{{ formatBytes(file.file_size) }}</span>
			</div>
			<div
				v-if="getFileDetailCount(file) > 0"
				class="rounded-full border-solid px-2.5 py-1"
				:class="getSeverityBadgeColor(getFileHighestSeverity(file))"
			>
				<span class="text-sm font-medium">{{
					capitalizeString(getFileHighestSeverity(file))
				}}</span>
			</div>
			<div
				v-if="getFileDetailCount(file) > 0"
				class="flex items-center gap-1 rounded-full border border-solid px-2.5 py-1 text-sm"
				:class="
					getFileMarkedCount(file) === getFileDetailCount(file)
						? 'border-green/60 bg-highlight-green text-green'
						: 'border-red/60 bg-highlight-red text-red'
				"
			>
				<CheckIcon v-if="getFileMarkedCount(file) === getFileDetailCount(file)" class="size-4" />
				{{ getFileMarkedCount(file) }}/{{ getFileDetailCount(file) }} flags
			</div>
			<!-- TODO: remove toString when backend supports it properly -->
			<div
				v-else-if="file.flag_reason.toString() === 'manual'"
				class="border-blue/60 flex items-center gap-1 rounded-full border border-solid bg-highlight-blue px-2.5 py-1 text-sm text-blue"
			>
				Manual review
			</div>
			<div
				v-else
				class="border-green/60 flex items-center gap-1 rounded-full border border-solid bg-highlight-green px-2.5 py-1 text-sm text-green"
			>
				No flags
			</div>
		</div>

		<div class="flex items-center gap-2">
			<ButtonLink
				type="outlined"
				target="_blank"
				:href="getVersionPageHref(project, file.version_id)"
				:aria-label="`Open version ${getVersionLabel(file)}`"
			>
				<VersionIcon aria-hidden="true" /> {{ getVersionLabel(file) }}
			</ButtonLink>
			<ButtonLink
				type="outlined"
				target="_blank"
				:href="`https://slicer.run/?url=${encodeURIComponent(file.download_url)}`"
				aria-label="Open in Slicer"
			>
				<ExternalIcon aria-hidden="true" /> Slicer
			</ButtonLink>
			<ButtonLink
				v-tooltip="`Download ${file.file_name} (${formatBytes(file.file_size)})`"
				type="outlined"
				:href="file.download_url"
				:download="file.file_name"
				tabindex="0"
				icon-only
				circular
			>
				<DownloadIcon />
			</ButtonLink>
		</div>
	</div>
</template>
