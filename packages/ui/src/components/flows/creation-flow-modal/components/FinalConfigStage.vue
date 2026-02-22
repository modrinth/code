<template>
	<div class="space-y-6">
		<div v-if="ctx.flowType !== 'server-onboarding'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">World name</span>
			<StyledInput v-model="worldName" placeholder="Enter world name" />
		</div>

		<div v-if="ctx.setupType.value === 'vanilla'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Game version</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				search-mode
				sync-with-selection
				placeholder="Select game version"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
						@mousedown.prevent
						@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
					>
						<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
						<EyeIcon v-else class="size-4" />
						{{ ctx.showSnapshots.value ? 'Hide snapshots' : 'Show all versions' }}
					</button>
				</template>
			</Combobox>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Gamemode</span>
			<Chips v-model="gamemode" :items="gamemodeItems" :format-label="capitalize" />
		</div>

		<Collapsible :collapsed="gamemode === 'hardcore'" overflow-visible>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">Difficulty</span>
				<Chips v-model="difficulty" :items="difficultyItems" :format-label="capitalize" />
			</div>
		</Collapsible>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">World type</span>
			<Combobox
				v-model="worldTypeOption"
				:options="worldTypeOptions"
				placeholder="Select world type"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"
				>World seed <span class="text-secondary font-normal">(Optional)</span></span
			>
			<StyledInput v-model="worldSeed" placeholder="Enter world seed" />
			<span class="text-sm text-secondary">Leave blank for a random seed.</span>
		</div>

		<div class="h-px w-full bg-surface-5" />

		<Accordion overflow-visible button-class="w-full bg-transparent m-0 p-0 border-none">
			<template #title>
				<SettingsIcon class="size-4 shrink-0 text-primary" />
				<span class="font-semibold text-contrast text-lg">Additional settings</span>
			</template>
			<div class="flex flex-col gap-4 pt-4">
				<div class="flex w-full flex-row items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<span class="font-semibold text-contrast">Generate structures</span>
						<span class="text-sm text-secondary">
							Controls whether villages, strongholds, and other structures generate in new chunks.
						</span>
					</div>
					<Toggle v-model="generateStructures" small class="shrink-0" />
				</div>

				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">Generator settings</span>
					<Combobox
						v-model="generatorSettingsMode"
						:options="generatorSettingsOptions"
						placeholder="Select generator settings"
					/>
					<StyledInput
						v-if="generatorSettingsMode === 'custom'"
						v-model="generatorSettingsCustom"
						multiline
						:rows="4"
						placeholder="Enter generator settings JSON"
						input-class="font-mono"
					/>
					<span class="text-sm text-secondary">
						Used for advanced world customization such as custom Superflat layers.
					</span>
				</div>
			</div>
		</Accordion>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon, EyeOffIcon, SettingsIcon } from '@modrinth/assets'
import { computed, watch } from 'vue'

import { injectTags } from '../../../../providers'
import Accordion from '../../../base/Accordion.vue'
import Chips from '../../../base/Chips.vue'
import Collapsible from '../../../base/Collapsible.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import Toggle from '../../../base/Toggle.vue'
import type { Difficulty, Gamemode, GeneratorSettingsMode } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'
import { capitalize } from '../shared'

const ctx = injectCreationFlowContext()
const {
	worldName,
	gamemode,
	difficulty,
	worldTypeOption,
	worldSeed,
	generateStructures,
	generatorSettingsMode,
	generatorSettingsCustom,
	selectedGameVersion,
} = ctx

// Game version options for vanilla flow
const tags = injectTags()
const gameVersionOptions = computed<ComboboxOption<string>[]>(() => {
	const versions = ctx.showSnapshots.value
		? tags.gameVersions.value
		: tags.gameVersions.value.filter((v) => v.version_type === 'release')
	return versions.map((v) => ({ value: v.version, label: v.version }))
})

// Auto-select latest game version for vanilla
watch(
	gameVersionOptions,
	(options) => {
		if (!selectedGameVersion.value && options.length > 0) {
			selectedGameVersion.value = options[0].value
		}
	},
	{ immediate: true },
)

// Hardcore locks difficulty to hard
let previousDifficulty: Difficulty = difficulty.value
watch(gamemode, (mode) => {
	if (mode === 'hardcore') {
		previousDifficulty = difficulty.value
		difficulty.value = 'hard'
	} else {
		difficulty.value = previousDifficulty
	}
})

const gamemodeItems: Gamemode[] = ['survival', 'creative', 'hardcore']
const difficultyItems: Difficulty[] = ['peaceful', 'easy', 'normal', 'hard']

const worldTypeOptions: ComboboxOption<string>[] = [
	{ value: 'minecraft:normal', label: 'Default' },
	{ value: 'minecraft:flat', label: 'Superflat' },
	{ value: 'minecraft:large_biomes', label: 'Large Biomes' },
	{ value: 'minecraft:amplified', label: 'Amplified' },
	{ value: 'minecraft:single_biome_surface', label: 'Single Biome' },
]

const generatorSettingsOptions: ComboboxOption<GeneratorSettingsMode>[] = [
	{ value: 'default', label: 'Default' },
	{ value: 'flat', label: 'Flat' },
	{ value: 'custom', label: 'Custom' },
]
</script>
