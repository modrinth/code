<template>
	<div class="space-y-6">
		<div
			v-if="ctx.flowType !== 'server-onboarding' && ctx.flowType !== 'reset-server'"
			class="flex flex-col gap-2"
		>
			<span class="font-semibold text-contrast">{{ formatMessage(messages.worldNameLabel) }}</span>
			<StyledInput
				v-model="worldName"
				:placeholder="formatMessage(messages.worldNamePlaceholder)"
			/>
		</div>

		<div v-if="ctx.setupType.value === 'vanilla'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{
				formatMessage(commonMessages.gameVersionLabel)
			}}</span>
			<Combobox
				v-model="selectedGameVersion"
				:options="gameVersionOptions"
				searchable
				sync-with-selection
				:placeholder="formatMessage(messages.gameVersionPlaceholder)"
			>
				<template v-if="ctx.showSnapshotToggle" #dropdown-footer>
					<button
						class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
						@mousedown.prevent
						@click="ctx.showSnapshots.value = !ctx.showSnapshots.value"
					>
						<EyeOffIcon v-if="ctx.showSnapshots.value" class="size-4" />
						<EyeIcon v-else class="size-4" />
						{{
							ctx.showSnapshots.value
								? formatMessage(commonMessages.hideSnapshotsButton)
								: formatMessage(commonMessages.showAllVersionsButton)
						}}
					</button>
				</template>
			</Combobox>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.gamemodeLabel) }}</span>
			<Chips v-model="gamemode" :items="gamemodeItems" :format-label="formatGamemodeLabel" />
		</div>

		<div v-if="gamemode !== 'hardcore'" class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.difficultyLabel) }}</span>
			<Chips v-model="difficulty" :items="difficultyItems" :format-label="formatDifficultyLabel" />
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">{{ formatMessage(messages.worldTypeLabel) }}</span>
			<Combobox
				v-model="worldTypeOption"
				:options="worldTypeOptions"
				:placeholder="formatMessage(messages.worldTypePlaceholder)"
			/>
		</div>

		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				<IntlFormatted :message-id="messages.worldSeedLabelWithOptional">
					<template #optional="{ children }">
						<span class="text-secondary font-normal">
							<component :is="() => children" />
						</span>
					</template>
				</IntlFormatted>
			</span>
			<StyledInput
				v-model="worldSeed"
				:placeholder="formatMessage(messages.worldSeedPlaceholder)"
			/>
			<span class="text-sm text-secondary">{{ formatMessage(messages.worldSeedDescription) }}</span>
		</div>

		<div class="h-px w-full bg-surface-5" />

		<Accordion overflow-visible button-class="w-full bg-transparent m-0 p-0 border-none">
			<template #title>
				<SettingsIcon class="size-4 shrink-0 text-primary" />
				<span class="font-semibold text-contrast text-lg">{{
					formatMessage(messages.additionalSettingsTitle)
				}}</span>
			</template>
			<div class="flex flex-col gap-4 pt-4">
				<div class="flex w-full flex-row items-center justify-between gap-4">
					<div class="flex flex-col gap-1">
						<span class="font-semibold text-contrast">{{
							formatMessage(messages.generateStructuresLabel)
						}}</span>
						<span class="text-sm text-secondary">
							{{ formatMessage(messages.generateStructuresDescription) }}
						</span>
					</div>
					<Toggle v-model="generateStructures" small class="shrink-0" />
				</div>

				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">{{
						formatMessage(messages.generatorSettingsLabel)
					}}</span>
					<Combobox
						v-model="generatorSettingsMode"
						:options="generatorSettingsOptions"
						:placeholder="formatMessage(messages.generatorSettingsPlaceholder)"
					/>
					<StyledInput
						v-if="generatorSettingsMode === 'custom'"
						v-model="generatorSettingsCustom"
						multiline
						:rows="4"
						:placeholder="formatMessage(messages.generatorSettingsJsonPlaceholder)"
						input-class="font-mono"
					/>
					<span class="text-sm text-secondary">
						{{ formatMessage(messages.generatorSettingsDescription) }}
					</span>
				</div>
			</div>
		</Accordion>

		<InlineBackupCreator
			v-if="ctx.flowType === 'reset-server'"
			ref="backupCreator"
			:backup-name="formatMessage(messages.beforeResetServerBackupName)"
			hide-shift-click-hint
			@update:buttons-disabled="ctx.isBackingUp.value = $event"
		/>
	</div>
