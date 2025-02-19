<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { SpinnerIcon, TrashIcon, UploadIcon, PlusIcon, EditIcon, CopyIcon } from '@modrinth/assets'
import { Avatar, ButtonStyled, OverflowMenu, Checkbox } from '@modrinth/ui'
import { computed, ref, type Ref, watch } from 'vue'
import { duplicate, edit, edit_icon, list, remove } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { trackEvent } from '@/helpers/analytics'
import { open } from '@tauri-apps/plugin-dialog'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { useRouter } from 'vue-router'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import type { InstanceSettingsTabProps, GameInstance } from '../../../helpers/types'

const { formatMessage } = useVIntl()
const router = useRouter()

const deleteConfirmModal = ref()

const props = defineProps<InstanceSettingsTabProps>()

const title = ref(props.instance.name)
const icon: Ref<string | undefined> = ref(props.instance.icon_path)
const groups = ref(props.instance.groups)

const newCategoryInput = ref('')

const installing = computed(() => props.instance.install_stage !== 'installed')

async function duplicateProfile() {
  await duplicate(props.instance.path).catch(handleError)
  trackEvent('InstanceDuplicate', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
  })
}

const allInstances = ref((await list()) as GameInstance[])
const availableGroups = computed(() => [
  ...new Set([...allInstances.value.flatMap((instance) => instance.groups), ...groups.value]),
])

async function resetIcon() {
  icon.value = undefined
  await edit_icon(props.instance.path, null).catch(handleError)
  trackEvent('InstanceRemoveIcon')
}

async function setIcon() {
  const value = await open({
    multiple: false,
    filters: [
      {
        name: 'Image',
        extensions: ['png', 'jpeg', 'svg', 'webp', 'gif', 'jpg'],
      },
    ],
  })

  if (!value) return

  icon.value = value
  await edit_icon(props.instance.path, icon.value).catch(handleError)

  trackEvent('InstanceSetIcon')
}

const editProfileObject = computed(() => ({
  name: title.value.trim().substring(0, 32) ?? 'Instance',
  groups: groups.value.map((x) => x.trim().substring(0, 32)).filter((x) => x.length > 0),
}))

const toggleGroup = (group: string) => {
  if (groups.value.includes(group)) {
    groups.value = groups.value.filter((x) => x !== group)
  } else {
    groups.value.push(group)
  }
}

const addCategory = () => {
  const text = newCategoryInput.value.trim()

  if (text.length > 0) {
    groups.value.push(text.substring(0, 32))
    newCategoryInput.value = ''
  }
}

watch(
  [title, groups, groups],
  async () => {
    await edit(props.instance.path, editProfileObject.value)
  },
  { deep: true },
)

const removing = ref(false)
async function removeProfile() {
  removing.value = true
  await remove(props.instance.path).catch(handleError)
  removing.value = false

  trackEvent('InstanceRemove', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
  })

  await router.push({ path: '/' })
}

const messages = defineMessages({
  name: {
    id: 'instance.settings.tabs.general.name',
    defaultMessage: 'Name',
  },
  libraryGroups: {
    id: 'instance.settings.tabs.general.library-groups',
    defaultMessage: 'Library groups',
  },
  libraryGroupsDescription: {
    id: 'instance.settings.tabs.general.library-groups.description',
    defaultMessage:
      'Library groups allow you to organize your instances into different sections in your library.',
  },
  libraryGroupsEnterName: {
    id: 'instance.settings.tabs.general.library-groups.enter-name',
    defaultMessage: 'Enter group name',
  },
  libraryGroupsCreate: {
    id: 'instance.settings.tabs.general.library-groups.create',
    defaultMessage: 'Create new group',
  },
  editIcon: {
    id: 'instance.settings.tabs.general.edit-icon',
    defaultMessage: 'Edit icon',
  },
  selectIcon: {
    id: 'instance.settings.tabs.general.edit-icon.select',
    defaultMessage: 'Select icon',
  },
  replaceIcon: {
    id: 'instance.settings.tabs.general.edit-icon.replace',
    defaultMessage: 'Replace icon',
  },
  removeIcon: {
    id: 'instance.settings.tabs.general.edit-icon.remove',
    defaultMessage: 'Remove icon',
  },
  duplicateInstance: {
    id: 'instance.settings.tabs.general.duplicate-instance',
    defaultMessage: 'Duplicate instance',
  },
  duplicateInstanceDescription: {
    id: 'instance.settings.tabs.general.duplicate-instance.description',
    defaultMessage: 'Creates a copy of this instance, including worlds, configs, mods, etc.',
  },
  duplicateButtonTooltipInstalling: {
    id: 'instance.settings.tabs.general.duplicate-button.tooltip.installing',
    defaultMessage: 'Cannot duplicate while installing.',
  },
  duplicateButton: {
    id: 'instance.settings.tabs.general.duplicate-button',
    defaultMessage: 'Duplicate',
  },
  deleteInstance: {
    id: 'instance.settings.tabs.general.delete',
    defaultMessage: 'Delete instance',
  },
  deleteInstanceDescription: {
    id: 'instance.settings.tabs.general.delete.description',
    defaultMessage:
      'Permanently deletes an instance from your device, including your worlds, configs, and all installed content. Be careful, as once you delete a instance there is no way to recover it.',
  },
  deleteInstanceButton: {
    id: 'instance.settings.tabs.general.delete.button',
    defaultMessage: 'Delete instance',
  },
  deletingInstanceButton: {
    id: 'instance.settings.tabs.general.deleting.button',
    defaultMessage: 'Deleting...',
  },
})
</script>

