<script setup lang="ts">
import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	HeartHandshakeIcon,
	LanguagesIcon,
	ModrinthIcon,
	PaintbrushIcon,
	Settings2Icon,
	ShieldIcon,
	ToggleRightIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	ProgressBar,
	TabbedModal,
	useVIntl,
} from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, ref, watch } from 'vue'

import PrivacySettings from '@/components/ui/settings/account/PrivacySettings.vue'
import SocialSettings from '@/components/ui/settings/account/SocialSettings.vue'
import AppearanceSettings from '@/components/ui/settings/display/AppearanceSettings.vue'
import BehaviorSettings from '@/components/ui/settings/display/BehaviorSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/display/FeatureFlagSettings.vue'
import LanguageSettings from '@/components/ui/settings/display/LanguageSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/instances/DefaultInstanceSettings.vue'
import JavaSettings from '@/components/ui/settings/instances/JavaSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/instances/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

// TODO: Apply COMPONENT_STRUCTURE.md here and extract out common setting option components
const themeStore = useTheming()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabCategories = defineMessages({
	display: {
		id: 'settings.sidebar.label.display',
		defaultMessage: 'Display',
	},
	account: {
		id: 'settings.sidebar.label.account',
		defaultMessage: 'Account',
	},
	instances: {
		id: 'app.settings.sidebar.label.instances',
		defaultMessage: 'Instances',
	},
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.display,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.display,
		icon: Settings2Icon,
		content: BehaviorSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		category: tabCategories.display,
		icon: LanguagesIcon,
		content: LanguageSettings,
		badge: commonMessages.beta,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
	{
		name: commonSettingsMessages.social,
		category: tabCategories.account,
		icon: HeartHandshakeIcon,
		content: SocialSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		category: tabCategories.account,
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default instance options',
		}),
		category: tabCategories.instances,
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		category: tabCategories.instances,
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		category: tabCategories.instances,
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
]

const availableTabs = computed(() => tabs.filter((tab) => !tab.developerOnly || themeStore.devMode))

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)

function show() {
	modal.value?.show()
}

defineExpose({ show })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		const selectedTab = modal.value ? availableTabs.value[modal.value.selectedTab] : undefined

		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (modal.value) {
			const selectedTabIndex = selectedTab ? availableTabs.value.indexOf(selectedTab) : -1
			modal.value.setTab(selectedTabIndex >= 0 ? selectedTabIndex : 0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	appVersion: {
		id: 'app.settings.app-version',
		defaultMessage: 'Modrinth App {version}',
	},
	macos: {
		id: 'app.settings.operating-system.macos',
		defaultMessage: 'macOS',
	},
	developerModeButtonLabel: {
		id: 'app.settings.developer-mode-button.label',
		defaultMessage: 'Toggle developer mode',
	},
})
</script>
<template>
	<TabbedModal ref="modal" :tabs="availableTabs" :width="'min(928px, calc(95vw - 10rem))'">
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(commonMessages.settingsLabel) }}
			</span>
		</template>
		<template #footer>
			<div class="mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
					{{ formatMessage(developerModeEnabled) }}
				</p>
				<div class="flex items-center gap-3">
					<button
						:aria-label="formatMessage(messages.developerModeButtonLabel)"
						class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
						:class="{
							'text-brand': themeStore.devMode,
							'text-secondary': !themeStore.devMode,
						}"
						@click="devModeCount"
					>
						<ModrinthIcon aria-hidden="true" class="w-6 h-6" />
					</button>
					<div class="max-w-[200px]">
						<p class="m-0">
							{{ formatMessage(messages.appVersion, { version }) }}
						</p>
						<p class="m-0">
							<span v-if="osPlatform === 'macos'">{{ formatMessage(messages.macos) }}</span>
							<span v-else class="capitalize">{{ osPlatform }}</span>
							{{ osVersion }}
						</p>
					</div>
				</div>
			</div>
		</template>
	</TabbedModal>
</template>
