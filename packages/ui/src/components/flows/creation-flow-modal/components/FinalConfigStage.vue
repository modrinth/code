<template>
	<div class="space-y-6">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">World name <span class="text-red">*</span></span>
			<StyledInput v-model="worldName" placeholder="Enter world name" />
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Gamemode</span>
			<Chips v-model="gamemode" :items="gamemodeItems" :format-label="capitalize" />
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">Difficulty</span>
			<Chips v-model="difficulty" :items="difficultyItems" :format-label="capitalize" />
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">World type</span>
			<Combobox
				v-model="worldTypeOption"
				:options="worldTypeOptions"
				placeholder="Select world type"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">World seed</span>
			<StyledInput v-model="worldSeed" placeholder="Enter world seed" />
			<span class="text-sm text-secondary">Leave blank for a random seed.</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import Chips from '../../../base/Chips.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import type { Difficulty, Gamemode } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'

const { worldName, gamemode, difficulty, worldTypeOption, worldSeed } = injectCreationFlowContext()

const gamemodeItems: Gamemode[] = ['survival', 'creative', 'hardcore']
const difficultyItems: Difficulty[] = ['peaceful', 'easy', 'normal', 'hard']

const capitalize = (item: string) => item.charAt(0).toUpperCase() + item.slice(1)

const worldTypeOptions: ComboboxOption<string>[] = [
	{ value: 'minecraft:normal', label: 'Default' },
	{ value: 'minecraft:flat', label: 'Superflat' },
	{ value: 'minecraft:large_biomes', label: 'Large Biomes' },
	{ value: 'minecraft:amplified', label: 'Amplified' },
	{ value: 'minecraft:single_biome_surface', label: 'Single Biome' },
]
</script>
