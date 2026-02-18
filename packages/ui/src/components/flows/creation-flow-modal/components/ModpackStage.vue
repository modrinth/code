<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">Already know what Modpack you want?</span>
		<Combobox
			v-model="ctx.modpackSearchProjectId.value"
			placeholder="Select modpack"
			:options="ctx.modpackSearchOptions.value"
			:searchable="true"
			search-placeholder="Search Modpacks"
			:no-options-message="searchLoading ? 'Loading...' : 'No results found'"
			:disable-search-filter="true"
			@search-input="(query) => handleSearch(query)"
		/>
		<Transition
			enter-from-class="opacity-0 max-h-0"
			enter-active-class="transition-all duration-300 ease-in-out overflow-hidden"
			enter-to-class="opacity-100 max-h-24"
			leave-from-class="opacity-100 max-h-24"
			leave-active-class="transition-all duration-200 ease-in-out overflow-hidden"
			leave-to-class="opacity-0 max-h-0"
		>
			<Combobox
				v-if="ctx.modpackSearchProjectId.value"
				v-model="ctx.modpackSearchVersionId.value"
				placeholder="Select version"
				:options="ctx.modpackVersionOptions.value"
				:searchable="true"
				:show-icon-in-selected="true"
				search-placeholder="Search versions..."
				:no-options-message="versionsLoading ? 'Loading...' : 'No versions found'"
			/>
		</Transition>
		<div class="flex items-center gap-3">
			<div class="flex-1 bg-surface-5 h-[1px] w-full" />
			<span class="text-sm text-secondary">Or</span>
			<div class="flex-1 bg-surface-5 h-[1px] w-full" />
		</div>
		<div class="flex gap-3">
			<ButtonStyled type="outlined">
				<button class="flex-1 !border-surface-4" @click="triggerFileInput">
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
import VersionChannelIndicator from '../../../version/VersionChannelIndicator.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const ctx = injectCreationFlowContext()
const { labrinth } = injectModrinthClient()
const filePicker = injectFilePicker()

const searchLoading = ref(false)
const versionsLoading = ref(false)

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

watch(
	() => ctx.modpackSearchProjectId.value,
	async (projectId, oldProjectId) => {
		// Don't reset if the project hasn't actually changed (e.g. re-mount)
		if (projectId === oldProjectId) return

		ctx.modpackSearchVersionId.value = undefined
		ctx.modpackVersionOptions.value = []

		if (!projectId) return

		versionsLoading.value = true
		try {
			const versions = await labrinth.versions_v3.getProjectVersions(projectId)
			ctx.modpackVersionOptions.value = versions.map((version) => {
				const gameVersion = version.game_versions?.[0]
				const label = gameVersion ? `${version.name} for ${gameVersion}` : version.name

				return {
					label,
					value: version.id,
					icon: defineAsyncComponent(() =>
						Promise.resolve({
							setup: () => () =>
								h(VersionChannelIndicator, {
									channel: version.version_type,
									size: 'xs',
								}),
						}),
					),
				}
			})
		} catch {
			ctx.modpackVersionOptions.value = []
		}
		versionsLoading.value = false

		if (ctx.modpackVersionOptions.value.length > 0) {
			ctx.modpackSearchVersionId.value = ctx.modpackVersionOptions.value[0].value
		}
	},
)

watch(
	() => ctx.modpackSearchVersionId.value,
	(versionId, oldVersionId) => {
		if (!versionId || !ctx.modpackSearchProjectId.value) return
		// Don't re-navigate if value hasn't changed (e.g. re-mount restoring state)
		if (versionId === oldVersionId) return

		const hit = ctx.modpackSearchHits.value[ctx.modpackSearchProjectId.value]
		if (hit) {
			ctx.modpackSelection.value = {
				projectId: ctx.modpackSearchProjectId.value,
				versionId,
				name: hit.title,
				iconUrl: hit.iconUrl,
			}
		}
	},
)

async function triggerFileInput() {
	const picked = await filePicker.pickModpackFile()
	if (picked) {
		ctx.modpackFile.value = picked.file
		ctx.modpackFilePath.value = picked.path ?? null
	}
}
</script>
