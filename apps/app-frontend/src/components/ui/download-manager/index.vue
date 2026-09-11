<script setup lang="ts">
import { Dropdown } from 'floating-vue'
import { computed, onScopeDispose, ref, useId, useTemplateRef, watch } from 'vue'

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
	copyDetails,
} = useDownloadManager()
const { task, completing } = useDownloadBarState({
	activeJobs,
	attentionJobs,
	completedJobs,
	initialized,
})
const shown = ref(false)
const animationsEnabled = ref(false)
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
		!!task.value ||
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

function close() {
	shown.value = false
	bar.value?.focus()
}
</script>

<template>
	<Dropdown
		v-if="visible"
		v-model:shown="shown"
		:triggers="[]"
		:distance="4"
		:skidding="0"
		:delay="0"
		:dispose-timeout="0"
		placement="bottom-end"
		popper-class="download-manager-popper"
		no-auto-focus
		@apply-show="panel?.focus()"
	>
		<DownloadManagerBar
			ref="bar"
			:task="task"
			:active-count="activeJobs.length"
			:progress="progress"
			:has-attention="attentionJobs.length > 0"
			:rate="rate"
			:completing="completing"
			:animated="animationsEnabled"
			:expanded="shown"
			:panel-id="panelId"
			@toggle="shown = !shown"
			@close="close"
		/>
		<template #popper>
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
				@copy-details="copyDetails"
				@open="shown = false"
				@keydown.esc.stop.prevent="close"
			/>
		</template>
	</Dropdown>
</template>

<style>
.v-popper__popper.v-popper--theme-dropdown.download-manager-popper .v-popper__inner {
	padding: 0 !important;
	border: 0 !important;
	border-radius: 1rem !important;
	box-shadow:
		0 2px 4px rgba(0, 0, 0, 0.04),
		0 5px 8px rgba(0, 0, 0, 0.04),
		0 10px 18px rgba(0, 0, 0, 0.03),
		0 24px 48px rgba(0, 0, 0, 0.03) !important;
}

.download-manager-popper .v-popper__arrow-container {
	display: none;
}
</style>
