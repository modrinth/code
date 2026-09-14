<script setup lang="ts">
import { FloatingMenu } from '@modrinth/ui'
import { computed, nextTick, onScopeDispose, ref, useId, useTemplateRef, watch } from 'vue'

import DownloadManagerBar from './download-manager-bar.vue'
import DownloadManagerPanel from './download-manager-panel.vue'
import { useDownloadBarState } from './use-download-bar-state'
import { useDownloadManager } from './use-download-manager'

const {
	activeJobs,
	attentionJobs,
	completedJobs,
	initialized,
	rate,
	retry,
	cancel,
	togglePause,
	dismiss,
	clearCompleted,
	copyDetails,
} = useDownloadManager()
const { selectedJob, completing } = useDownloadBarState({
	activeJobs,
	attentionJobs,
	completedJobs,
	initialized,
})
const shown = ref(false)
const animationsEnabled = ref(false)
const menu = useTemplateRef('menu')
const bar = useTemplateRef('bar')
const panel = useTemplateRef('panel')
const panelId = useId()
const progress = computed(() =>
	activeJobs.value.length
		? activeJobs.value.reduce((total, job) => total + job.overallProgress, 0) /
			activeJobs.value.length
		: 0,
)
const visible = computed(
	() =>
		shown.value ||
		!!selectedJob.value ||
		activeJobs.value.length ||
		attentionJobs.value.length ||
		completedJobs.value.length,
)
let animationFrame = 0

watch(
	initialized,
	(ready) => {
		if (!ready) return
		animationFrame = requestAnimationFrame(() => {
			animationFrame = requestAnimationFrame(() => {
				animationsEnabled.value = true
			})
		})
	},
	{ flush: 'post' },
)
onScopeDispose(() => cancelAnimationFrame(animationFrame))

function onOpen() {
	shown.value = true
	nextTick(() => panel.value?.focus())
}

function close() {
	menu.value?.hide()
	bar.value?.focus()
}
</script>

<template>
	<FloatingMenu
		v-if="visible"
		ref="menu"
		bare
		:arrow="false"
		placement="bottom-end"
		panel-class="download-manager-popper"
		@open="onOpen"
		@close="shown = false"
	>
		<DownloadManagerBar
			ref="bar"
			:selected-job="selectedJob"
			:active-count="activeJobs.length"
			:progress="progress"
			:has-attention="attentionJobs.length > 0"
			:rate="rate"
			:completing="completing"
			:animated="animationsEnabled"
			:expanded="shown"
			:panel-id="panelId"
			@close="close"
		/>
		<template #popper="{ hide }">
			<DownloadManagerPanel
				:id="panelId"
				ref="panel"
				:active-jobs="activeJobs"
				:attention-jobs="attentionJobs"
				:completed-jobs="completedJobs"
				:rate="rate"
				@retry="retry"
				@cancel="cancel"
				@toggle-pause="togglePause"
				@dismiss="dismiss"
				@clear-completed="clearCompleted"
				@copy-details="copyDetails"
				@open="hide"
				@keydown.esc.stop.prevent="close"
			/>
		</template>
	</FloatingMenu>
</template>

<style>
.download-manager-popper {
	border-radius: 1rem;
	box-shadow:
		0 2px 4px rgba(0, 0, 0, 0.04),
		0 5px 8px rgba(0, 0, 0, 0.04),
		0 10px 18px rgba(0, 0, 0, 0.03),
		0 24px 48px rgba(0, 0, 0, 0.03);
}
</style>
