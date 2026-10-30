<template>
	<div class="review-details -mt-1 min-w-0 pb-10 text-sm text-secondary">
		<p v-if="isLoading" role="status" class="m-0">Loading...</p>
		<div v-else-if="error" role="alert">
			<p class="m-0">Could not load project data</p>
			<Button @click="refresh">Retry</Button>
		</div>
		<p v-else-if="!project" class="m-0">No project selected</p>
		<template v-else>
			<Title v-if="!settings.get(general.HideLeftPanelTitle)" />
			<Slug v-if="!settings.get(general.HideLeftPanelSlug)" />
			<Icon v-if="!settings.get(general.HideLeftPanelIcon)" />
			<Summary v-if="!settings.get(general.HideLeftPanelSummary)" />
			<License v-if="!project.minecraft_server && !settings.get(general.HideLeftPanelLicense)" />
			<Tags v-if="!settings.get(general.HideLeftPanelTags)" />
			<Links v-if="!settings.get(general.HideLeftPanelLinks)" />
			<Compatibility
				v-if="!project.minecraft_server && !settings.get(general.HideLeftPanelCompatibility)"
			/>
			<ServerDetails
				v-if="project.minecraft_server && !settings.get(general.HideLeftPanelServerDetails)"
			/>
			<Members v-if="!settings.get(general.HideLeftPanelMembers)" />
			<Details v-if="!settings.get(general.HideLeftPanelDetails)" />
		</template>
	</div>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { Button } from '@modrinth/ui'

import { useModerationSettings } from '~/composables/moderation'
import { injectProjectReviewPageContext } from '~/providers/project-review'

import Members from './members/index.vue'
import Compatibility from './project/compatibility.vue'
import Details from './project/details.vue'
import Icon from './project/icon.vue'
import License from './project/license.vue'
import Links from './project/links.vue'
import ServerDetails from './project/server-details.vue'
import Slug from './project/slug.vue'
import Summary from './project/summary.vue'
import Tags from './project/tags.vue'
import Title from './project/title.vue'

const settings = useModerationSettings()
const general = moderationSettings.General
const { project, isLoading, error, refresh } = injectProjectReviewPageContext()
</script>

<style scoped>
.review-details > :deep(.review-section:first-child) {
	@apply border-t-0;
}
.review-details :deep(.review-badge) {
	@apply rounded bg-button-bg px-1.5 py-0.5 text-[0.6875rem];
}
</style>
