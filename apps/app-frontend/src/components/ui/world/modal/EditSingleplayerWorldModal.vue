<script setup lang="ts">
import { ChevronRightIcon, SaveIcon, XIcon, UndoIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, commonMessages } from '@modrinth/ui'
import { computed, ref } from 'vue'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import type { GameInstance } from '@/helpers/types'
import type { DisplayStatus, SingleplayerWorld } from '@/helpers/worlds.ts'
import { set_world_display_status, rename_world, reset_world_icon } from '@/helpers/worlds.ts'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { handleError } from '@/store/notifications'
import HideFromHomeOption from '@/components/ui/world/modal/HideFromHomeOption.vue'

const { formatMessage } = useVIntl()

const emit = defineEmits<{
  submit: [path: string, name: string, removeIcon: boolean, displayStatus: DisplayStatus]
}>()

const props = defineProps<{
  instance: GameInstance
}>()

const modal = ref()

const icon = ref()
const name = ref()
const path = ref()
const removeIcon = ref(false)
const displayStatus = ref<DisplayStatus>('normal')
const hideFromHome = ref(false)

const newDisplayStatus = computed(() => (hideFromHome.value ? 'hidden' : 'normal'))

async function saveWorld() {
  await rename_world(props.instance.path, path.value, name.value).catch(handleError)

  if (removeIcon.value) {
    await reset_world_icon(props.instance.path, path.value).catch(handleError)
  }
  if (newDisplayStatus.value !== displayStatus.value) {
    await set_world_display_status(
      props.instance.path,
      'singleplayer',
      path.value,
      newDisplayStatus.value,
    )
  }

  emit('submit', path.value, name.value, removeIcon.value, newDisplayStatus.value)
  hide()
}

function show(world: SingleplayerWorld) {
  name.value = world.name
  path.value = world.path
  icon.value = world.icon
  displayStatus.value = world.display_status
  hideFromHome.value = world.display_status === 'hidden'
  modal.value.show()
}

function hide() {
  modal.value.hide()
}

defineExpose({ show })

const messages = defineMessages({
  title: {
    id: 'instance.edit-world.title',
    defaultMessage: 'Edit world',
  },
  name: {
    id: 'instance.edit-world.name',
    defaultMessage: 'Name',
  },
  placeholderName: {
    id: 'instance.edit-world.placeholder-name',
    defaultMessage: 'Minecraft World',
  },
  resetIcon: {
    id: 'instance.edit-world.reset-icon',
    defaultMessage: 'Reset icon',
  },
})
</script>
<template>
  <ModalWrapper ref="modal">
    <template #title>
      <Avatar :src="removeIcon || !icon ? undefined : icon" size="24px" />
      {{ instance.name }} <ChevronRightIcon />
      <span class="font-extrabold text-lg text-contrast">{{ formatMessage(messages.title) }}</span>
    </template>
    <div class="w-[450px]">
      <h2 class="text-lg font-extrabold text-contrast mt-0 mb-1">
        {{ formatMessage(messages.name) }}
      </h2>
      <input
        v-model="name"
        type="text"
        :placeholder="formatMessage(messages.placeholderName)"
        class="w-full"
        autocomplete="off"
      />
      <HideFromHomeOption v-model="hideFromHome" class="mt-3" />
    </div>
    <div class="flex gap-2 mt-4">
      <ButtonStyled color="brand">
        <button @click="saveWorld">
          <SaveIcon />
          {{ formatMessage(commonMessages.saveChangesButton) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button :disabled="removeIcon || !icon" @click="removeIcon = true">
          <UndoIcon />
          {{ formatMessage(messages.resetIcon) }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button @click="hide()">
          <XIcon />
          {{ formatMessage(commonMessages.cancelButton) }}
        </button>
      </ButtonStyled>
    </div>
  </ModalWrapper>
</template>
