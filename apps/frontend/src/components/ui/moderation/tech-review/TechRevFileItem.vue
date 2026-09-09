<script setup lang="ts">
import { RightArrowIcon, VersionIcon } from '@modrinth/assets'
import { Button, ButtonLink, useFormatBytes } from '@modrinth/ui'
import { computed } from 'vue'

import { getFileDetailCount, getVersionPageHref, truncateMiddle } from './helpers'
import TechRevFileActions from './TechRevFileActions.vue'
import TechRevFlagBadges from './TechRevFlagBadges.vue'
import type { FlattenedFileReport, TechRevProjectRef } from './types'
import { injectTechReviewDecisions } from './use-tech-review-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	project: TechRevProjectRef
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const formatBytes = useFormatBytes()
const { getFileMarkedCount } = injectTechReviewDecisions()
const details = computed(() => props.file.issues.flatMap((issue) => issue.details))
const hasFlags = computed(() => details.value.length > 0)
const isManualReview = computed(() => props.file.flag_reason.toString() === 'manual')
const allFlagsMarked = computed(
	() => hasFlags.value && getFileMarkedCount(props.file) === getFileDetailCount(props.file),
)

const truncatedFileName = computed(() => truncateMiddle(props.file.file_name, 50))
const versionHref = computed(() => getVersionPageHref(props.project, props.file.version_id))
const fileSizeLabel = computed(() => formatBytes(props.file.file_size))

function viewFlags() {
	if (hasFlags.value) {
		emit('viewFlags', props.file)
	}
}
</script>

<template>
	<div class="flex items-center justify-between px-4 py-3">
		<div class="flex items-center gap-2">
			<ButtonLink
				v-tooltip="'View version'"
				type="outlined"
				target="_blank"
				:href="versionHref"
				class="!bg-surface-2"
				icon-only
				circular
			>
				<VersionIcon aria-hidden="true" />
			</ButtonLink>
			<div
				v-tooltip="file.file_name === truncatedFileName ? undefined : file.file_name"
				class="group flex h-9 cursor-pointer flex-col justify-center rounded-sm font-medium text-contrast hover:underline"
				:class="{ 'opacity-50': allFlagsMarked }"
				@click="viewFlags"
			>
				{{ truncatedFileName }}
			</div>
			<div
				class="rounded-full border border-solid border-surface-5 bg-surface-3 px-2.5 py-1"
				:class="{ 'opacity-50': allFlagsMarked }"
			>
				<span class="text-sm font-medium text-secondary">{{ fileSizeLabel }}</span>
			</div>
			<TechRevFlagBadges v-if="hasFlags" :details="details" />
			<!-- TODO: remove toString when backend supports it properly -->
			<div
				v-else-if="isManualReview"
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
			<Button v-if="hasFlags" @click="viewFlags"> Flags <RightArrowIcon /> </Button>
			<TechRevFileActions :file="file" />
		</div>
	</div>
</template>
