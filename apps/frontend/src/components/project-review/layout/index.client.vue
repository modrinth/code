<template>
	<section
		class="project-review relative flex flex-col overflow-hidden bg-surface-1"
		aria-label="Project Review"
	>
		<ArchiveSession
			:source="archive.source.value"
			:path="archive.path.value"
			@update:path="archive.setPath"
		>
			<ReviewWorkbench
				v-if="nestedArchive"
				ref="reviewWorkbench"
				nested-archive
				:slots="slots"
				:tabs="tabs"
				:side-panels="sidePanels"
				:counts="counts"
				:reset-key="resetKey"
				:ready="ready"
			/>
			<FilePageLayout
				v-else
				workbench
				workspace="project-review"
				default-browsing="sidebar"
				compact-navigation
				:sidebar="sidebar"
			>
				<template #workspace>
					<ReviewWorkbench
						ref="reviewWorkbench"
						:nested-archive="false"
						:slots="slots"
						:tabs="tabs"
						:side-panels="sidePanels"
						:counts="counts"
						:reset-key="resetKey"
						:ready="ready"
					/>
				</template>
			</FilePageLayout>
		</ArchiveSession>
	</section>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { ArchiveSession, FilePageLayout } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectReviewArchive } from '~/providers/project-review/review-archive'

import ReviewWorkbench from './review-workbench.vue'
import type {
	ProjectReviewSidePanel,
	ProjectReviewSlot,
	ProjectReviewSlots,
	ProjectReviewTab,
} from './types'

/**
 * The review's layout: a workbench of the review's panels and tabs, along with the archive it
 * shows (by default the latest version's primary file), either spread among those panels or in a
 * single panel of its own, as the settings choose.
 */
defineProps<{
	tabs: readonly ProjectReviewTab[]
	sidePanels: readonly ProjectReviewSidePanel[]
	/** Item counts shown in the panels' tabs, e.g. how many issues there are. */
	counts?: Partial<Record<ProjectReviewSlot, number>>
	resetKey: string
	ready: boolean
}>()
const slots = defineSlots<ProjectReviewSlots>()
const archive = injectReviewArchive()
const settings = useModerationSettings()

/** Whether the whole archive is kept in a single panel, with a layout of its own. */
const nestedArchive = computed(
	() => settings.value.get(moderationSettings.General.ArchiveIntegration) === 'exclusive',
)

const reviewWorkbench = ref<InstanceType<typeof ReviewWorkbench>>()

/** The files layout's sidebar toggle shows and hides the archive's files, wherever they are. */
const sidebar = {
	isOpen: () => {
		const workbench = reviewWorkbench.value?.workbench
		if (!workbench) return true
		const tree = workbench.panels.value.find((panel) => panel.id === 'tree')
		if (!tree || !workbench.isOpen('tree')) return false
		const host = workbench.regionOf(tree)
		return host === 'center' || workbench.visible[host].value
	},
	setOpen: (open: boolean) => {
		const workbench = reviewWorkbench.value?.workbench
		const tree = workbench?.panels.value.find((panel) => panel.id === 'tree')
		if (!workbench || !tree) return
		const host = workbench.regionOf(tree)
		if (open) workbench.revealPanel('tree')
		else if (host === 'center') workbench.closePanel('tree')
		else workbench.setRegionVisible(host, false)
	},
}

watch(archive.openRequests, () => {
	const workbench = reviewWorkbench.value
	workbench?.reveal(workbench.filesPanel)
})

defineExpose({
	reveal: (slot: ProjectReviewSlot) => reviewWorkbench.value?.reveal(slot),
})
</script>

<style scoped>
.project-review {
	height: 100dvh;
	width: 100%;
}
</style>