</template>

<script setup lang="ts">
import { EyeIcon, EyeOffIcon, SettingsIcon } from '@modrinth/assets'
import { commonMessages, defineMessages, IntlFormatted, useVIntl } from '@modrinth/ui'
import { computed, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'

import InlineBackupCreator from '../../../../layouts/shared/content-tab/components/modals/InlineBackupCreator.vue'
import { injectTags } from '../../../../providers'
import Accordion from '../../../base/Accordion.vue'
import Chips from '../../../base/Chips.vue'
import Combobox, { type ComboboxOption } from '../../../base/Combobox.vue'
import StyledInput from '../../../base/StyledInput.vue'
import Toggle from '../../../base/Toggle.vue'
import type { Difficulty, Gamemode, GeneratorSettingsMode } from '../creation-flow-context'
import { injectCreationFlowContext } from '../creation-flow-context'

const debug = useDebugLogger('FinalConfigStage')
const ctx = injectCreationFlowContext()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	worldNameLabel: {
		id: 'creation-flow.modal.final-config.world-name.label',
		defaultMessage: 'World name',
	},
	worldNamePlaceholder: {
		id: 'creation-flow.modal.final-config.world-name.placeholder',
		defaultMessage: 'Enter world name',
	},
	gameVersionPlaceholder: {
		id: 'creation-flow.modal.final-config.game-version.placeholder',
		defaultMessage: 'Select game version',
	},
	gamemodeLabel: {
		id: 'creation-flow.modal.final-config.gamemode.label',
		defaultMessage: 'Gamemode',
	},
	gamemodeSurvival: {
		id: 'creation-flow.modal.final-config.gamemode.survival',
		defaultMessage: 'Survival',
	},
	gamemodeCreative: {
		id: 'creation-flow.modal.final-config.gamemode.creative',
		defaultMessage: 'Creative',
	},
	gamemodeHardcore: {
		id: 'creation-flow.modal.final-config.gamemode.hardcore',
		defaultMessage: 'Hardcore',
	},
	difficultyLabel: {
		id: 'creation-flow.modal.final-config.difficulty.label',
		defaultMessage: 'Difficulty',
	},
	difficultyPeaceful: {
		id: 'creation-flow.modal.final-config.difficulty.peaceful',
		defaultMessage: 'Peaceful',
	},
	difficultyEasy: {
		id: 'creation-flow.modal.final-config.difficulty.easy',
		defaultMessage: 'Easy',
	},
	difficultyNormal: {
		id: 'creation-flow.modal.final-config.difficulty.normal',
		defaultMessage: 'Normal',
	},
	difficultyHard: {
		id: 'creation-flow.modal.final-config.difficulty.hard',
		defaultMessage: 'Hard',
	},
	worldTypeLabel: {
		id: 'creation-flow.modal.final-config.world-type.label',
		defaultMessage: 'World type',
	},
	worldTypePlaceholder: {
		id: 'creation-flow.modal.final-config.world-type.placeholder',
		defaultMessage: 'Select world type',
	},
	worldTypeDefault: {
		id: 'creation-flow.modal.final-config.world-type.default',
		defaultMessage: 'Default',
	},
	worldTypeSuperflat: {
		id: 'creation-flow.modal.final-config.world-type.superflat',
		defaultMessage: 'Superflat',
	},
	worldTypeLargeBiomes: {
		id: 'creation-flow.modal.final-config.world-type.large-biomes',
		defaultMessage: 'Large Biomes',
	},
	worldTypeAmplified: {
		id: 'creation-flow.modal.final-config.world-type.amplified',
		defaultMessage: 'Amplified',
	},
	worldTypeSingleBiome: {
		id: 'creation-flow.modal.final-config.world-type.single-biome',
		defaultMessage: 'Single Biome',
	},
	worldSeedLabelWithOptional: {
		id: 'creation-flow.modal.final-config.world-seed.label-with-optional',
		defaultMessage: 'World seed <optional>(Optional)</optional>',
	},
	worldSeedPlaceholder: {
		id: 'creation-flow.modal.final-config.world-seed.placeholder',
		defaultMessage: 'Enter world seed',
	},
	worldSeedDescription: {
		id: 'creation-flow.modal.final-config.world-seed.description',
		defaultMessage: 'Leave blank for a random seed.',
	},
	additionalSettingsTitle: {
		id: 'creation-flow.modal.final-config.additional-settings.title',
		defaultMessage: 'Additional settings',
	},
	generateStructuresLabel: {
		id: 'creation-flow.modal.final-config.generate-structures.label',
		defaultMessage: 'Generate structures',
	},
	generateStructuresDescription: {
		id: 'creation-flow.modal.final-config.generate-structures.description',
		defaultMessage:
			'Controls whether villages, strongholds, and other structures generate in new chunks.',
	},
	generatorSettingsLabel: {
		id: 'creation-flow.modal.final-config.generator-settings.label',
		defaultMessage: 'Generator settings',
	},
	generatorSettingsPlaceholder: {
		id: 'creation-flow.modal.final-config.generator-settings.placeholder',
		defaultMessage: 'Select generator settings',
	},
	generatorSettingsDefault: {
		id: 'creation-flow.modal.final-config.generator-settings.default',
		defaultMessage: 'Default',
	},
	generatorSettingsFlat: {
		id: 'creation-flow.modal.final-config.generator-settings.flat',
		defaultMessage: 'Flat',
	},
	generatorSettingsCustom: {
		id: 'creation-flow.modal.final-config.generator-settings.custom',
		defaultMessage: 'Custom',
	},
	generatorSettingsJsonPlaceholder: {
		id: 'creation-flow.modal.final-config.generator-settings-json.placeholder',
		defaultMessage: 'Enter generator settings JSON',
	},
	generatorSettingsDescription: {
		id: 'creation-flow.modal.final-config.generator-settings.description',
		defaultMessage: 'Used for advanced world customization such as custom Superflat layers.',
	},
	beforeResetServerBackupName: {
		id: 'creation-flow.modal.final-config.backup.before-reset-server.name',
		defaultMessage: 'Before reset server',
	},
})

