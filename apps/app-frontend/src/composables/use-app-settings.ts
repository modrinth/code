import { reactive, ref } from 'vue'

export const DEFAULT_FEATURE_FLAGS = {
	project_background: false,
	page_path: false,
	worlds_in_home: true,
	server_project_qa: false,
	show_version_environment_column: false,
	server_ram_as_bytes_always_on: false,
	always_show_app_controls: false,
	skip_non_essential_warnings: false,
	skip_unknown_pack_warning: false,
	pride_fundraiser: true,
	i18n_debug: false,
	show_instance_play_time: true,
	compact_instance_cards: false,
	advanced_filters_collapsed: true,
	always_show_copy_details: false,
	hide_installed_modpacks: false,
	friends_active_collapsed: false,
	friends_online_collapsed: false,
	friends_offline_collapsed: true,
	friends_pending_collapsed: true,
	dismissed_photosensitivity_filter_warning: false,
}

export type FeatureFlag = keyof typeof DEFAULT_FEATURE_FLAGS
type FeatureFlags = Record<FeatureFlag, boolean>

const syncBehaviorAcrossDevices = ref(false)
const featureFlags = reactive<FeatureFlags>({ ...DEFAULT_FEATURE_FLAGS })

function setBehaviorSyncAcrossDevices(enabled: boolean): void {
	syncBehaviorAcrossDevices.value = enabled
}

function getFeatureFlag(key: FeatureFlag): boolean {
	return featureFlags[key] ?? DEFAULT_FEATURE_FLAGS[key]
}

const appSettings = reactive({
	syncBehaviorAcrossDevices,
	hideNametagSkinsPage: false,
	toggleSidebar: false,
	devMode: false,
	featureFlags,
	setBehaviorSyncAcrossDevices,
	getFeatureFlag,
})

export function useAppSettings() {
	return appSettings
}
