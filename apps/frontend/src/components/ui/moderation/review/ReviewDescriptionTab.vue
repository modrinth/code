<template>
	<div class="flex flex-col gap-4">
		<section class="rounded-xl border border-solid border-divider bg-surface-2 p-3">
			<h2 class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">Summary</h2>
			<p class="m-0 mt-1 whitespace-pre-wrap text-primary">
				{{ projectV2.description || '—' }}
			</p>
		</section>

		<section v-if="projectV2.body" class="card">
			<ProjectPageDescription :description="projectV2.body" />
		</section>
		<p v-else class="m-0 text-secondary">No description provided.</p>
	</div>
</template>

<script setup lang="ts">
import { injectProjectPageContext, ProjectPageDescription } from '@modrinth/ui'

const { projectV2 } = injectProjectPageContext()
</script>
