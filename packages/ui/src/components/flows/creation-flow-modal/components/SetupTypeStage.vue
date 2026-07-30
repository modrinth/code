<template>
	<div class="flex flex-col gap-4">
		<span class="font-semibold text-contrast">
			{{ formatMessage(messages.knownProjectPrompt) }}
		</span>
		<Combobox
			v-model="ctx.projectSearchProjectId.value"
			v-tooltip="ctx.finishDisabled.value ? ctx.finishDisabledTooltip.value : undefined"
			:options="ctx.projectSearchOptions.value"
			searchable
			:disabled="ctx.finishDisabled.value"
			:search-placeholder="formatMessage(messages.searchProjectPlaceholder)"
			:no-options-message="
				searchLoading
					? formatMessage(commonMessages.loadingLabel)
					: formatMessage(messages.noResultsFound)
			"
			:disable-search-filter="true"
			@search-input="handleSearch"
		>
			<template #option-suffix="{ item }">
				<div
					class="flex shrink-0 items-center gap-1.5 text-sm font-semibold text-secondary opacity-0 transition-opacity group-hover/option:opacity-100 group-data-[focused=true]/option:opacity-100"
				>
					<span>
						{{
							formatMessage(
								isModpackOption(item.value) ? messages.installModpack : messages.createInstance,
							)
						}}
					</span>
					<DownloadIcon v-if="isModpackOption(item.value)" class="size-5 shrink-0" />
					<RightArrowIcon v-else class="size-5 shrink-0" />
				</div>
			</template>
		</Combobox>

		<div class="flex items-center gap-3">
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
			<span class="text-sm text-secondary">{{ formatMessage(commonMessages.orLabel) }}</span>
			<div class="h-[1px] w-full flex-1 bg-surface-5" />
		</div>

		<span class="font-semibold text-contrast">
			{{ setupTypeTitle }}
		</span>

		<template v-if="ctx.flowType === 'instance'">
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customSetupTitle)"
					:description="formatMessage(messages.customSetupDescription)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="CompassIcon"
					:title="formatMessage(messages.modpackBaseTitle)"
					:description="formatMessage(messages.modpackBaseDescription)"
					@click="browseModpacks"
				/>
				<BigOptionButton
					:icon="ImportIcon"
					:title="formatMessage(messages.uploadModpackTitle)"
					:description="formatMessage(messages.uploadModpackDescription)"
					@click="triggerFileInput"
				/>
				<BigOptionButton
					:icon="BoxImportIcon"
					:title="formatMessage(messages.importInstanceTitle)"
					:description="formatMessage(messages.importInstanceDescription)"
					@click="ctx.setImportMode()"
				/>
			</div>
			<span class="text-sm text-secondary">
				{{ formatMessage(messages.instanceDescription) }}
			</span>
		</template>

		<template v-else>
			<div class="flex flex-col gap-3">
				<BigOptionButton
					:icon="CompassIcon"
					:title="formatMessage(messages.modpackBaseTitle)"
					:description="formatMessage(messages.modpackBaseDescription)"
					@click="browseModpacks"
				/>
				<BigOptionButton
					:icon="ImportIcon"
					:title="formatMessage(messages.uploadModpackTitle)"
					:description="formatMessage(messages.uploadModpackDescription)"
					@click="triggerFileInput"
				/>
				<BigOptionButton
					:icon="BoxesIcon"
					:title="formatMessage(messages.customSetupTitle)"
					:description="formatMessage(messages.customSetupDescription)"
					@click="setSetupType('custom')"
				/>
				<BigOptionButton
					:icon="BoxIcon"
					:title="formatMessage(messages.vanillaMinecraftTitle)"
					:description="formatMessage(messages.vanillaMinecraftDescription)"
					@click="setSetupType('vanilla')"
				/>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	BoxesIcon,
	BoxIcon,
	BoxImportIcon,
	CompassIcon,
	DownloadIcon,
	ImportIcon,
	RightArrowIcon,
} from '@modrinth/assets'
import { commonMessages, defineMessages, useVIntl } from '@modrinth/ui'
import { computed, defineAsyncComponent, h, onMounted, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import { injectFilePicker } from '../../../../providers'
import BigOptionButton from '../../../base/BigOptionButton.vue'
import Combobox from '../../../base/Combobox.vue'
import { injectCreationFlowContext } from '../creation-flow-context'

const debug = useDebugLogger('SetupTypeStage')
const ctx = injectCreationFlowContext()
const filePicker = injectFilePicker()
const { setSetupType: _setSetupType } = ctx
const { formatMessage } = useVIntl()

const searchLoading = ref(false)

const messages = defineMessages({
	knownProjectPrompt: {
		id: 'creation-flow.modal.project.known-project.prompt',
		defaultMessage: 'Already know what you want to play?',
	},
	searchProjectPlaceholder: {
		id: 'creation-flow.modal.project.search.placeholder',
		defaultMessage: 'Search mods, modpacks, and more...',
	},
	noResultsFound: {
		id: 'creation-flow.modal.project.search.no-results',
		defaultMessage: 'No results found',
	},
	installModpack: {
		id: 'creation-flow.modal.project.search.install-modpack',
		defaultMessage: 'Install modpack',
	},
	createInstance: {
		id: 'creation-flow.modal.project.search.create-instance',
		defaultMessage: 'Create instance',
	},
	instanceTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.instance',
		defaultMessage: 'Choose instance type',
	},
	installationTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.installation',
		defaultMessage: 'Select installation type',
	},
	worldTypeTitle: {
		id: 'creation-flow.modal.setup-type.title.world',
		defaultMessage: 'Select world type',
	},
	customSetupTitle: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.title',
		defaultMessage: 'Custom setup',
	},
	customSetupDescription: {
		id: 'creation-flow.modal.setup-type.option.custom-setup.description',
		defaultMessage: 'Start from scratch by picking a loader and game version.',
	},
	modpackBaseTitle: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.title',
		defaultMessage: 'Start from a mod or modpack',
	},
	modpackBaseDescription: {
		id: 'creation-flow.modal.setup-type.option.modpack-base.description',
		defaultMessage: 'Choose a project and we’ll use its latest version.',
	},
	uploadModpackTitle: {
		id: 'creation-flow.modal.setup-type.option.upload-modpack.title',
		defaultMessage: 'Upload a modpack',
	},
	uploadModpackDescription: {
		id: 'creation-flow.modal.setup-type.option.upload-modpack.description',
		defaultMessage: 'Install a modpack from an .mrpack file on your device.',
	},
	importInstanceTitle: {
		id: 'creation-flow.modal.setup-type.option.import-instance.title',
		defaultMessage: 'Import instance',
	},
	importInstanceDescription: {
		id: 'creation-flow.modal.setup-type.option.import-instance.description',
		defaultMessage: 'Import an instance from Prism, CurseForge, or similar.',
	},
	instanceDescription: {
		id: 'creation-flow.modal.setup-type.instance.description',
		defaultMessage: 'An instance is a Minecraft setup with a specific loader, version, and mods.',
	},
	vanillaMinecraftTitle: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.title',
		defaultMessage: 'Vanilla Minecraft',
	},
	vanillaMinecraftDescription: {
		id: 'creation-flow.modal.setup-type.option.vanilla-minecraft.description',
		defaultMessage: 'Classic Minecraft with no mods or plugins.',
	},
})

