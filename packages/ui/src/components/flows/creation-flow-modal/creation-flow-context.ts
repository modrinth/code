import { computed, type ComputedRef, type Ref, ref, type ShallowRef } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import { createContext } from '../../../providers'
import type { MultiStageModal, StageConfigInput } from '../../base'
import type { ComboboxOption } from '../../base/Combobox.vue'
import { stageConfigs } from './stages'

export type FlowType = 'world' | 'server-onboarding' | 'instance'
export type WorldType = 'modpack' | 'custom' | 'vanilla'
export type Gamemode = 'survival' | 'creative' | 'hardcore'
export type Difficulty = 'peaceful' | 'easy' | 'normal' | 'hard'
export type LoaderVersionType = 'stable' | 'latest' | 'other'

export interface ModpackSelection {
	projectId: string
	versionId: string
	name: string
	iconUrl?: string
}

export interface ModpackSearchHit {
	title: string
	iconUrl?: string
	latestVersion?: string
}

export const flowTypeHeadings: Record<FlowType, string> = {
	world: 'Create world',
	'server-onboarding': 'Set up server',
	instance: 'Create instance',
}

export interface CreationFlowContextValue {
	// Flow
	flowType: FlowType

	// State
	worldType: Ref<WorldType | null>
	worldName: Ref<string>
	gamemode: Ref<Gamemode>
	difficulty: Ref<Difficulty>
	worldSeed: Ref<string>
	worldTypeOption: Ref<string>

	// Loader/version state (custom setup)
	selectedLoader: Ref<string | null>
	selectedGameVersion: Ref<string | null>
	loaderVersionType: Ref<LoaderVersionType>
	selectedLoaderVersion: Ref<string | null>
	hideLoaderFields: ComputedRef<boolean>

	// Modpack state
	modpackSelection: Ref<ModpackSelection | null>
	modpackFile: Ref<File | null>

	// Modpack search state (persisted across stage navigation)
	modpackSearchProjectId: Ref<string | undefined>
	modpackSearchVersionId: Ref<string | undefined>
	modpackSearchOptions: Ref<ComboboxOption<string>[]>
	modpackVersionOptions: Ref<ComboboxOption<string>[]>
	modpackSearchHits: Ref<Record<string, ModpackSearchHit>>

	// Modal
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	stageConfigs: StageConfigInput<CreationFlowContextValue>[]

	// Methods
	reset: () => void
	setWorldType: (type: WorldType) => void
	finish: () => void
}

export const [injectCreationFlowContext, provideCreationFlowContext] =
	createContext<CreationFlowContextValue>('CreationFlowModal')

export function createCreationFlowContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
	flowType: FlowType,
	emit: {
		browseModpacks: () => void
		create: (config: CreationFlowContextValue) => void
	},
): CreationFlowContextValue {
	const worldType = ref<WorldType | null>(null)
	const worldName = ref('')
	const gamemode = ref<Gamemode>('survival')
	const difficulty = ref<Difficulty>('normal')
	const worldSeed = ref('')
	const worldTypeOption = ref('minecraft:normal')

	const selectedLoader = ref<string | null>(null)
	const selectedGameVersion = ref<string | null>(null)
	const loaderVersionType = ref<LoaderVersionType>('stable')
	const selectedLoaderVersion = ref<string | null>(null)

	const modpackSelection = ref<ModpackSelection | null>(null)
	const modpackFile = ref<File | null>(null)

	// Modpack search state (persisted across stage navigation)
	const modpackSearchProjectId = ref<string | undefined>()
	const modpackSearchVersionId = ref<string | undefined>()
	const modpackSearchOptions = ref<ComboboxOption<string>[]>([])
	const modpackVersionOptions = ref<ComboboxOption<string>[]>([])
	const modpackSearchHits = ref<Record<string, ModpackSearchHit>>({})

	const hideLoaderFields = computed(() => worldType.value === 'vanilla')

	function reset() {
		worldType.value = null
		worldName.value = ''
		gamemode.value = 'survival'
		difficulty.value = 'normal'
		worldSeed.value = ''
		worldTypeOption.value = 'minecraft:normal'
		selectedLoader.value = null
		selectedGameVersion.value = null
		loaderVersionType.value = 'stable'
		selectedLoaderVersion.value = null
		modpackSelection.value = null
		modpackFile.value = null
		modpackSearchProjectId.value = undefined
		modpackSearchVersionId.value = undefined
		modpackSearchOptions.value = []
		modpackVersionOptions.value = []
		modpackSearchHits.value = {}
	}

	function setWorldType(type: WorldType) {
		worldType.value = type
		if (type === 'modpack') {
			modal.value?.setStage('modpack')
		} else {
			// both custom and vanilla go to custom-setup
			// vanilla just hides loader fields via hideLoaderFields computed
			modal.value?.setStage('custom-setup')
		}
	}

	function finish() {
		emit.create(contextValue)
	}

	const contextValue: CreationFlowContextValue = {
		flowType,
		worldType,
		worldName,
		gamemode,
		difficulty,
		worldSeed,
		worldTypeOption,
		selectedLoader,
		selectedGameVersion,
		loaderVersionType,
		selectedLoaderVersion,
		hideLoaderFields,
		modpackSelection,
		modpackFile,
		modpackSearchProjectId,
		modpackSearchVersionId,
		modpackSearchOptions,
		modpackVersionOptions,
		modpackSearchHits,
		modal,
		stageConfigs,
		reset,
		setWorldType,
		finish,
	}

	return contextValue
}
