<script setup lang="ts">
import { NewModal } from '@modrinth/ui'
import {
  ShieldIcon,
  SettingsIcon,
  GaugeIcon,
  PaintBrushIcon,
  GameIcon,
  CoffeeIcon,
} from '@modrinth/assets'
import { ref } from 'vue'
import { useVIntl, defineMessage } from '@vintl/vintl'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import { getVersion } from '@tauri-apps/api/app'
import { version as getOsVersion, platform as getOsPlatform } from '@tauri-apps/plugin-os'

const modal = ref()

function show() {
  modal.value.show()
}

const { formatMessage } = useVIntl()

const selectedTab = ref(0)

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
      id: 'app.settings.tabs.java-versions',
      defaultMessage: 'Java versions',
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
]

defineExpose({ show })

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
</script>
/
<template>
  <NewModal ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
        <SettingsIcon /> Settings
      </span>
    </template>
    <div class="grid grid-cols-[auto_1fr] gap-4">
      <div class="flex flex-col gap-1 border-solid pr-4 border-0 border-r-[1px] border-button-bg">
        <button
          v-for="(tab, index) in tabs"
          :key="index"
          :class="`flex gap-2 items-center text-left rounded-xl px-4 py-2 border-none text-nowrap font-semibold cursor-pointer active:scale-[0.97] transition-transform ${selectedTab === index ? 'bg-highlight text-brand' : 'bg-transparent text-button-text'}`"
          @click="() => (selectedTab = index)"
        >
          <component :is="tab.icon" class="w-4 h-4" />
          <span>{{ formatMessage(tab.name) }}</span>
        </button>

        <div class="mt-auto text-secondary text-sm">
          <p class="m-0">Modrinth App {{ version }}</p>
          <p class="m-0">
            <span v-if="osPlatform === 'macos'">MacOS</span>
            <span v-else class="capitalize">{{ osPlatform }}</span>
            {{ osVersion }}
          </p>
        </div>
      </div>
      <div class="w-[600px] h-[500px] overflow-y-auto">
        <component :is="tabs[selectedTab].content" />
      </div>
    </div>
  </NewModal>
</template>