const setupTypeTitle = computed(() => {
	if (ctx.flowType === 'instance') {
		return formatMessage(messages.instanceTypeTitle)
	}
	if (ctx.flowType === 'server-onboarding' || ctx.flowType === 'reset-server') {
		return formatMessage(messages.installationTypeTitle)
	}
	return formatMessage(messages.worldTypeTitle)
})

function isModpackOption(projectId: string) {
	return ctx.projectSearchHits.value[projectId]?.projectType === 'modpack'
}

function setSetupType(type: 'custom' | 'vanilla') {
	debug('selected:', type)
	_setSetupType(type)
}

function selectModpack() {
	debug('selected: modpack')
	_setSetupType('modpack')
}

function proceedWithModpack() {
	if (ctx.finishDisabled.value) return

	if (ctx.flowType === 'instance') {
		ctx.finish()
	} else {
		ctx.modal.value?.setStage('final-config')
	}
}

function browseModpacks() {
	if (ctx.finishDisabled.value) return

	selectModpack()
	ctx.browseModpacks()
}

async function triggerFileInput() {
	if (ctx.finishDisabled.value) return

	const picked = await filePicker.pickModpackFile({
		readFile: ctx.flowType !== 'instance',
	})
	if (!picked) return

	selectModpack()
	ctx.modpackFile.value = picked.file ?? null
	ctx.modpackFilePath.value = picked.path ?? null
	proceedWithModpack()
}

async function search(query: string) {
	try {
		if (!query.trim()) {
			ctx.projectSearchOptions.value = []
			return
		}
		const results = await ctx.searchProjects(query.trim(), 10)

		ctx.projectSearchHits.value = {}
		for (const hit of results.hits) {
			ctx.projectSearchHits.value[hit.project_id] = {
				title: hit.title,
				iconUrl: hit.icon_url,
				latestVersion: hit.latest_version,
				projectType: hit.project_type ?? 'modpack',
			}
		}

		ctx.projectSearchOptions.value = results.hits.map((hit) => ({
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
		debug('project search failed:', error)
		ctx.projectSearchOptions.value = []
	} finally {
		searchLoading.value = false
	}
}

async function handleSearch(query: string) {
	searchLoading.value = true
	await search(query)
}

onMounted(() => {
	ctx.projectSearchProjectId.value = undefined
	search('')
})

watch(
	() => ctx.projectSearchProjectId.value,
	async (projectId, oldProjectId) => {
		if (projectId === oldProjectId) return

		if (!projectId) return
		const hit = ctx.projectSearchHits.value[projectId]

		if (ctx.flowType === 'instance') {
			void ctx.selectProject(projectId, hit?.projectType ?? 'mod')
			return
		}

		try {
			const versions = await ctx.getProjectVersions(projectId)
			if (ctx.projectSearchProjectId.value !== projectId || versions.length === 0) return

			selectModpack()
			ctx.modpackSelection.value = {
				projectId,
				versionId: versions[0].id,
				name: hit?.title ?? '',
				iconUrl: hit?.iconUrl,
			}
			proceedWithModpack()
		} catch (error) {
			debug('failed to load project versions:', error)
		}
	},
)
</script>
