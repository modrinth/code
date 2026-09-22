<template>
	<section class="flex h-full min-h-0 flex-col gap-2.5 overflow-hidden">
		<ReviewPanel
			v-if="panels.resolve({ kind: 'permissions' })"
			mode="inline"
			:target="{ kind: 'permissions' }"
			class="shrink-0 !overflow-visible"
		/>
		<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
			<ProjectPermissions
				v-if="project"
				:key="project.id"
				:project="project"
				:members="members"
				is-moderator
			/>
		</div>
	</section>
</template>

<script setup lang="ts">
import ProjectPermissions from '~/components/ui/project-settings/modpack-permissions/ProjectPermissions.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import ReviewPanel from '../review-panel/index.vue'

const panels = injectReviewPanels()
const { project, members } = injectProjectReviewPageContext()
</script>
