<template>
	<Combobox
		v-model="projectId"
		placeholder="Select project"
		:options="options"
		:searchable="true"
		search-placeholder="Search by name, slug, or paste ID..."
		@search-input="(query) => handleSearch(query)"
	/>
</template>

<script lang="ts" setup>
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import type { DropdownOption } from '@modrinth/ui/src/components/base/Combobox.vue'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'
import { defineAsyncComponent, h } from 'vue'

const { addNotification } = injectNotificationManager()
const projectId = defineModel<string>()

const options = ref<DropdownOption<string>[]>([])

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
				facets: JSON.stringify([['project_type:mod']]),
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
		} catch (error: any) {
			addNotification({
				title: 'An error occurred',
				text: error.data ? error.data.description : error,
				type: 'error',
			})
		}
	}, 500)
}
</script>
