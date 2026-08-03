import { reactive, ref } from 'vue'

export const DEFAULT_FEATURE_FLAGS = {
	project_background: false,
	page_path: false,
	server_project_qa: false,
	show_version_environment_column: false,
	server_ram_as_bytes_always_on: false,
	always_show_app_controls: false,
	show_sync_instances_update_modal: false,
	pride_fundraiser: true,
	i18n_debug: false,
	ServerPlayFrontend: false,
	localhost_sign_in: false,
}

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS
type FeatureFlags = Record<FeatureFlag, boolean>

const syncBehaviorAcrossDevices = ref(false)
const syncFeaturesAcrossDevices = ref(false)
const featureFlags = reactive<FeatureFlags>({ ...DEFAULT_FEATURE_FLAGS })

function setBehaviorSyncAcrossDevices(enabled: boolean): void {
	syncBehaviorAcrossDevices.value = enabled
}

function setFeaturesSyncAcrossDevices(enabled: boolean): void {
	syncFeaturesAcrossDevices.value = enabled
}

function getFeatureFlag(key: FeatureFlag): boolean {
	return featureFlags[key] ?? DEFAULT_FEATURE_FLAGS[key]
}

const appSettings = reactive({
	showJumpIn: true,
	alwaysShowCopyDetails: false,
	hideInstalledModpacks: false,
	advancedFiltersCollapsed: true,
	dismissedPhotosensitivityFilterWarning: false,
	friendsActiveCollapsed: false,
	friendsOnlineCollapsed: false,
	friendsOfflineCollapsed: true,
	friendsPendingCollapsed: true,
	refocusOnGameClose: false,
	compactInstanceCards: false,
	showPlayTime: true,
	warnOnUnknownModpacks: true,
	skipNonEssentialWarnings: false,
	syncBehaviorAcrossDevices,
	syncFeaturesAcrossDevices,
	hideNametagSkinsPage: false,
	toggleSidebar: false,
	showFilesTabInInstances: true,
	showWorldsTabInInstances: true,
	showScreenshotsTabInInstances: false,
	showSkinSelectorInSidebar: true,
	nativeDecorations: false,
	devMode: false,
	featureFlags,
	setBehaviorSyncAcrossDevices,
	setFeaturesSyncAcrossDevices,
	getFeatureFlag,
})

export function useAppSettings() {
	return appSettings
}
