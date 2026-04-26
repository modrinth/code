<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">{{
			formatMessage(messages.knownModpackPrompt)
		}}</span>
		<Combobox
			v-model="ctx.modpackSearchProjectId.value"
			:options="ctx.modpackSearchOptions.value"
			searchable
			:search-placeholder="formatMessage(messages.searchModpackPlaceholder)"
			:no-options-message="
				searchLoading
					? formatMessage(commonMessages.loadingLabel)
					: formatMessage(messages.noResultsFound)
			"
			:disable-search-filter="true"
			@search-input="(query) => handleSearch(query)"
		>
			<template #option-suffix>
				<RightArrowIcon
					class="size-5 shrink-0 text-secondary opacity-0 transition-opacity group-hover/option:opacity-100 group-data-[focused=true]/option:opacity-100"
				/>
			</template>
		</Combobox>
		<div class="flex items-center gap-3">
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
			<span class="text-sm text-secondary">{{ formatMessage(commonMessages.orLabel) }}</span>
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
		</div>
		<div class="flex gap-3">
			<ButtonStyled type="outlined">
				<button class="flex-1 !border-surface-4" @click="triggerFileInput">
					<ImportIcon />
					{{ formatMessage(messages.importModpack) }}
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button class="flex-1" @click="ctx.browseModpacks()">
					<CompassIcon />
					{{ formatMessage(messages.browseModpacks) }}
				</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { CompassIcon, ImportIcon, RightArrowIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { defineAsyncComponent, h, onMounted, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectFilePicker } from '../../../../providers'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import Combobox from '../../../base/Combobox.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const debug = useDebugLogger('ModpackStage')
const ctx = injectCreationFlowContext()
const filePicker = injectFilePicker()
const { formatMessage } = useVIntl()

const searchLoading = ref(false)

const messages = defineMessages({
	knownModpackPrompt: {
		id: 'creation-flow.modal.modpack.known-modpack.prompt',
		defaultMessage: 'Already know the modpack you want to install?',
	},
	searchModpackPlaceholder: {
		id: 'creation-flow.modal.modpack.search.placeholder',
		defaultMessage: 'Search for modpack',
	},
	noResultsFound: {
		id: 'creation-flow.modal.modpack.search.no-results',
		defaultMessage: 'No results found',
	},
	importModpack: {
		id: 'creation-flow.modal.modpack.action.import',
		defaultMessage: 'Import modpack',
	},
	browseModpacks: {
		id: 'creation-flow.modal.modpack.action.browse',
		defaultMessage: 'Browse modpacks',
	},
})

function proceedWithModpack() {
	debug('proceedWithModpack:', {
		flowType: ctx.flowType,
		modpackSelection: ctx.modpackSelection.value,
	})
	if (ctx.flowType === 'instance') {
		ctx.finish()
	} else {
		ctx.modal.value?.setStage('final-config')
	}
}

const search = async (query: string) => {
	query = query.trim()
	debug('search() called:', { query, trimmed: query })

	try {
		debug('search() calling API...', {
			query: query || undefined,
			facets: [['project_type:modpack']],
			limit: 10,
		})
		const results = await ctx.searchModpacks(query, 10)
		debug('search() API returned:', {
			totalHits: results.total_hits,
			hitCount: results.hits.length,
			firstHit: results.hits[0]?.title,
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
		debug('search() options set:', {
			optionCount: ctx.modpackSearchOptions.value.length,
			labels: ctx.modpackSearchOptions.value.map((o) => o.label),
		})
	} catch (err) {
		debug('search() ERROR:', err)
		ctx.modpackSearchOptions.value = []
	}
	searchLoading.value = false
	debug('search() done, searchLoading:', searchLoading.value)
}

const handleSearch = async (query: string) => {
	debug('handleSearch() called:', { query })
	searchLoading.value = true
	await search(query)
}

onMounted(() => {
	debug('onMounted() firing, resetting and calling search("")')
	ctx.modpackSearchProjectId.value = undefined
	search('')
})

// When a project is selected via search, fetch its latest version and auto-proceed
watch(
	() => ctx.modpackSearchProjectId.value,
	async (projectId, oldProjectId) => {
		if (projectId === oldProjectId) return

		ctx.modpackSearchVersionId.value = undefined
		ctx.modpackVersionOptions.value = []

		if (!projectId) return

		const hit = ctx.modpackSearchHits.value[projectId]

		// Always fetch the actual latest version from the API since search index can be stale
		try {
			const versions = await ctx.getProjectVersions(projectId)
			if (ctx.modpackSearchProjectId.value !== projectId) return
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
