<script setup lang="ts">
import { defineMessages, TagItem, useFormatBytes, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import { getFileDetailCount, getHighestSeverity } from '../helpers'
import { severityPresentation } from '../traces/presentation'
import type { FlattenedFileReport } from '../types'
import { injectTechReviewDecisions } from '../verdict/use-decisions'

const props = defineProps<{
	file: FlattenedFileReport
	selected?: boolean
}>()

const emit = defineEmits<{
	viewFlags: [file: FlattenedFileReport]
}>()

const formatBytes = useFormatBytes()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	manual: {
		id: 'moderation.tech-review.file.manual',
		defaultMessage: 'Manual review',
	},
	noFlags: {
		id: 'moderation.tech-review.file.no-flags',
		defaultMessage: 'No flags',
	},
})
const { getFileMarkedCount } = injectTechReviewDecisions()
const details = computed(() => props.file.issues.flatMap((issue) => issue.details))
const hasFlags = computed(() => details.value.length > 0)
const canSelect = computed(() => !props.selected)
const isManualReview = computed(() => props.file.flag_reason.toString() === 'manual')
const fileSizeLabel = computed(() => formatBytes(props.file.file_size))

const markedCount = computed(() => getFileMarkedCount(props.file))
const traceCount = computed(() => getFileDetailCount(props.file))
const severity = computed(() => getHighestSeverity(details.value))
const severityLabel = computed(
	() => severity.value.charAt(0).toUpperCase() + severity.value.slice(1),
)
const severityStyle = computed(() => ({
	'--_bg-color': severityPresentation[severity.value].bgColor,
	'--_color': severityPresentation[severity.value].color,
}))

function viewFlags() {
	if (canSelect.value) {
		emit('viewFlags', props.file)
	}
}
</script>

<template>
	<button
		type="button"
		:disabled="!canSelect"
		:aria-current="selected ? 'true' : undefined"
		class="grid h-[72px] w-full min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-4 rounded-lg border-0 px-3 py-2.5 text-left transition-[background-color,filter] focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand disabled:cursor-default"
		:class="[
			selected ? 'bg-surface-3' : 'bg-transparent',
			canSelect ? 'cursor-pointer hover:bg-surface-3 hover:brightness-125' : 'cursor-default',
		]"
		@click="viewFlags"
	>
		<span class="flex min-w-0 flex-col gap-1.5">
			<span class="truncate text-sm font-medium text-contrast" :title="file.file_name">{{
				file.file_name
			}}</span>
			<span class="flex min-w-0 items-center gap-2 text-xs text-secondary">
				<span v-if="file.version_number" class="truncate">{{ file.version_number }}</span>
				<span v-if="file.version_number" aria-hidden="true">·</span>
				<span class="shrink-0">{{ fileSizeLabel }}</span>
			</span>
		</span>
		<span class="flex shrink-0 flex-col items-end gap-1.5">
			<TagItem v-if="hasFlags" class="!px-1.5 !py-0.5 !text-xs" :style="severityStyle">
				<component
					:is="severityPresentation[severity].icon"
					:style="{ color: severityPresentation[severity].color }"
					aria-hidden="true"
				/>
				{{ severityLabel }}
			</TagItem>
			<span v-else class="text-xs" :class="isManualReview ? 'text-blue' : 'text-secondary'">{{
				formatMessage(isManualReview ? messages.manual : messages.noFlags)
			}}</span>
			<span v-if="hasFlags" class="text-xs tabular-nums text-secondary"
				>{{ markedCount }}/{{ traceCount }} reviewed</span
			>
		</span>
	</button>
</template>