<template>
  <ConfirmModalWrapper
    ref="deleteConfirmModal"
    title="Are you sure you want to delete this instance?"
    description="If you proceed, all data for your instance will be permanently erased, including your worlds. You will not be able to recover it."
    :has-to-type="false"
    proceed-label="Delete"
    :show-ad-on-close="false"
    @proceed="removeProfile"
  />
  <div class="block">
    <div class="float-end ml-4 relative group">
      <OverflowMenu
        v-tooltip="formatMessage(messages.editIcon)"
        class="bg-transparent border-none appearance-none p-0 m-0 cursor-pointer group-active:scale-95 transition-transform"
        :options="[
          {
            id: 'select',
            action: () => setIcon(),
          },
          {
            id: 'remove',
            color: 'danger',
            action: () => resetIcon(),
            shown: !!icon,
          },
        ]"
      >
        <Avatar
          :src="icon ? convertFileSrc(icon) : icon"
          size="108px"
          class="!border-4 group-hover:brightness-75"
          :tint-by="props.instance.path"
          no-shadow
        />
        <div class="absolute top-0 right-0 m-2">
          <div
            class="p-2 m-0 text-primary flex items-center justify-center aspect-square bg-button-bg rounded-full border-button-border border-solid border-[1px] hovering-icon-shadow"
          >
            <EditIcon aria-hidden="true" class="h-4 w-4 text-primary" />
          </div>
        </div>
        <template #select>
          <UploadIcon />
          {{ icon ? formatMessage(messages.replaceIcon) : formatMessage(messages.selectIcon) }}
        </template>
        <template #remove> <TrashIcon /> {{ formatMessage(messages.removeIcon) }} </template>
      </OverflowMenu>
    </div>
    <label for="instance-name" class="m-0 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.name) }}
    </label>
    <div class="flex">
      <input
        id="instance-name"
        v-model="title"
        autocomplete="off"
        maxlength="80"
        class="flex-grow"
        type="text"
      />
    </div>
    <template v-if="instance.install_stage == 'installed'">
      <div>
        <h2
          id="duplicate-instance-label"
          class="m-0 mt-4 mb-1 text-lg font-extrabold text-contrast block"
        >
          {{ formatMessage(messages.duplicateInstance) }}
        </h2>
        <p class="m-0 mb-2">
          {{ formatMessage(messages.duplicateInstanceDescription) }}
        </p>
      </div>
      <ButtonStyled>
        <button
          v-tooltip="installing ? formatMessage(messages.duplicateButtonTooltipInstalling) : null"
          aria-labelledby="duplicate-instance-label"
          :disabled="installing"
          @click="duplicateProfile"
        >
          <CopyIcon /> {{ formatMessage(messages.duplicateButton) }}
        </button>
      </ButtonStyled>
    </template>
    <h2 class="m-0 mt-4 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.libraryGroups) }}
    </h2>
    <p class="m-0 mb-2">
      {{ formatMessage(messages.libraryGroupsDescription) }}
    </p>
    <div class="flex flex-col gap-1">
      <Checkbox
        v-for="group in availableGroups"
        :key="group"
        :model-value="groups.includes(group)"
        :label="group"
        @click="toggleGroup(group)"
      />
      <div class="flex gap-2 items-center">
        <input
          v-model="newCategoryInput"
          type="text"
          :placeholder="formatMessage(messages.libraryGroupsEnterName)"
          @submit="() => addCategory"
        />
        <ButtonStyled>
          <button class="w-fit" @click="() => addCategory()">
            <PlusIcon /> {{ formatMessage(messages.libraryGroupsCreate) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
    <h2 id="delete-instance-label" class="m-0 mt-4 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.deleteInstance) }}
    </h2>
    <p class="m-0 mb-2">
      {{ formatMessage(messages.deleteInstanceDescription) }}
    </p>
    <ButtonStyled color="red">
      <button
        aria-labelledby="delete-instance-label"
        :disabled="removing"
        @click="deleteConfirmModal.show()"
      >
        <SpinnerIcon v-if="removing" class="animate-spin" />
        <TrashIcon v-else />
        {{
          removing
            ? formatMessage(messages.deletingInstanceButton)
            : formatMessage(messages.deleteInstanceButton)
        }}
      </button>
    </ButtonStyled>
  </div>
</template>
<style scoped lang="scss">
.hovering-icon-shadow {
  box-shadow: var(--shadow-inset-sm), var(--shadow-raised);
}
</style>
