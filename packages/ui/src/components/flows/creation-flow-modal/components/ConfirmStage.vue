<template>
	<div class="space-y-6">
		<div
			class="flex flex-col gap-3 rounded-2xl bg-surface-2 p-4 border-surface-4 border border-solid"
		>
			<span class="text-lg font-semibold text-contrast">Summary</span>

			<div
				v-if="ctx.worldType.value === 'modpack' && ctx.modpackSelection.value"
				class="flex items-center gap-3 rounded-xl bg-surface-3 border-surface-4 border border-solid p-3"
			>
				<Avatar :src="ctx.modpackSelection.value.iconUrl" size="2.5rem" />
				<div class="flex flex-col">
					<span class="font-semibold text-contrast">
						{{ ctx.modpackSelection.value.name }}
					</span>
					<span v-if="selectedVersionLabel" class="text-sm text-secondary">
						{{ selectedVersionLabel }}
					</span>
				</div>
			</div>

			<div
				v-else-if="ctx.worldType.value === 'modpack' && ctx.modpackFile.value"
				class="flex items-center gap-3 rounded-xl bg-surface-3 border-surface-4 border border-solid p-3"
			>
				<FileIcon class="size-5 text-secondary" />
				<span class="font-semibold text-contrast">
					{{ ctx.modpackFile.value.name }}
				</span>
			</div>

			<div class="flex flex-col gap-2 text-sm">
				<template v-if="ctx.worldType.value !== 'modpack'">
					<div v-if="ctx.selectedLoader.value" class="flex items-center justify-between">
						<span class="text-secondary">Loader</span>
						<span class="font-medium text-contrast">
							{{ formatLoader(ctx.selectedLoader.value) }}
						</span>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-secondary">Game version</span>
						<span class="font-medium text-contrast">
							{{ ctx.selectedGameVersion.value }}
						</span>
					</div>
					<div
						v-if="ctx.selectedLoaderVersion.value && !ctx.hideLoaderVersion.value"
						class="flex items-center justify-between"
					>
						<span class="text-secondary">Loader version</span>
						<span class="font-medium text-contrast">
							{{ ctx.selectedLoaderVersion.value }}
						</span>
					</div>
				</template>
				<div class="flex items-center justify-between">
					<span class="text-secondary">Gamemode</span>
					<span class="font-medium text-contrast">
						{{ capitalize(ctx.gamemode.value) }}
					</span>
				</div>
				<div class="flex items-center justify-between">
					<span class="text-secondary">Difficulty</span>
					<span class="font-medium text-contrast">
						{{ capitalize(ctx.difficulty.value) }}
					</span>
				</div>
			</div>
		</div>

		<div
			v-if="!ctx.isInitialSetup"
			class="flex w-full flex-col gap-2 rounded-2xl bg-surface-2 p-4 border-surface-4 border border-solid"
		>
			<div class="flex w-full flex-row items-center justify-between gap-4">
				<div class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">Erase all data</span>
					<span class="text-sm text-secondary">
						Removes all data on your server, including your worlds, mods, and configuration files,
						then reinstalls it with the selected version.
					</span>
				</div>
				<Toggle id="hard-reset" v-model="ctx.hardReset.value" class="shrink-0" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { FileIcon } from '@modrinth/assets'
import { computed } from 'vue'

import Avatar from '../../../base/Avatar.vue'
import Toggle from '../../../base/Toggle.vue'
import { injectCreationFlowContext } from '../creation-flow-context'
import { capitalize, formatLoaderLabel } from '../shared'

const ctx = injectCreationFlowContext()

const formatLoader = formatLoaderLabel

const selectedVersionLabel = computed(() => {
	const versionId = ctx.modpackSelection.value?.versionId
	if (!versionId) return null
	return ctx.modpackVersionOptions.value.find((o) => o.value === versionId)?.label ?? null
})
</script>
