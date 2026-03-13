import type { Labrinth } from '@modrinth/api-client'
import type { ComputedRef, Ref } from 'vue'

import { createContext } from '#ui/providers/create-context'

import type {
	ContentDiffPreview,
	GameVersionOption,
	InstallationInfoRow,
	InstallationModpackData,
	LoaderVersionEntry,
} from '../types'

export interface InstallationSettingsContext {
	loading: Ref<boolean> | ComputedRef<boolean>
	installationInfo: ComputedRef<InstallationInfoRow[]>
	isLinked: ComputedRef<boolean>
	isBusy: Ref<boolean> | ComputedRef<boolean>

	modpack: Ref<InstallationModpackData | null> | ComputedRef<InstallationModpackData | null>

	currentPlatform: ComputedRef<string>
	currentGameVersion: ComputedRef<string>
	currentLoaderVersion: ComputedRef<string>

	availablePlatforms: string[]

	resolveGameVersions: (loader: string, showSnapshots: boolean) => GameVersionOption[]
	resolveLoaderVersions: (loader: string, gameVersion: string) => LoaderVersionEntry[]
	resolveHasSnapshots: (loader: string) => boolean

	save: (platform: string, gameVersion: string, loaderVersionId: string | null) => Promise<void>
	repair: () => Promise<void>
	reinstallModpack: () => Promise<void>
	unlinkModpack: () => Promise<void>

	getCachedModpackVersions: () => Labrinth.Versions.v2.Version[] | null
	fetchModpackVersions: () => Promise<Labrinth.Versions.v2.Version[]>
	getVersionChangelog: (versionId: string) => Promise<Labrinth.Versions.v2.Version | null>
	onModpackVersionConfirm: (version: Labrinth.Versions.v2.Version) => Promise<void>

	updaterModalProps: ComputedRef<{
		isApp: boolean
		currentVersionId: string
		projectIconUrl?: string
		projectName: string
		currentGameVersion: string
		currentLoader: string
	}>

	isServer: boolean
	isApp: boolean

	/** When false, hides change-version and reinstall buttons in linked state (default: true) */
	showModpackVersionActions?: boolean

	repairing?: Ref<boolean>
	reinstalling?: Ref<boolean>

	afterSave?: () => Promise<void>

	lockPlatform?: boolean
	hideLoaderVersion?: boolean
	previewSave?: (
		platform: string,
		gameVersion: string,
		loaderVersionId: string | null,
		signal?: AbortSignal,
	) => Promise<ContentDiffPreview | null>

	/**
	 * Optional refs for the editing form state. When provided, the composable
	 * uses these instead of creating its own. This lets the wrapper observe
	 * editing state for reactive query dependencies (e.g. paper/purpur builds).
	 */
	editingPlatformRef?: Ref<string>
	editingGameVersionRef?: Ref<string>
}

export const [injectInstallationSettings, provideInstallationSettings] =
	createContext<InstallationSettingsContext>(
		'InstallationSettingsLayout',
		'installationSettingsContext',
	)
