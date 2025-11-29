<template>
	<div class="w-[496px] max-w-[496px]">
		<div class="grid gap-6">
			<div class="flex flex-col gap-4">
				<span class="font-semibold text-contrast">Add dependency</span>
				<div class="grid gap-2.5 rounded-2xl border border-solid border-surface-5 p-4">
					<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
					<div class="iconified-input w-full">
						<SearchIcon aria-hidden="true" class="text-lg" />
						<input
							v-model="query"
							class="h-12"
							autocomplete="off"
							spellcheck="false"
							type="text"
							:placeholder="`Search ${projectType.display}s...`"
							@input="handleSearch"
						/>
					</div>
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
import { SearchIcon } from '@modrinth/assets'
import { injectModrinthClient } from '@modrinth/ui'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'

const query = ref('')
const newDependencyId = ref('')
const newDependencyType = ref<'required' | 'optional' | 'incompatible' | 'embedded' | null>(null)
const projectType = ref({ display: 'mod' })

const client = injectModrinthClient()
let searchTimeout: ReturnType<typeof setTimeout> | null = null

const handleSearch = async () => {
	if (searchTimeout) clearTimeout(searchTimeout)

	if (!query.value.trim()) return

	searchTimeout = setTimeout(async () => {
		try {
			const results = await client.labrinth.projects_v2.search({
				query: query.value,
				limit: 20,
				// @ts-ignore - for some reason, facet need to be wrapped in another array. either type is wrong or api client implementation is wrong
				facets: [[['project_type:mod']]],
			})

			console.log('Search results:', results)
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
