<template>
	<section class="normal-page__content">
		<div v-if="project.body" class="card">
			<ProjectPageDescription :description="project.body" />
		</div>
		<p v-else class="ml-2">
			No description provided.
			<NuxtLink to="settings/description">
				<span class="font-medium hover:underline">
					Visit project settings to add your description.
				</span>
			</NuxtLink>
		</p>
	</section>
</template>

<script setup>
import { ProjectPageDescription } from '@modrinth/ui'

defineProps({
	project: {
		type: Object,
		default() {
			return {}
		},
	},
	versions: {
		type: Array,
		default() {
			return {}
		},
	},
	members: {
		type: Array,
		default() {
			return {}
		},
	},
	organization: {
		type: Object,
		default() {
			return {}
		},
	},
})
</script>
