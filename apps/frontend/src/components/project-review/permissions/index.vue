<template>
	<section ref="interactionScope" class="flex h-full min-h-0 flex-col gap-2.5 overflow-hidden p-2">
		<ReviewPanel
			v-if="panels.resolve({ kind: 'permissions' })"
			mode="inline"
			:target="{ kind: 'permissions' }"
			:interaction-scope="interactionScope"
			class="shrink-0"
		/>
		<div class="min-h-0 flex-1 overflow-y-auto overscroll-contain">
			<ProjectPermissions
				v-if="project"
				:key="project.id"
				:project="project"
				:members="members"
				:pixelated="pixelated"
				is-moderator
				collapse-all-icon-only
				collapse-attributed-by-default
			>
				<template #image-viewer-actions="{ item }">
					<ImageViewerActions v-model:pixelated="pixelated" :src="item.src" />
				</template>
			</ProjectPermissions>
		</div>
	</section>
</template>

<script setup lang="ts">
import { useTemplateRef } from 'vue'

import ProjectPermissions from '~/components/ui/project-settings/modpack-permissions/ProjectPermissions.vue'
import { injectProjectReviewPageContext } from '~/providers/project-review'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import ImageViewerActions from '../image-viewer-actions.vue'
import ReviewPanel from '../review-panel/index.vue'

const panels = injectReviewPanels()
const interactionScope = useTemplateRef<HTMLElement>('interactionScope')
const { project, members, pixelated } = injectProjectReviewPageContext()
</script>
