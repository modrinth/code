<template>
	<div class="flex flex-col gap-2">
		<section>
			<ChecklistStageButtons stage-id="summary">
				<span class="min-w-0 whitespace-pre-wrap text-primary">{{
					projectV2.description || '—'
				}}</span>
			</ChecklistStageButtons>
		</section>

		<div class="my-1 h-px w-full bg-divider" />

		<section>
			<ChecklistStageButtons stage-id="description">
				<div v-if="projectV2.body" class="card">
					<ProjectPageDescription :description="projectV2.body" />
				</div>
				<p v-else class="m-0 text-secondary">No description provided.</p>
			</ChecklistStageButtons>
		</section>
	</div>
</template>

<script setup lang="ts">
import { injectProjectPageContext, ProjectPageDescription } from '@modrinth/ui'

import ChecklistStageButtons from './ChecklistStageButtons.vue'

const { projectV2 } = injectProjectPageContext()
</script>
