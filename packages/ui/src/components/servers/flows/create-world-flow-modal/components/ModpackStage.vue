<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Already know what Modpack you want?</span>
		<div class="flex flex-col gap-2">
			<StyledInput
				v-model="searchQuery"
				:icon="SearchIcon"
				type="search"
				placeholder="Search Modpacks"
			/>
			<div
				v-if="searchResults.length > 0"
				class="flex max-h-48 flex-col gap-1 overflow-y-auto rounded-xl border border-solid border-surface-5 bg-surface-2 p-1"
			>
				<button
					v-for="result in searchResults"
					:key="result.project_id"
					class="flex items-center gap-3 rounded-lg border-none bg-transparent p-2 text-left transition-colors hover:bg-surface-3"
					@click="selectModpack(result)"
				>
					<Avatar :src="result.icon_url" size="2rem" :alt="result.title" />
					<div class="flex min-w-0 flex-col">
						<span class="truncate text-sm font-bold text-contrast">{{ result.title }}</span>
						<span class="truncate text-xs text-secondary">{{ result.description }}</span>
					</div>
				</button>
			</div>
		</div>
		<div class="flex items-center gap-3">
			<hr class="flex-1 border-surface-5" />
			<span class="text-sm text-secondary">Or</span>
			<hr class="flex-1 border-surface-5" />
		</div>
		<div class="flex gap-3">
			<ButtonStyled type="outlined" class="flex-1">
				<button @click="triggerFileInput">
					<ImportIcon />
					Import modpack
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand" class="flex-1">
				<!-- TODO: emit browse-modpacks event through the modal -->
				<button @click="ctx.modal.value?.hide()">
					<SearchIcon />
					Browse modpacks
				</button>
			</ButtonStyled>
		</div>
		<input ref="fileInput" type="file" accept=".mrpack" class="hidden" @change="onFileSelected" />
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ImportIcon, SearchIcon } from '@modrinth/assets'
import { useDebounceFn } from '@vueuse/core'
import { ref, watch } from 'vue'

import { injectModrinthClient } from '../../../../../providers'
import Avatar from '../../../../base/Avatar.vue'
import ButtonStyled from '../../../../base/ButtonStyled.vue'
import StyledInput from '../../../../base/StyledInput.vue'
import { injectCreateWorldContext } from '../create-world-context'

const ctx = injectCreateWorldContext()
const client = injectModrinthClient()

const searchQuery = ref('')
const searchResults = ref<Labrinth.Projects.v2.SearchResultHit[]>([])
const fileInput = ref<HTMLInputElement | null>(null)

const debouncedSearch = useDebounceFn(async (query: string) => {
	if (!query.trim()) {
		searchResults.value = []
		return
	}
	try {
		const results = await client.labrinth.projects_v2.search({
			query,
			facets: [['project_type:modpack']],
			limit: 10,
		})
		searchResults.value = results.hits
	} catch {
		searchResults.value = []
	}
}, 500)

watch(searchQuery, (query) => {
	if (!query?.toString().trim()) {
		searchResults.value = []
		return
	}
	debouncedSearch(query.toString())
})

function selectModpack(result: Labrinth.Projects.v2.SearchResultHit) {
	ctx.modpackSelection.value = {
		projectId: result.project_id,
		versionId: result.latest_version ?? '',
		name: result.title,
		iconUrl: result.icon_url,
	}
	ctx.modal.value?.setStage('final-config')
}

function triggerFileInput() {
	fileInput.value?.click()
}

function onFileSelected(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	if (file) {
		ctx.modpackFile.value = file
		ctx.modal.value?.setStage('final-config')
	}
}
</script>
