<template>
	<div
		class="mod-review-shell sticky top-0 flex overflow-hidden border-0 border-t border-solid border-divider"
		:class="{ 'select-none': resizing }"
	>
		<ModerationReviewSidebar
			:project="reviewData.project.value"
			:project-v3="reviewData.projectV3.value"
			:organization="reviewData.organization.value"
			:members="reviewData.members.value"
			:creators-loading="reviewData.creatorsLoading.value"
			:is-server-project="reviewData.isServerProject.value"
			:server-data-loaded="reviewData.serverDataLoaded.value"
			:server-required-content="reviewData.serverRequiredContent.value"
			:server-recommended-version="reviewData.serverRecommendedVersion.value"
			:server-supported-versions="reviewData.serverSupportedVersions.value"
			:server-modpack-loaders="reviewData.serverModpackLoaders.value"
			:width="layout.sidebarWidth.value"
		/>
		<div
			v-if="!layout.sidebarCollapsed.value"
			class="w-1 shrink-0 cursor-col-resize bg-divider transition-colors hover:bg-brand"
			:class="{ 'bg-brand': resizing === 'sidebar' }"
			@pointerdown="startResize('sidebar', $event)"
		/>

		<div class="flex min-h-0 min-w-0 flex-1 flex-col">
			<ModerationReviewPanels dock="main" class="min-h-0 flex-1" />
			<ModerationChecklistWalkthrough />
		</div>

		<div
			class="w-1 shrink-0 cursor-col-resize bg-divider transition-colors hover:bg-brand"
			:class="{ 'bg-brand': resizing === 'panel' }"
			@pointerdown="startResize('panel', $event)"
		/>
		<ModerationChecklistPanel :style="{ width: `${layout.checklistPanelWidth.value}px` }" />

		<ModerationReviewPipHost v-if="pipOpen" />
	</div>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { computed, ref } from 'vue'

import { injectReviewLayoutData } from '~/components/ui/moderation/checklist/checklist-context'
import { useModerationReviewLayout } from '~/services/moderation/review-layout'

import ModerationChecklistPanel from './ModerationChecklistPanel.vue'
import ModerationChecklistWalkthrough from './ModerationChecklistWalkthrough.vue'
import ModerationReviewPanels from './ModerationReviewPanels.vue'
import ModerationReviewPipHost from './ModerationReviewPipHost.vue'
import ModerationReviewSidebar from './ModerationReviewSidebar.vue'

const layout = useModerationReviewLayout()
const reviewData = injectReviewLayoutData()
const settings = useModerationSettings()

const pipOpen = computed(() => layout.pipOpen.value)

const resizing = ref<'sidebar' | 'panel' | null>(null)

function startResize(which: 'sidebar' | 'panel', event: PointerEvent) {
	event.preventDefault()
	const win = (event.view as Window | null) ?? window
	const startX = event.clientX
	const startWidth =
		which === 'sidebar' ? layout.sidebarWidth.value : layout.checklistPanelWidth.value
	resizing.value = which

	function onMove(e: PointerEvent) {
		const delta = e.clientX - startX
		if (which === 'sidebar') layout.setSidebarWidth(startWidth + delta)
		else layout.setChecklistPanelWidth(startWidth - delta)
	}
	function onUp() {
		win.removeEventListener('pointermove', onMove)
		win.removeEventListener('pointerup', onUp)
		resizing.value = null
	}
	win.addEventListener('pointermove', onMove)
	win.addEventListener('pointerup', onUp)
}
</script>

<style scoped>
.mod-review-shell {
	height: calc(100dvh - 4.5rem);
}
</style>
