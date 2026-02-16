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
export type GeneratorSettingsMode = 'default' | 'flat' | 'custom'

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

	// Configuration
	availableLoaders: string[]
	showSnapshotToggle: boolean
	disableClose: boolean
	isInitialSetup: boolean

	// Initial values (for pre-selection when re-opening modal)
	initialWorldType: WorldType | null
	initialLoader: string | null
	initialGameVersion: string | null

	// State
	worldType: Ref<WorldType | null>
	worldName: Ref<string>
	gamemode: Ref<Gamemode>
	difficulty: Ref<Difficulty>
	worldSeed: Ref<string>
	worldTypeOption: Ref<string>
	generateStructures: Ref<boolean>
	generatorSettingsMode: Ref<GeneratorSettingsMode>
	generatorSettingsCustom: Ref<string>

	// Loader/version state (custom setup)
	selectedLoader: Ref<string | null>
	selectedGameVersion: Ref<string | null>
	loaderVersionType: Ref<LoaderVersionType>
	selectedLoaderVersion: Ref<string | null>
	hideLoaderFields: ComputedRef<boolean>
	showSnapshots: Ref<boolean>

	// Modpack state
	modpackSelection: Ref<ModpackSelection | null>
	modpackFile: Ref<File | null>

	// Modpack search state (persisted across stage navigation)
	modpackSearchProjectId: Ref<string | undefined>
	modpackSearchVersionId: Ref<string | undefined>
	modpackSearchOptions: Ref<ComboboxOption<string>[]>
	modpackVersionOptions: Ref<ComboboxOption<string>[]>
	modpackSearchHits: Ref<Record<string, ModpackSearchHit>>

	// Confirm stage
	hardReset: Ref<boolean>

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

export interface CreationFlowOptions {
	availableLoaders?: string[]
	showSnapshotToggle?: boolean
	disableClose?: boolean
	isInitialSetup?: boolean
	initialWorldType?: WorldType
	initialLoader?: string
	initialGameVersion?: string
}

export function createCreationFlowContext(
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>,
	flowType: FlowType,
	emit: {
		browseModpacks: () => void
		create: (config: CreationFlowContextValue) => void
	},
	options: CreationFlowOptions = {},
): CreationFlowContextValue {
	const availableLoaders = options.availableLoaders ?? ['fabric', 'neoforge', 'forge', 'quilt']
	const showSnapshotToggle = options.showSnapshotToggle ?? false
	const disableClose = options.disableClose ?? false
	const isInitialSetup = options.isInitialSetup ?? false
	const initialWorldType = options.initialWorldType ?? null
	const initialLoader = options.initialLoader ?? null
	const initialGameVersion = options.initialGameVersion ?? null

	const worldType = ref<WorldType | null>(null)
	const worldName = ref('')
	const gamemode = ref<Gamemode>('survival')
	const difficulty = ref<Difficulty>('normal')
	const worldSeed = ref('')
	const worldTypeOption = ref('minecraft:normal')
	const generateStructures = ref(true)
	const generatorSettingsMode = ref<GeneratorSettingsMode>('default')
	const generatorSettingsCustom = ref('')

	const selectedLoader = ref<string | null>(null)
	const selectedGameVersion = ref<string | null>(null)
	const loaderVersionType = ref<LoaderVersionType>('stable')
	const selectedLoaderVersion = ref<string | null>(null)
	const showSnapshots = ref(false)

	const modpackSelection = ref<ModpackSelection | null>(null)
	const modpackFile = ref<File | null>(null)

	// Modpack search state (persisted across stage navigation)
	const modpackSearchProjectId = ref<string | undefined>()
	const modpackSearchVersionId = ref<string | undefined>()
	const modpackSearchOptions = ref<ComboboxOption<string>[]>([])
	const modpackVersionOptions = ref<ComboboxOption<string>[]>([])
	const modpackSearchHits = ref<Record<string, ModpackSearchHit>>({})

	const hardReset = ref(isInitialSetup)

	const hideLoaderFields = computed(
		() => worldType.value === 'vanilla' || selectedLoader.value === 'vanilla',
	)

	function reset() {
		worldType.value = null
		worldName.value = ''
		gamemode.value = 'survival'
		difficulty.value = 'normal'
		worldSeed.value = ''
		worldTypeOption.value = 'minecraft:normal'
		generateStructures.value = true
		generatorSettingsMode.value = 'default'
		generatorSettingsCustom.value = ''
		selectedLoader.value = null
		selectedGameVersion.value = null
		loaderVersionType.value = 'stable'
		selectedLoaderVersion.value = null
		showSnapshots.value = false
		modpackSelection.value = null
		modpackFile.value = null
		modpackSearchProjectId.value = undefined
		modpackSearchVersionId.value = undefined
		modpackSearchOptions.value = []
		modpackVersionOptions.value = []
		modpackSearchHits.value = {}
		hardReset.value = isInitialSetup
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

	const resolvedStageConfigs = disableClose
		? stageConfigs.map((stage) => ({ ...stage, disableClose: true }))
		: stageConfigs

	const contextValue: CreationFlowContextValue = {
		flowType,
		availableLoaders,
		showSnapshotToggle,
		disableClose,
		isInitialSetup,
		initialWorldType,
		initialLoader,
		initialGameVersion,
		worldType,
		worldName,
		gamemode,
		difficulty,
		worldSeed,
		worldTypeOption,
		generateStructures,
		generatorSettingsMode,
		generatorSettingsCustom,
		selectedLoader,
		selectedGameVersion,
		loaderVersionType,
		selectedLoaderVersion,
		hideLoaderFields,
		showSnapshots,
		modpackSelection,
		modpackFile,
		modpackSearchProjectId,
		modpackSearchVersionId,
		modpackSearchOptions,
		modpackVersionOptions,
		modpackSearchHits,
		hardReset,
		modal,
		stageConfigs: resolvedStageConfigs,
		reset,
		setWorldType,
		finish,
	}

	return contextValue
}