const backupCreator = ref<InstanceType<typeof InlineBackupCreator> | null>(null)
watch(backupCreator, (creator) => {
	ctx.cancelBackup.value = creator?.cancelBackup ?? null
})
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

debug(
	'mounted, setupType:',
	ctx.setupType.value,
	'loader:',
	ctx.selectedLoader.value,
	'gameVersion:',
	ctx.selectedGameVersion.value,
	'loaderVersion:',
	ctx.selectedLoaderVersion.value,
)

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

function formatGamemodeLabel(mode: Gamemode): string {
	switch (mode) {
		case 'survival':
			return formatMessage(messages.gamemodeSurvival)
		case 'creative':
			return formatMessage(messages.gamemodeCreative)
		case 'hardcore':
			return formatMessage(messages.gamemodeHardcore)
	}
}

function formatDifficultyLabel(value: Difficulty): string {
	switch (value) {
		case 'peaceful':
			return formatMessage(messages.difficultyPeaceful)
		case 'easy':
			return formatMessage(messages.difficultyEasy)
		case 'normal':
			return formatMessage(messages.difficultyNormal)
		case 'hard':
			return formatMessage(messages.difficultyHard)
	}
}

const worldTypeOptions = computed<ComboboxOption<string>[]>(() => [
	{ value: 'minecraft:normal', label: formatMessage(messages.worldTypeDefault) },
	{ value: 'minecraft:flat', label: formatMessage(messages.worldTypeSuperflat) },
	{ value: 'minecraft:large_biomes', label: formatMessage(messages.worldTypeLargeBiomes) },
	{ value: 'minecraft:amplified', label: formatMessage(messages.worldTypeAmplified) },
	{ value: 'minecraft:single_biome_surface', label: formatMessage(messages.worldTypeSingleBiome) },
])

const generatorSettingsOptions = computed<ComboboxOption<GeneratorSettingsMode>[]>(() => [
	{ value: 'default', label: formatMessage(messages.generatorSettingsDefault) },
	{ value: 'flat', label: formatMessage(messages.generatorSettingsFlat) },
	{ value: 'custom', label: formatMessage(messages.generatorSettingsCustom) },
])
</script>
