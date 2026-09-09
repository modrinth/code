<template>
	<div
		class="mod-review-shell sticky top-0 flex overflow-hidden border-0 border-t border-solid border-divider"
	>
		<ModerationReviewSidebar
			:project="project"
			:project-v3="projectV3"
			:organization="organization"
			:members="members"
			:creators-loading="creatorsLoading"
			:is-server-project="isServerProject"
			:server-data-loaded="serverDataLoaded"
			:server-required-content="serverRequiredContent"
			:server-recommended-version="serverRecommendedVersion"
			:server-supported-versions="serverSupportedVersions"
			:server-modpack-loaders="serverModpackLoaders"
		/>
		<ModerationReviewPanels dock="main" />
		<ModerationReviewPipHost v-if="pipOpen" />
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { computed } from 'vue'

import { useModerationReviewLayout } from '~/services/moderation/review-layout'

import ModerationReviewPanels from './ModerationReviewPanels.vue'
import ModerationReviewPipHost from './ModerationReviewPipHost.vue'
import ModerationReviewSidebar from './ModerationReviewSidebar.vue'

type LooseProject = Labrinth.Projects.v2.Project & Record<string, unknown>

withDefaults(
	defineProps<{
		project: LooseProject
		projectV3: Labrinth.Projects.v3.Project
		organization?: unknown
		members?: unknown[]
		creatorsLoading?: boolean
		isServerProject?: boolean
		serverDataLoaded?: boolean
		serverRequiredContent?: unknown
		serverRecommendedVersion?: unknown
		serverSupportedVersions?: unknown[]
		serverModpackLoaders?: unknown[]
	}>(),
	{
		organization: null,
		members: () => [],
		creatorsLoading: false,
		isServerProject: false,
		serverDataLoaded: false,
		serverRequiredContent: null,
		serverRecommendedVersion: null,
		serverSupportedVersions: () => [],
		serverModpackLoaders: () => [],
	},
)

const layout = useModerationReviewLayout()
const pipOpen = computed(() => layout.pipOpen.value)
</script>

<style scoped>
.mod-review-shell {
	height: calc(100dvh - 4.5rem);
}
</style>
