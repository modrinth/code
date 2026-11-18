<script setup lang="ts">
import { CheckIcon, ClipboardCopyIcon, LoaderCircleIcon } from '@modrinth/assets'
import { IconButton, injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { ref } from 'vue'

import TraceName from '../trace-name.vue'
import type { FlagItem, TraceVerdictEvent } from '../types'
import { injectTechReviewDecisions } from '../use-tech-review-decisions'
import VerdictButtons from '../verdict-buttons/index.vue'
import TraceData from './trace-data.vue'

const props = defineProps<{
	flag: FlagItem
	focused?: boolean
	divider?: boolean
	canHighlight?: (value: string) => boolean
}>()
const emit = defineEmits<{
	verdict: [event: TraceVerdictEvent]
	selectSourceValue: [value: string]
}>()

const row = ref<HTMLElement>()
const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { isPreReviewed } = injectTechReviewDecisions()
const copyingCel = ref(false)
const copiedCel = ref(false)

function emitVerdict(event: TraceVerdictEvent) {
	emit('verdict', event)
}

async function copyDetailCelInput() {
	const detailId = props.flag.detail.id
	if (copyingCel.value) return

	copyingCel.value = true
	try {
		const input = await client.labrinth.tech_review_internal.getDetailRuleInput(detailId)
		const clipboard = row.value?.ownerDocument.defaultView?.navigator.clipboard
		if (!clipboard) throw new Error('Clipboard unavailable')
		await clipboard.writeText(JSON.stringify(input, null, 2))
		copiedCel.value = true
		setTimeout(() => {
			copiedCel.value = false
		}, 2000)
	} catch (error) {
		console.error('Failed to copy CEL input:', error)
		addNotification({
			type: 'error',
			title: 'Failed to copy CEL input',
			text: 'An error occurred while loading the trace rule input.',
		})
	} finally {
		copyingCel.value = false
	}
}
</script>

<template>
	<div
		:id="`tech-review-detail-${flag.detail.id}`"
		ref="row"
		class="flex shrink-0 flex-col gap-0.5 border-0 border-solid border-surface-3"
		:class="{
			'border-b pb-4': divider,
			'focused-trace -mx-2 p-2': focused,
			'py-1.5 first:pt-0': !focused,
		}"
	>
		<div class="flex flex-wrap items-center justify-between gap-2">
			<div
				class="flex flex-wrap items-center gap-1.5"
				:class="{
					'opacity-50': isPreReviewed(flag.detail.id, flag.detail.status),
				}"
			>
				<TraceName :issue-type="flag.issueType" :severity="flag.detail.severity" />
			</div>

			<div class="flex flex-wrap items-center gap-1">
				<IconButton
					v-tooltip="copiedCel ? 'Copied!' : 'Copy CEL to clipboard'"
					type="quiet"
					size="sm"
					:label="copiedCel ? 'Copied!' : 'Copy CEL to clipboard'"
					:disabled="copyingCel"
					@click="copyDetailCelInput"
				>
					<LoaderCircleIcon
						v-if="copyingCel"
						class="!size-4 !min-h-4 !min-w-4 animate-spin"
						aria-hidden="true"
					/>
					<CheckIcon v-else-if="copiedCel" class="!size-4 !min-h-4 !min-w-4" aria-hidden="true" />
					<ClipboardCopyIcon v-else class="!size-4 !min-h-4 !min-w-4" aria-hidden="true" />
				</IconButton>
				<VerdictButtons
					variant="trace"
					:detail="flag.detail"
					@global-safe="
						emitVerdict({
							detail: flag.detail,
							decision: 'safe',
							scope: 'global',
						})
					"
					@local-safe="
						emitVerdict({
							detail: flag.detail,
							decision: 'safe',
							scope: 'local',
						})
					"
					@local-unsafe="
						emitVerdict({
							detail: flag.detail,
							decision: 'malware',
							scope: 'local',
						})
					"
					@global-unsafe="
						emitVerdict({
							detail: flag.detail,
							decision: 'malware',
							scope: 'global',
						})
					"
				/>
			</div>
		</div>
		<TraceData
			v-if="flag.detail.data && Object.keys(flag.detail.data).length > 0"
			:value="flag.detail.data"
			:can-highlight="canHighlight"
			@select="emit('selectSourceValue', $event)"
		/>
	</div>
</template>

<style scoped>
.focused-trace {
	background-color: color-mix(in srgb, var(--color-blue) 15%, transparent);
}
</style>
