<template>
	<Combobox
		:model-value="modelValue"
		placeholder="Select project"
		:options="options"
		:searchable="true"
		search-placeholder="Search by name, slug, or paste ID..."
		@update:model-value="emit('update:modelValue', $event)"
		@search-input="(query) => handleSearch(query)"
	/>
</template>

<script lang="ts" setup>
import { injectModrinthClient } from '@modrinth/ui'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'
import { defineAsyncComponent, h } from 'vue'

interface Props {
	modelValue: string
}

interface Emits {
	(e: 'update:modelValue', value: string): void
}

withDefaults(defineProps<Props>(), {
	modelValue: '',
	dependencyType: null,
})

const emit = defineEmits<Emits>()

const options = ref<Array<{ label: string; value: string; icon: Component }>>([])

const client = injectModrinthClient()
let searchTimeout: ReturnType<typeof setTimeout> | null = null

const handleSearch = async (query: string) => {
	if (searchTimeout) clearTimeout(searchTimeout)

	if (!query.trim()) return

	searchTimeout = setTimeout(async () => {
		try {
			const results = await client.labrinth.projects_v2.search({
				query: query,
				limit: 20,
				// @ts-ignore - for some reason, facet need to be wrapped in another array. either type is wrong or api client implementation is wrong
				facets: [[['project_type:mod']]],
			})

			options.value = results.hits.map((hit) => ({
				label: hit.title,
				value: hit.project_id,
				icon: defineAsyncComponent(() =>
					Promise.resolve({
						setup: () => () =>
							h('img', {
								src: hit.icon_url,
								alt: hit.title,
								class: 'h-5 w-5 rounded',
							}),
					}),
				),
			}))
		} catch (error) {
			console.error('Search failed:', error)
		}
	}, 500)
}
</script>
