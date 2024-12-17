<script setup lang="ts">
import { HammerIcon, DownloadIcon, WrenchIcon, UndoIcon, SpinnerIcon } from '@modrinth/assets'
import { Checkbox, Chips, ButtonStyled, TeleportDropdownMenu } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref, shallowRef, watch } from 'vue'
import { edit, install } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { trackEvent } from '@/helpers/analytics'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import { formatCategory, type GameVersionTag, type PlatformTag } from '@modrinth/utils'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'

const { formatMessage } = useVIntl()

const repairConfirmModal = ref()

const props = defineProps<{
  instance: GameInstance
  offline?: boolean
}>()

const loader = ref(props.instance.loader)
const gameVersion = ref(props.instance.game_version)

const showSnapshots = ref(false)

const [
  fabric_versions,
  forge_versions,
  quilt_versions,
  neoforge_versions,
  all_game_versions,
  loaders,
] = await Promise.all([
  get_loader_versions('fabric')
    .then((manifest: Manifest) => shallowRef(manifest))
    .catch(handleError),
  get_loader_versions('forge')
    .then((manifest: Manifest) => shallowRef(manifest))
    .catch(handleError),
  get_loader_versions('quilt')
    .then((manifest: Manifest) => shallowRef(manifest))
    .catch(handleError),
  get_loader_versions('neo')
    .then((manifest: Manifest) => shallowRef(manifest))
    .catch(handleError),
  get_game_versions()
    .then((gameVersions: GameVersionTag[]) => shallowRef(gameVersions))
    .catch(handleError),
  get_loaders()
    .then((value: PlatformTag[]) =>
      value
        .filter(
          (item) => item.supported_project_types.includes('modpack') || item.name === 'vanilla',
        )
        .sort((a, b) => (a.name === 'vanilla' ? -1 : b.name === 'vanilla' ? 1 : 0)),
    )
    .then((loader: PlatformTag[]) => ref(loader))
    .catch(handleError),
])

const currentLoaderIcon = computed(
  () => loaders?.value.find((x) => x.name === props.instance.loader)?.icon,
)

const gameVersionsForLoader = computed(() => {
  return all_game_versions?.value.filter((item) => {
    if (loader.value === 'fabric') {
      return !!fabric_versions?.value.gameVersions.some((x) => item.version === x.id)
    } else if (loader.value === 'forge') {
      return !!forge_versions?.value.gameVersions.some((x) => item.version === x.id)
    } else if (loader.value === 'quilt') {
      return !!quilt_versions?.value.gameVersions.some((x) => item.version === x.id)
    } else if (loader.value === 'neoforge') {
      return !!neoforge_versions?.value.gameVersions.some((x) => item.version === x.id)
    }

    return []
  })
})

const hasSnapshots = computed(() =>
  gameVersionsForLoader.value?.some((x) => x.version_type !== 'release'),
)

const selectableGameVersionNumbers = computed(() => {
  return gameVersionsForLoader.value
    ?.filter((x) => x.version_type === 'release' || showSnapshots.value)
    .map((x) => x.version)
})

const selectableLoaderVersions: ComputedRef<ManifestLoaderVersion[] | undefined> = computed(() => {
  if (gameVersion.value) {
    if (loader.value === 'fabric') {
      return fabric_versions?.value.gameVersions[0].loaders
    } else if (loader.value === 'forge') {
      return forge_versions?.value?.gameVersions?.find((item) => item.id === gameVersion.value)
        ?.loaders
    } else if (loader.value === 'quilt') {
      return quilt_versions?.value.gameVersions[0].loaders
    } else if (loader.value === 'neoforge') {
      return neoforge_versions?.value?.gameVersions?.find((item) => item.id === gameVersion.value)
        ?.loaders
    }
  }
  return []
})
const loaderVersionIndex: Ref<number> = ref(-1)

resetLoaderVersionIndex()

function resetLoaderVersionIndex() {
  loaderVersionIndex.value =
    selectableLoaderVersions.value?.findIndex((x) => x.id === props.instance.loader_version) || -1
}

