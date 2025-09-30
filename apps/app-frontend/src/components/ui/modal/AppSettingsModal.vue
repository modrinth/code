<script setup lang="ts">
import {
	CoffeeIcon,
	GameIcon,
	GaugeIcon,
	ModrinthIcon,
	PaintbrushIcon,
	ReportIcon,
	SettingsIcon,
	ShieldIcon,
} from '@modrinth/assets'
import { ProgressBar, TabbedModal } from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { defineMessage, defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

const themeStore = useTheming()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default instance options',
		}),
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.feature-flags',
			defaultMessage: 'Feature flags',
		}),
		icon: ReportIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
]

const modal = ref()

function show() {
	modal.value.show()
}

const isOpen = computed(() => modal.value?.isOpen)

defineExpose({ show, isOpen })

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
		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (!themeStore.devMode && tabs[modal.value.selectedTab].developerOnly) {
			modal.value.setTab(0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
})
</script>
<template>
	<ModalWrapper ref="modal">
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				<SettingsIcon /> Settings
			</span>
		</template>

		<TabbedModal :tabs="tabs.filter((t) => !t.developerOnly || themeStore.devMode)">
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
							class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
							:class="{
								'text-brand': themeStore.devMode,
								'text-secondary': !themeStore.devMode,
							}"
							@click="devModeCount"
						>
							<ModrinthIcon class="w-6 h-6" />
						</button>
						<div>
							<p class="m-0">Modrinth App {{ version }}</p>
							<p class="m-0">
								<span v-if="osPlatform === 'macos'">macOS</span>
								<span v-else class="capitalize">{{ osPlatform }}</span>
								{{ osVersion }}
							</p>
						</div>
					</div>
				</div>
			</template>
		</TabbedModal>
	</ModalWrapper>
</template>
