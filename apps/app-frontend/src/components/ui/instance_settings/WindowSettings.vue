<script setup lang="ts">
import { Checkbox, Toggle } from '@modrinth/ui'
import { computed, ref, type Ref, watch } from 'vue'
import { handleError } from '@/store/notifications'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { get } from '@/helpers/settings.ts'
import { edit } from '@/helpers/profile'
import type { AppSettings, InstanceSettingsTabProps } from '../../../helpers/types'

const { formatMessage } = useVIntl()

const props = defineProps<InstanceSettingsTabProps>()

const globalSettings = (await get().catch(handleError)) as AppSettings

const overrideWindowSettings = ref(
  !!props.instance.game_resolution || !!props.instance.force_fullscreen,
)
const resolution: Ref<[number, number]> = ref(
  props.instance.game_resolution ?? (globalSettings.game_resolution.slice() as [number, number]),
)
const fullscreenSetting: Ref<boolean> = ref(
  props.instance.force_fullscreen ?? globalSettings.force_fullscreen,
)

const editProfileObject = computed(() => {
  const editProfile: {
    force_fullscreen?: boolean
    game_resolution?: [number, number]
  } = {}

  if (overrideWindowSettings.value) {
    editProfile.force_fullscreen = fullscreenSetting.value

    if (!fullscreenSetting.value) {
      editProfile.game_resolution = resolution.value
    }
  }

  return editProfile
})

watch(
  [overrideWindowSettings, resolution, fullscreenSetting],
  async () => {
    await edit(props.instance.path, editProfileObject.value)
  },
  { deep: true },
)

const messages = defineMessages({
  customWindowSettings: {
    id: 'instance.settings.tabs.window.custom-window-settings',
    defaultMessage: 'Custom window settings',
  },
  fullscreen: {
    id: 'instance.settings.tabs.window.fullscreen',
    defaultMessage: 'Fullscreen',
  },
  fullscreenDescription: {
    id: 'instance.settings.tabs.window.fullscreen.description',
    defaultMessage: 'Make the game start in full screen when launched (using options.txt).',
  },
  width: {
    id: 'instance.settings.tabs.window.width',
    defaultMessage: 'Width',
  },
  widthDescription: {
    id: 'instance.settings.tabs.window.width.description',
    defaultMessage: 'The width of the game window when launched.',
  },
  enterWidth: {
    id: 'instance.settings.tabs.window.width.enter',
    defaultMessage: 'Enter width...',
  },
  height: {
    id: 'instance.settings.tabs.window.height',
    defaultMessage: 'Height',
  },
  heightDescription: {
    id: 'instance.settings.tabs.window.height.description',
    defaultMessage: 'The height of the game window when launched.',
  },
  enterHeight: {
    id: 'instance.settings.tabs.window.height.enter',
    defaultMessage: 'Enter height...',
  },
})
</script>

<template>
  <div>
    <Checkbox
      v-model="overrideWindowSettings"
      :label="formatMessage(messages.customWindowSettings)"
      @update:model-value="
        (value) => {
          if (!value) {
            resolution = globalSettings.game_resolution
            fullscreenSetting = globalSettings.force_fullscreen
          }
        }
      "
    />
    <div class="mt-2 flex items-center gap-4 justify-between">
      <div>
        <h2 class="m-0 mb-1 text-lg font-extrabold text-contrast">
          {{ formatMessage(messages.fullscreen) }}
        </h2>
        <p class="m-0">
          {{ formatMessage(messages.fullscreenDescription) }}
        </p>
      </div>
      <Toggle
        id="fullscreen"
        :model-value="overrideWindowSettings ? fullscreenSetting : globalSettings.force_fullscreen"
        :disabled="!overrideWindowSettings"
        @update:model-value="
          (e) => {
            fullscreenSetting = e
          }
        "
      />
    </div>

    <div class="mt-4 flex items-center gap-4 justify-between">
      <div>
        <h2 class="m-0 mb-1 text-lg font-extrabold text-contrast">
          {{ formatMessage(messages.width) }}
        </h2>
        <p class="m-0">
          {{ formatMessage(messages.widthDescription) }}
        </p>
      </div>
      <input
        id="width"
        v-model="resolution[0]"
        autocomplete="off"
        :disabled="!overrideWindowSettings || fullscreenSetting"
        type="number"
        :placeholder="formatMessage(messages.enterWidth)"
      />
    </div>

    <div class="mt-4 flex items-center gap-4 justify-between">
      <div>
        <h2 class="m-0 mb-1 text-lg font-extrabold text-contrast">
          {{ formatMessage(messages.height) }}
        </h2>
        <p class="m-0">
          {{ formatMessage(messages.heightDescription) }}
        </p>
      </div>
      <input
        id="height"
        v-model="resolution[1]"
        autocomplete="off"
        :disabled="!overrideWindowSettings || fullscreenSetting"
        type="number"
        :placeholder="formatMessage(messages.enterHeight)"
      />
    </div>
  </div>
</template>
