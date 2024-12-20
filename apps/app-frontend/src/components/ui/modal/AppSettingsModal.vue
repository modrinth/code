<script setup lang="ts">
import {
  ReportIcon,
  ModrinthIcon,
  ShieldIcon,
  SettingsIcon,
  GaugeIcon,
  PaintBrushIcon,
  GameIcon,
  CoffeeIcon,
} from '@modrinth/assets'
import { TabbedModal } from '@modrinth/ui'
import { computed, ref } from 'vue'
import { useVIntl, defineMessage } from '@vintl/vintl'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import { getVersion } from '@tauri-apps/api/app'
import { version as getOsVersion, platform as getOsPlatform } from '@tauri-apps/plugin-os'
import { useTheming } from '@/store/state'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'

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
    icon: PaintBrushIcon,
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

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
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
          <p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
            {{ formatMessage(developerModeEnabled) }}
          </p>
          <div class="flex items-center gap-3">
            <button
              class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
              :class="{ 'text-brand': themeStore.devMode, 'text-secondary': !themeStore.devMode }"
              @click="
                () => {
                  devModeCounter++
                  if (devModeCounter > 5) {
                    themeStore.devMode = !themeStore.devMode
                    devModeCounter = 0

                    if (!themeStore.devMode && tabs[modal.selectedTab].developerOnly) {
                      modal.setTab(0)
                    }
                  }
                }
              "
            >
              <ModrinthIcon class="w-6 h-6" />
            </button>
            <div>
              <p class="m-0">Modrinth App {{ version }}</p>
              <p class="m-0">
                <span v-if="osPlatform === 'macos'">MacOS</span>
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
