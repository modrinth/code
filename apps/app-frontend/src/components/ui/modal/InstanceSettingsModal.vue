<script setup lang="ts">
import {
  SettingsIcon,
  CoffeeIcon,
  InfoIcon,
  WrenchIcon,
  BoxIcon,
  ArchiveIcon,
  MonitorIcon,
} from '@modrinth/assets'
import { TabbedModal } from '@modrinth/ui'
import { ref } from 'vue'
import { defineMessage } from '@vintl/vintl'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'

const props = defineProps({
  instance: {
    type: Object,
    default() {
      return {}
    },
  },
})

const tabs = [
  {
    name: defineMessage({
      id: 'instance.settings.tabs.general',
      defaultMessage: 'General',
    }),
    icon: InfoIcon,
    content: GeneralSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.java',
      defaultMessage: 'Java',
    }),
    icon: CoffeeIcon,
    content: GeneralSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.window',
      defaultMessage: 'Window',
    }),
    icon: MonitorIcon,
    content: GeneralSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.hooks',
      defaultMessage: 'Hooks',
    }),
    icon: WrenchIcon,
    content: GeneralSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.modpack',
      defaultMessage: 'Modpack',
    }),
    icon: BoxIcon,
    content: GeneralSettings,
    modpackOnly: true,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.manage',
      defaultMessage: 'Manage',
    }),
    icon: ArchiveIcon,
    content: GeneralSettings,
  },
]

const modal = ref()

function show() {
  modal.value.show()
}

defineExpose({ show })
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
        <SettingsIcon /> Instance settings
      </span>
    </template>

    <TabbedModal :tabs="tabs" />
  </ModalWrapper>
</template>
