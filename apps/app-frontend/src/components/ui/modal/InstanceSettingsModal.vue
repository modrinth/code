<script setup lang="ts">
import {
  ChevronRightIcon,
  CoffeeIcon,
  InfoIcon,
  WrenchIcon,
  MonitorIcon,
  CodeIcon,
} from '@modrinth/assets'
import { Avatar, TabbedModal, type TabbedModalTab } from '@modrinth/ui'
import { ref } from 'vue'
import { defineMessage, useVIntl } from '@vintl/vintl'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import GeneralSettings from '@/components/ui/instance_settings/GeneralSettings.vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import InstallationSettings from '@/components/ui/instance_settings/InstallationSettings.vue'
import JavaSettings from '@/components/ui/instance_settings/JavaSettings.vue'
import WindowSettings from '@/components/ui/instance_settings/WindowSettings.vue'
import HooksSettings from '@/components/ui/instance_settings/HooksSettings.vue'
import type { InstanceSettingsTabProps } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const tabs: TabbedModalTab<InstanceSettingsTabProps>[] = [
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
      id: 'instance.settings.tabs.installation',
      defaultMessage: 'Installation',
    }),
    icon: WrenchIcon,
    content: InstallationSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.window',
      defaultMessage: 'Window',
    }),
    icon: MonitorIcon,
    content: WindowSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.java',
      defaultMessage: 'Java and memory',
    }),
    icon: CoffeeIcon,
    content: JavaSettings,
  },
  {
    name: defineMessage({
      id: 'instance.settings.tabs.hooks',
      defaultMessage: 'Launch hooks',
    }),
    icon: CodeIcon,
    content: HooksSettings,
  },
]

const modal = ref()

function show() {
  modal.value.show()
}

defineExpose({ show })

const titleMessage = defineMessage({
  id: 'instance.settings.title',
  defaultMessage: 'Settings',
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <span class="flex items-center gap-2 text-lg font-semibold text-primary">
        <Avatar
          :src="instance.icon_path ? convertFileSrc(instance.icon_path) : undefined"
          size="24px"
          :tint-by="props.instance.path"
        />
        {{ instance.name }} <ChevronRightIcon />
        <span class="font-extrabold text-contrast">{{ formatMessage(titleMessage) }}</span>
      </span>
    </template>

    <TabbedModal :tabs="tabs.map((tab) => ({ ...tab, props }))" />
  </ModalWrapper>
</template>
