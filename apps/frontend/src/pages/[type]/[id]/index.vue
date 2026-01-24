<template>
	<section class="normal-page__content">
		<div v-if="projectV2.body" class="card">
			<ProjectPageDescription :description="projectV2.body" />
		</div>
		<p v-else class="ml-2">
			No description provided. Visit
			<NuxtLink :to="`${route.fullPath}/settings/description`">
				<span class="font-medium text-green hover:underline">project settings</span> to
			</NuxtLink>
			add your description.
		</p>
	</section>
</template>

<script setup>
import { injectProjectPageContext, ProjectPageDescription } from '@modrinth/ui'

const route = useRoute()

const { projectV2 } = injectProjectPageContext()
</script>