const isValid = computed(() => {
  return (
    selectableGameVersionNumbers.value?.includes(gameVersion.value) &&
    ((loaderVersionIndex.value !== undefined && loaderVersionIndex.value >= 0) ||
      loader.value === 'vanilla')
  )
})

const isChanged = computed(() => {
  return (
    loader.value !== props.instance.loader ||
    gameVersion.value !== props.instance.game_version ||
    (loader.value !== 'vanilla' &&
      loaderVersionIndex.value !== undefined &&
      loaderVersionIndex.value >= 0 &&
      selectableLoaderVersions.value?.[loaderVersionIndex.value].id !==
        props.instance.loader_version)
  )
})

watch(loader, () => (loaderVersionIndex.value = 0))

const editing = ref(false)
async function saveGvLoaderEdits() {
  editing.value = true

  const editProfile: { loader?: string; game_version?: string; loader_version?: string } = {}
  editProfile.loader = loader.value
  editProfile.game_version = gameVersion.value

  if (loader.value !== 'vanilla' && loaderVersionIndex.value !== undefined) {
    editProfile.loader_version = selectableLoaderVersions.value?.[loaderVersionIndex.value].id
  } else {
    loaderVersionIndex.value = -1
  }
  await edit(props.instance.path, editProfile).catch(handleError)
  await repairProfile(false)

  editing.value = false
}

const installing = computed(() => props.instance.install_stage !== 'installed')
const repairing = ref(false)

async function repairProfile(force: boolean) {
  repairing.value = true
  await install(props.instance.path, force).catch(handleError)
  repairing.value = false

  trackEvent('InstanceRepair', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
  })
}

const messages = defineMessages({
  currentlyInstalled: {
    id: 'instance.settings.tabs.installation.currently-installed',
    defaultMessage: 'Currently installed',
  },
  platform: {
    id: 'instance.settings.tabs.installation.platform',
    defaultMessage: 'Platform',
  },
  gameVersion: {
    id: 'instance.settings.tabs.installation.game-version',
    defaultMessage: 'Game version',
  },
  loaderVersion: {
    id: 'instance.settings.tabs.installation.loader-version',
    defaultMessage: '{loader} version',
  },
  showAllVersions: {
    id: 'instance.settings.tabs.installation.show-all-versions',
    defaultMessage: 'Show all versions',
  },
  install: {
    id: 'instance.settings.tabs.installation.install',
    defaultMessage: 'Install',
  },
  installing: {
    id: 'instance.settings.tabs.installation.installing',
    defaultMessage: 'Installing...',
  },
  resetSelections: {
    id: 'instance.settings.tabs.installation.reset-selections',
    defaultMessage: 'Reset selections',
  },
  unknownVersion: {
    id: 'instance.settings.tabs.installation.unknown-version',
    defaultMessage: '(unknown version)',
  },
  repairConfirmTitle: {
    id: 'instance.settings.tabs.installation.repair.confirm-title',
    defaultMessage: 'Repair instance?',
  },
  repairConfirmDescription: {
    id: 'instance.settings.tabs.installation.repair.description',
    defaultMessage:
      'Repairing reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors.',
  },
  repairButton: {
    id: 'instance.settings.tabs.installation.repair.button',
    defaultMessage: 'Repair',
  },
  repairingButton: {
    id: 'instance.settings.tabs.installation.repairing.button',
    defaultMessage: 'Repairing',
  },
  repairInProgress: {
    id: 'instance.settings.tabs.installation.repair.in-progress',
    defaultMessage: 'Repair in progress',
  },
  repairCannotWhileInstalling: {
    id: 'instance.settings.tabs.installation.repair.cannot-while-installing',
    defaultMessage: 'Cannot repair while installing',
  },
  repairCannotWhileOffline: {
    id: 'instance.settings.tabs.installation.repair.cannot-while-offline',
    defaultMessage: 'Cannot repair while offline',
  },
  minecraftVersion: {
    id: 'instance.settings.tabs.installation.minecraft-version',
    defaultMessage: 'Minecraft {version}',
  },
})
</script>

