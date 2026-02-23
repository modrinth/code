<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Already know the modpack you want to install?</span>
		<Combobox
			v-model="ctx.modpackSearchProjectId.value"
			:options="ctx.modpackSearchOptions.value"
			searchable
			search-placeholder="Search for modpack"
			:no-options-message="searchLoading ? 'Loading...' : 'No results found'"
			:disable-search-filter="true"
			@search-input="(query) => handleSearch(query)"
		/>
		<div class="flex items-center gap-3">
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
			<span class="text-sm text-secondary">or</span>
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
		</div>
		<div class="flex gap-3">
			<ButtonStyled type="outlined">
				<button
					disabled
					v-tooltip="`Not supported by backend yet.`"
					class="flex-1 !border-surface-4"
					@click="triggerFileInput"
				>
					<ImportIcon />
					Import modpack
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button class="flex-1" @click="ctx.browseModpacks()">
					<CompassIcon />
					Browse modpacks
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CompassIcon, ImportIcon } from '@modrinth/assets'
import { useDebounceFn } from '@vueuse/core'
import { defineAsyncComponent, h, ref, watch } from 'vue'

import { injectFilePicker, injectModrinthClient } from '../../../../providers'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Combobox from '../../../base/Combobox.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { labrinth } = injectModrinthClient()
const filePicker = injectFilePicker()

const searchLoading = ref(false)

function proceedWithModpack() {
	if (ctx.flowType === 'instance') {
		ctx.finish()
	} else {
		ctx.modal.value?.setStage('final-config')
	}
}

const search = async (query: string) => {
	query = query.trim()
	if (!query) {
		searchLoading.value = false
		return
	}

	try {
		const results = await labrinth.projects_v2.search({
			query,
			facets: [['project_type:modpack']],
			limit: 10,
		})

		ctx.modpackSearchHits.value = {}
		for (const hit of results.hits) {
			ctx.modpackSearchHits.value[hit.project_id] = {
				title: hit.title,
				iconUrl: hit.icon_url,
				latestVersion: hit.latest_version,
			}
		}

		ctx.modpackSearchOptions.value = results.hits.map((hit) => ({
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
	} catch {
		ctx.modpackSearchOptions.value = []
	}
	searchLoading.value = false
}

const throttledSearch = useDebounceFn(search, 500)

const handleSearch = async (query: string) => {
	searchLoading.value = true
	await throttledSearch(query)
}

// When a project is selected via search, fetch its latest version and auto-proceed
watch(
	() => ctx.modpackSearchProjectId.value,
	async (projectId, oldProjectId) => {
		if (projectId === oldProjectId) return

		ctx.modpackSearchVersionId.value = undefined
		ctx.modpackVersionOptions.value = []

		if (!projectId) return

		const hit = ctx.modpackSearchHits.value[projectId]

		// If the hit has a latestVersion, use it directly
		if (hit?.latestVersion) {
			ctx.modpackSelection.value = {
				projectId,
				versionId: hit.latestVersion,
				name: hit.title,
				iconUrl: hit.iconUrl,
			}
			proceedWithModpack()
			return
		}

		// Otherwise fetch versions and use the first one
		try {
			const versions = await labrinth.versions_v3.getProjectVersions(projectId)
			if (versions.length > 0) {
				const version = versions[0]
				ctx.modpackSelection.value = {
					projectId,
					versionId: version.id,
					name: hit?.title ?? '',
					iconUrl: hit?.iconUrl,
				}
				proceedWithModpack()
			}
		} catch {
			// Failed to fetch versions — do nothing
		}
	},
)

async function triggerFileInput() {
	const picked = await filePicker.pickModpackFile()
	if (picked) {
		ctx.modpackFile.value = picked.file
		ctx.modpackFilePath.value = picked.path ?? null
		proceedWithModpack()
	}
}
</script>
