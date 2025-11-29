<template>
	<div class="w-[496px] max-w-[496px]">
		<div class="grid gap-6">
			<div class="flex flex-col gap-4">
				<span class="font-semibold text-contrast">Add dependency</span>
				<div class="grid gap-2.5 rounded-2xl border border-solid border-surface-5 p-4">
					<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
					<Combobox
						v-model="newDependencyId"
						placeholder="Select project"
						:options="options"
						:searchable="true"
						search-placeholder="Search by name, slug, or paste ID..."
						@search-input="(query) => handleSearch(query)"
					/>
				</div>
				<ButtonStyled>
					<button
						class="self-start"
						@click="addDependency('project', newDependencyId, newDependencyType)"
					>
						Add Dependency
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { injectModrinthClient } from '@modrinth/ui'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'
import { defineAsyncComponent, h } from 'vue'

const newDependencyId = ref('')
const newDependencyType = ref<'required' | 'optional' | 'incompatible' | 'embedded' | null>(null)
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

const addDependency = (
	_mode: 'project' | 'version',
	_id: string,
	_type: 'required' | 'optional' | 'incompatible' | 'embedded' | null,
) => {
	// todo
}
</script>
