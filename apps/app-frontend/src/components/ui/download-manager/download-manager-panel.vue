<script setup lang="ts">
import { DropdownIcon } from '@modrinth/assets'
import {
	Collapsible,
	defineMessages,
	injectPageContext,
	useFormatNumber,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { useResizeObserver } from '@vueuse/core'
import { computed, ref, useId, useTemplateRef } from 'vue'

import DownloadManagerJobRow from './download-manager-job.vue'
import type { DownloadManagerJob } from './use-download-manager'

const props = defineProps<{
	activeJobs: DownloadManagerJob[]
	attentionJobs: DownloadManagerJob[]
	completedJobs: DownloadManagerJob[]
	rate: string
}>()
defineEmits<{
	retry: [id: string]
	cancel: [id: string]
	togglePause: [id: string]
	dismiss: [id: string]
	copyDetails: [id: string]
	open: []
}>()

const { formatMessage } = useVIntl()
const { showAds } = injectPageContext()
const formatNumber = useFormatNumber()
const messages = defineMessages({
	tasks: { id: 'app.download-manager.tasks', defaultMessage: 'Tasks' },
	active: { id: 'app.download-manager.active', defaultMessage: 'Active' },
	attention: { id: 'app.download-manager.attention', defaultMessage: 'Needs attention' },
	complete: { id: 'app.download-manager.complete', defaultMessage: 'Complete' },
	empty: { id: 'app.download-manager.empty', defaultMessage: 'No installation tasks' },
	rate: { id: 'app.download-manager.estimated-rate', defaultMessage: 'Estimated download speed' },
})
const panel = useTemplateRef('panel')
const scrollContainer = useTemplateRef<HTMLElement>('scroll-container')
const scrollContent = useTemplateRef<HTMLElement>('scroll-content')
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollContainer)
useResizeObserver(scrollContent, checkScrollState)
const sectionId = useId()
const collapsed = ref(new Set<string>())
const sections = computed(() =>
	[
		{ id: 'active', label: formatMessage(messages.active), jobs: props.activeJobs },
		{ id: 'attention', label: formatMessage(messages.attention), jobs: props.attentionJobs },
		{ id: 'complete', label: formatMessage(messages.complete), jobs: props.completedJobs },
	].filter((section) => section.jobs.length),
)

function toggleSection(id: string) {
	if (collapsed.value.has(id)) collapsed.value.delete(id)
	else collapsed.value.add(id)
}

defineExpose({ focus: () => panel.value?.focus() })
</script>

<template>
	<div
		ref="panel"
		role="dialog"
		:aria-label="formatMessage(messages.tasks)"
		tabindex="-1"
		class="flex w-[360px] max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-2xl bg-surface-3 ring-1 ring-inset ring-surface-5 outline-none"
	>
		<div class="flex h-14 shrink-0 items-center gap-2 p-4 shadow-[inset_0_-1px_0_var(--surface-5)]">
			<span class="text-base font-semibold leading-6 text-contrast">
				{{ formatMessage(messages.tasks) }}
			</span>
			<span
				v-if="activeJobs.length"
				class="flex h-6 min-w-6 items-center justify-center rounded-full border border-solid border-brand px-1 text-sm font-semibold leading-[18px] text-brand tabular-nums"
				style="background-color: var(--color-green-highlight)"
			>
				{{ formatNumber(activeJobs.length) }}
			</span>
			<span
				v-if="rate"
				v-tooltip="formatMessage(messages.rate)"
				class="text-sm font-medium leading-5 text-primary tabular-nums"
			>
				{{ rate }}
			</span>
		</div>
		<div class="relative min-h-0">
			<div
				class="pointer-events-none absolute inset-x-0 top-0 z-10 h-6 bg-gradient-to-b from-surface-3 to-transparent transition-opacity duration-200 motion-reduce:transition-none"
				:class="showTopFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
			<div
				ref="scroll-container"
				class="min-h-0 overflow-y-auto overscroll-contain"
				:style="{
					maxHeight: `min(32rem, max(0px, calc(100dvh - 8rem - ${showAds ? 250 : 0}px)))`,
				}"
			>
				<div ref="scroll-content" class="flex flex-col gap-4 p-2.5">
					<template v-for="(section, index) in sections" :key="section.id">
						<div v-if="index" class="h-px shrink-0 bg-surface-5" />
						<section class="flex flex-col">
							<button
								type="button"
								class="flex w-full items-center justify-between gap-2 rounded-md border-0 bg-transparent px-1.5 text-sm font-medium leading-5 text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand"
								:class="{
									'pt-1.5': index === 0,
									'pb-1.5': index === sections.length - 1,
								}"
								:aria-expanded="!collapsed.has(section.id)"
								:aria-controls="`${sectionId}-${section.id}`"
								@click="toggleSection(section.id)"
							>
								{{ section.label }}
								<DropdownIcon
									class="size-4 shrink-0 transition-transform"
									:class="{ 'rotate-180': !collapsed.has(section.id) }"
									aria-hidden="true"
								/>
							</button>
							<Collapsible
								:id="`${sectionId}-${section.id}`"
								:collapsed="collapsed.has(section.id)"
							>
								<div
									class="flex flex-col"
									:class="{
										'pt-1.5': index !== sections.length - 1,
										'gap-2': section.id === 'complete',
										'gap-1.5': section.id !== 'complete',
									}"
								>
									<DownloadManagerJobRow
										v-for="job in section.jobs"
										:key="job.id"
										:job="job"
										@retry="$emit('retry', $event)"
										@cancel="$emit('cancel', $event)"
										@toggle-pause="$emit('togglePause', $event)"
										@dismiss="$emit('dismiss', $event)"
										@copy-details="$emit('copyDetails', $event)"
										@open="$emit('open')"
									/>
								</div>
							</Collapsible>
						</section>
					</template>
					<p v-if="!sections.length" class="m-0 p-1.5 text-sm text-primary">
						{{ formatMessage(messages.empty) }}
					</p>
				</div>
			</div>
			<div
				class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-6 bg-gradient-to-t from-surface-3 to-transparent transition-opacity duration-200 motion-reduce:transition-none"
				:class="showBottomFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
		</div>
	</div>
</template>
