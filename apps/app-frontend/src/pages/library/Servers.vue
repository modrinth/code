<script setup lang="ts">
import { ref, onMounted } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { get_search_results_v3 } from '@/helpers/cache.js'
import type { Instance } from '@/store/instance.js'

const props = defineProps({
	instances: {
		type: Array as () => Instance[],
		required: true,
	},
})

const projects = ref<Labrinth.Search.v3.Project[]>([])
const loading = ref(true)

onMounted(async () => {
	try {
		const results = await get_search_results_v3(
			'author=fraa2a&facets=[["project_type:modpack"],["project_type:resourcepack"]]&limit=20',
		)
		projects.value = results.hits || []
	} catch (e) {
		console.error('Failed to fetch fraa2a projects', e)
	} finally {
		loading.value = false
	}
})
</script>
<template>
	<div class="p-4">
		<h2 class="text-2xl font-bold mb-4">Shop</h2>
		<p class="text-primary mb-6">
			Modpack e texture pack creati da <strong>fraa2a</strong>
		</p>

		<div v-if="loading" class="text-center py-8 text-secondary">
			Caricamento in corso...
		</div>

		<GridDisplay
			v-else-if="projects && projects.length > 0"
			label="Prodotti fraa2a"
			:instances="instances"
			:projects="projects"
		/>

		<div v-else class="text-center py-8 text-secondary">
			Nessun progetto trovato.
		</div>
	</div>
</template>
