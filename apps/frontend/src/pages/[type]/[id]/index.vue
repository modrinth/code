<template>
	<section class="normal-page__content">
		<div v-if="project.body" class="card">
			<ProjectPageDescription :description="project.body" />
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
import { ProjectPageDescription } from '@modrinth/ui'

const route = useRoute()

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