<template>
  <ConfirmModalWrapper
    ref="repairConfirmModal"
    :title="formatMessage(messages.repairConfirmTitle)"
    :description="formatMessage(messages.repairConfirmDescription)"
    :proceed-icon="HammerIcon"
    :proceed-label="formatMessage(messages.repairButton)"
    :danger="false"
    @proceed="() => repairProfile(true)"
  />
  <div>
    <h2 id="project-name" class="m-0 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.currentlyInstalled) }}
    </h2>
    <div class="flex gap-4 items-center justify-between p-4 bg-bg rounded-2xl">
      <div class="flex gap-2 items-center">
        <div
          class="w-10 h-10 flex items-center justify-center rounded-xl bg-button-bg border-solid border-[1px] border-button-border p-2 [&_svg]:h-full [&_svg]:w-full"
        >
          <div v-if="!!currentLoaderIcon" class="contents" v-html="currentLoaderIcon" />
          <WrenchIcon v-else />
        </div>
        <div class="flex flex-col gap-2 justify-center">
          <span class="font-semibold leading-none">
            {{ formatMessage(messages.minecraftVersion, { version: instance.game_version }) }}
          </span>
          <span class="text-sm text-secondary leading-none">
            {{ formatCategory(instance.loader) }}
            <template v-if="instance.loader !== 'vanilla'">
              {{ instance.loader_version || formatMessage(messages.unknownVersion) }}
            </template>
          </span>
        </div>
      </div>
      <div>
        <ButtonStyled color="orange" type="transparent" hover-color-fill="background">
          <button
            v-tooltip="repairing ? formatMessage(messages.repairInProgress) :
            installing ? formatMessage(messages.repairCannotWhileInstalling) :
              offline ? formatMessage(messages.repairCannotWhileOffline) : null"
            :disabled="installing || repairing || offline"
            @click="repairConfirmModal.show()"
          >
            <SpinnerIcon v-if="repairing" class="animate-spin" />
            <HammerIcon v-else />
            {{ repairing ? formatMessage(messages.repairingButton) : formatMessage(messages.repairButton) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
    <h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.platform) }}
    </h2>
    <Chips v-if="loaders" v-model="loader" :items="loaders.map((x) => x.name)" class="mt-2" />
    <h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.gameVersion) }}
    </h2>
    <div class="flex flex-wrap mt-2 gap-2">
      <TeleportDropdownMenu
        v-if="selectableGameVersionNumbers !== undefined"
        v-model="gameVersion"
        :options="selectableGameVersionNumbers"
        name="Game Version Dropdown"
      />
      <Checkbox
        v-if="hasSnapshots"
        v-model="showSnapshots"
        :label="formatMessage(messages.showAllVersions)"
      />
    </div>
    <template v-if="loader !== 'vanilla'">
      <h2 class="m-0 mt-4 text-lg font-extrabold text-contrast block">
        {{ formatMessage(messages.loaderVersion, { loader: formatCategory(loader) }) }}
      </h2>
      <TeleportDropdownMenu
        v-if="selectableLoaderVersions"
        :model-value="selectableLoaderVersions[loaderVersionIndex]"
        :options="selectableLoaderVersions"
        :display-name="(option: ManifestLoaderVersion) => option?.id"
        name="Version selector"
        class="mt-2"
        @change="(value) => (loaderVersionIndex = value.index)"
      />
    </template>
    <div class="mt-4 flex flex-wrap gap-2">
      <ButtonStyled color="brand">
        <button :disabled="!isValid || !isChanged || editing" @click="saveGvLoaderEdits()">
          <SpinnerIcon v-if="editing" class="animate-spin" />
          <DownloadIcon v-else />
          {{ editing ? 'Installing...' : 'Install' }}
        </button>
      </ButtonStyled>
      <ButtonStyled>
        <button
          :disabled="!isChanged"
          @click="
            () => {
              loader = instance.loader
              gameVersion = instance.game_version
              resetLoaderVersionIndex()
            }
          "
        >
          <UndoIcon />
          {{ formatMessage(messages.resetSelections) }}
        </button>
      </ButtonStyled>
    </div>
  </div>
</template>
