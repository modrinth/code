import { computed, type ComputedRef, type Ref, ref, type ShallowRef, watch } from 'vue'
import type { ComponentExposed } from 'vue-component-type-helpers'

import { createContext } from '../../../providers'
import type { ImportableLauncher } from '../../../providers/instance-import'
import type { MultiStageModal, StageConfigInput } from '../../base'
import type { ComboboxOption } from '../../base/Combobox.vue'
import { stageConfigs } from './stages'

export type FlowType = 'world' | 'server-onboarding' | 'instance'
export type SetupType = 'modpack' | 'custom' | 'vanilla'
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
	initialSetupType: SetupType | null
	initialLoader: string | null
	initialGameVersion: string | null

	// State
	setupType: Ref<SetupType | null>
	isImportMode: Ref<boolean>
	worldName: Ref<string>
	gamemode: Ref<Gamemode>
	difficulty: Ref<Difficulty>
	worldSeed: Ref<string>
	worldTypeOption: Ref<string>
	generateStructures: Ref<boolean>
	generatorSettingsMode: Ref<GeneratorSettingsMode>
	generatorSettingsCustom: Ref<string>

	// Instance-specific state
	instanceName: Ref<string>
	instanceIcon: Ref<File | null>
	instanceIconUrl: Ref<string | null>

	// Loader/version state (custom setup)
	selectedLoader: Ref<string | null>
	selectedGameVersion: Ref<string | null>
	loaderVersionType: Ref<LoaderVersionType>
	selectedLoaderVersion: Ref<string | null>
	hideLoaderChips: ComputedRef<boolean>
	hideLoaderVersion: ComputedRef<boolean>
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

	// Import state (instance flow only)
	importLaunchers: Ref<ImportableLauncher[]>
	importSelectedInstances: Ref<Record<string, Set<string>>>
	importSearchQuery: Ref<string>

	// Confirm stage
	hardReset: Ref<boolean>

	// Modal
	modal: ShallowRef<ComponentExposed<typeof MultiStageModal> | null>
	stageConfigs: StageConfigInput<CreationFlowContextValue>[]

	// Methods
	reset: () => void
	setSetupType: (type: SetupType) => void
	setImportMode: () => void
	finish: () => void
}

export const [injectCreationFlowContext, provideCreationFlowContext] =
	createContext<CreationFlowContextValue>('CreationFlowModal')

export interface CreationFlowOptions {
	availableLoaders?: string[]
	showSnapshotToggle?: boolean
	disableClose?: boolean
	isInitialSetup?: boolean
	initialSetupType?: SetupType
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
	const initialSetupType = options.initialSetupType ?? null
	const initialLoader = options.initialLoader ?? null
	const initialGameVersion = options.initialGameVersion ?? null

	const setupType = ref<SetupType | null>(null)
	const isImportMode = ref(false)
	const worldName = ref('')
	const gamemode = ref<Gamemode>('survival')
	const difficulty = ref<Difficulty>('normal')
	const worldSeed = ref('')
	const worldTypeOption = ref('minecraft:normal')
	const generateStructures = ref(true)
	const generatorSettingsMode = ref<GeneratorSettingsMode>('default')
	const generatorSettingsCustom = ref('')

	// Instance-specific state
	const instanceName = ref('')
	const instanceIcon = ref<File | null>(null)
	const instanceIconUrl = ref<string | null>(null)

	// Revoke old object URL when icon changes to avoid memory leaks
	watch(instanceIcon, (_newIcon, _oldIcon) => {
		if (instanceIconUrl.value) {
			URL.revokeObjectURL(instanceIconUrl.value)
			instanceIconUrl.value = null
		}
	})

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

	// Import state (instance flow only)
	const importLaunchers = ref<ImportableLauncher[]>([])
	const importSelectedInstances = ref<Record<string, Set<string>>>({})
	const importSearchQuery = ref('')

	const hardReset = ref(isInitialSetup)

	// hideLoaderChips: hides the entire loader chips section (only for vanilla world type in world/server flows)
	const hideLoaderChips = computed(() => setupType.value === 'vanilla')

	// hideLoaderVersion: hides the loader version section (vanilla world type OR vanilla selected as loader chip)
	const hideLoaderVersion = computed(
		() => setupType.value === 'vanilla' || selectedLoader.value === 'vanilla',
	)

	function reset() {
		setupType.value = null
		isImportMode.value = false
		worldName.value = ''
		gamemode.value = 'survival'
		difficulty.value = 'normal'
		worldSeed.value = ''
		worldTypeOption.value = 'minecraft:normal'
		generateStructures.value = true
		generatorSettingsMode.value = 'default'
		generatorSettingsCustom.value = ''

		// Instance-specific
		instanceName.value = ''
		if (instanceIconUrl.value) {
			URL.revokeObjectURL(instanceIconUrl.value)
		}
		instanceIcon.value = null
		instanceIconUrl.value = null

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

		// Import state
		importLaunchers.value = []
		importSelectedInstances.value = {}
		importSearchQuery.value = ''

		hardReset.value = isInitialSetup
	}

	function setSetupType(type: SetupType) {
		isImportMode.value = false
		setupType.value = type
		if (type === 'modpack') {
			modal.value?.setStage('modpack')
		} else {
			// both custom and vanilla go to custom-setup
			// vanilla just hides loader chips via hideLoaderChips computed
			modal.value?.setStage('custom-setup')
		}
	}

	function setImportMode() {
		isImportMode.value = true
		setupType.value = null
		modal.value?.setStage('import-instance')
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
		initialSetupType,
		initialLoader,
		initialGameVersion,
		setupType,
		isImportMode,
		worldName,
		gamemode,
		difficulty,
		worldSeed,
		worldTypeOption,
		generateStructures,
		generatorSettingsMode,
		generatorSettingsCustom,
		instanceName,
		instanceIcon,
		instanceIconUrl,
		selectedLoader,
		selectedGameVersion,
		loaderVersionType,
		selectedLoaderVersion,
		hideLoaderChips,
		hideLoaderVersion,
		showSnapshots,
		modpackSelection,
		modpackFile,
		modpackSearchProjectId,
		modpackSearchVersionId,
		modpackSearchOptions,
		modpackVersionOptions,
		modpackSearchHits,
		importLaunchers,
		importSelectedInstances,
		importSearchQuery,
		hardReset,
		modal,
		stageConfigs: resolvedStageConfigs,
		reset,
		setSetupType,
		setImportMode,
		finish,
	}

	return contextValue
}
