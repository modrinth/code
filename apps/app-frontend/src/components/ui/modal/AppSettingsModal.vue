<script setup lang="ts">
import { NewModal } from '@modrinth/ui'
import { ShieldIcon, SettingsIcon, GaugeIcon, PaintBrushIcon, GameIcon, CoffeeIcon } from '@modrinth/assets'
import { ref } from 'vue'
import { useVIntl, defineMessage } from '@vintl/vintl'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import AppSettings from '@/components/ui/settings/AppSettings.vue'

const modal = ref()

function show() {
  modal.value.show()
}

const { formatMessage } = useVIntl();

const selectedTab = ref(0)

const tabs = ref([
  {
    name: defineMessage({
      id: "app.settings.tabs.appearance",
      defaultMessage: "Appearance",
    }),
    icon: PaintBrushIcon,
    content: AppearanceSettings,
  },
  {
    name: defineMessage({
      id: "app.settings.tabs.privacy",
      defaultMessage: "Privacy",
    }),
    icon: ShieldIcon,
    content: AppSettings,
  },
  {
    name: defineMessage({
      id: "app.settings.tabs.java-versions",
      defaultMessage: "Java versions",
    }),
    icon: CoffeeIcon,
    content: AppSettings,
  },
  {
    name: defineMessage({
      id: "app.settings.tabs.default-instance-options",
      defaultMessage: "Default instance options",
    }),
    icon: GameIcon,
    content: AppSettings,
  },
  {
    name: defineMessage({
      id: "app.settings.tabs.resource-management",
      defaultMessage: "Resource management",
    }),
    icon: GaugeIcon,
    content: AppSettings,
  }
])

defineExpose({ show })
</script>/
<template>
  <NewModal ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
        <SettingsIcon /> Settings
      </span>
    </template>
    <div class="grid grid-cols-[auto_1fr] gap-4">
      <div class="flex flex-col gap-1 border-solid pr-4 border-0 border-r-[1px] border-button-bg">
        <button v-for="(tab, index) in tabs" :key="index" :class="`flex gap-2 items-center text-left rounded-xl px-4 py-2 border-none text-nowrap font-semibold cursor-pointer active:scale-[0.97] transition-transform ${selectedTab === index ? 'bg-highlight text-brand' : 'bg-transparent text-button-text'}`" @click="() => selectedTab = index">
          <component :is="tab.icon" class="w-4 h-4" />
          <span>{{ formatMessage(tab.name) }}</span>
        </button>
      </div>
      <div class="w-[600px] h-[500px]">
        <component :is="tabs[selectedTab].content"/>
      </div>
    </div>
  </NewModal>
</template>
