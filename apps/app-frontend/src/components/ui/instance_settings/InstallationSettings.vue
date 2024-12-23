<script setup lang="ts">
import {
  TransferIcon,
  IssuesIcon,
  HammerIcon,
  DownloadIcon,
  WrenchIcon,
  UndoIcon,
  SpinnerIcon,
  UnplugIcon,
  UnlinkIcon,
} from '@modrinth/assets'
import { Avatar, Checkbox, Chips, ButtonStyled, TeleportDropdownMenu } from '@modrinth/ui'
import { computed, type ComputedRef, type Ref, ref, shallowRef, watch } from 'vue'
import { edit, install, update_repair_modrinth } from '@/helpers/profile'
import { handleError } from '@/store/notifications'
import { trackEvent } from '@/helpers/analytics'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { get_loader_versions } from '@/helpers/metadata'
import { get_game_versions, get_loaders } from '@/helpers/tags'
import {
  formatCategory,
  type GameVersionTag,
  type PlatformTag,
  type Project,
  type Version,
} from '@modrinth/utils'
import ConfirmModalWrapper from '@/components/ui/modal/ConfirmModalWrapper.vue'
import { get_project, get_version_many } from '@/helpers/cache'
import ModpackVersionModal from '@/components/ui/ModpackVersionModal.vue'
import dayjs from 'dayjs'
import type {
  InstanceSettingsTabProps,
  ManifestLoaderVersion,
  Manifest,
} from '../../../helpers/types'

const { formatMessage } = useVIntl()

const repairConfirmModal = ref()
const modpackVersionModal = ref()
const modalConfirmUnpair = ref()
const modalConfirmReinstall = ref()

const props = defineProps<InstanceSettingsTabProps>()

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

const modpackProject: Ref<Project | null> = ref(null)
const modpackVersion: Ref<Version | null> = ref(null)
const modpackVersions: Ref<Version[] | null> = ref(null)
const fetching = ref(true)

if (props.instance.linked_data && props.instance.linked_data.project_id && !props.offline) {
  get_project(props.instance.linked_data.project_id, 'must_revalidate')
    .then((project) => {
      modpackProject.value = project

      if (project && project.versions) {
        get_version_many(project.versions, 'must_revalidate')
          .then((versions: Version[]) => {
            modpackVersions.value = versions.sort((a, b) =>
              dayjs(b.date_published).diff(dayjs(a.date_published)),
            )
            modpackVersion.value =
              versions.find(
                (version: Version) => version.id === props.instance.linked_data?.version_id,
              ) ?? null
          })
          .catch(handleError)
          .finally(() => {
            fetching.value = false
          })
      }
    })
    .catch((err) => {
      handleError(err)
      fetching.value = false
    })
} else {
  fetching.value = false
}

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
    selectableLoaderVersions.value?.findIndex((x) => x.id === props.instance.loader_version) ?? -1
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

watch(loader, () => {
  loaderVersionIndex.value = 0
})

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
  console.log('Editing:')
  console.log(loader.value)

  await edit(props.instance.path, editProfile).catch(handleError)
  await repairProfile(false)

  editing.value = false
}

const installing = computed(() => props.instance.install_stage !== 'installed')
const repairing = ref(false)
const reinstalling = ref(false)
const changingVersion = ref(false)

async function repairProfile(force: boolean) {
  if (force) {
    repairing.value = true
  }
  await install(props.instance.path, force).catch(handleError)
  if (force) {
    repairing.value = false
  }

  trackEvent('InstanceRepair', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
  })
}

async function unpairProfile() {
  await edit(props.instance.path, {
    linked_data: null,
  })
  modpackProject.value = null
  modpackVersion.value = null
  modpackVersions.value = null
  modalConfirmUnpair.value.hide()
}

async function repairModpack() {
  reinstalling.value = true
  await update_repair_modrinth(props.instance.path).catch(handleError)
  reinstalling.value = false

  trackEvent('InstanceRepair', {
    loader: props.instance.loader,
    game_version: props.instance.game_version,
  })
}

const messages = defineMessages({
  cannotWhileInstalling: {
    id: 'instance.settings.tabs.installation.tooltip.cannot-while-installing',
    defaultMessage: 'Cannot {action} while installing',
  },
  cannotWhileOffline: {
    id: 'instance.settings.tabs.installation.tooltip.cannot-while-offline',
    defaultMessage: 'Cannot {action} while offline',
  },
  cannotWhileRepairing: {
    id: 'instance.settings.tabs.installation.tooltip.cannot-while-repairing',
    defaultMessage: 'Cannot {action} while repairing',
  },
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
  resetSelections: {
    id: 'instance.settings.tabs.installation.reset-selections',
    defaultMessage: 'Reset to current',
  },
  unknownVersion: {
    id: 'instance.settings.tabs.installation.unknown-version',
    defaultMessage: '(unknown version)',
  },
  repairConfirmTitle: {
    id: 'instance.settings.tabs.installation.repair.confirm.title',
    defaultMessage: 'Repair instance?',
  },
  repairConfirmDescription: {
    id: 'instance.settings.tabs.installation.repair.confirm.description',
    defaultMessage:
      'Repairing reinstalls Minecraft dependencies and checks for corruption. This may resolve issues if your game is not launching due to launcher-related errors, but will not resolve issues or crashes related to installed mods.',
  },
  repairButton: {
    id: 'instance.settings.tabs.installation.repair.button',
    defaultMessage: 'Repair',
  },
  repairingButton: {
    id: 'instance.settings.tabs.installation.repair.button.repairing',
    defaultMessage: 'Repairing',
  },
  repairInProgress: {
    id: 'instance.settings.tabs.installation.repair.in-progress',
    defaultMessage: 'Repair in progress',
  },
  repairAction: {
    id: 'instance.settings.tabs.installation.tooltip.action.repair',
    defaultMessage: 'repair',
  },
  changeVersionCannotWhileFetching: {
    id: 'instance.settings.tabs.installation.change-version.cannot-while-fetching',
    defaultMessage: 'Fetching modpack versions',
  },
  changeVersionButton: {
    id: 'instance.settings.tabs.installation.change-version.button',
    defaultMessage: 'Change version',
  },
  changeVersionAction: {
    id: 'instance.settings.tabs.installation.tooltip.action.change-version',
    defaultMessage: 'change version',
  },
  installingButton: {
    id: 'instance.settings.tabs.installation.change-version.button.installing',
    defaultMessage: 'Installing',
  },
  installInProgress: {
    id: 'instance.settings.tabs.installation.install.in-progress',
    defaultMessage: 'Installation in progress',
  },
  installButton: {
    id: 'instance.settings.tabs.installation.change-version.button.install',
    defaultMessage: 'Install',
  },
  alreadyInstalledVanilla: {
    id: 'instance.settings.tabs.installation.change-version.already-installed.vanilla',
    defaultMessage: 'Vanilla {game_version} already installed',
  },
  alreadyInstalledModded: {
    id: 'instance.settings.tabs.installation.change-version.already-installed.modded',
    defaultMessage: '{platform} {version} for Minecraft {game_version} already installed',
  },
  installAction: {
    id: 'instance.settings.tabs.installation.tooltip.action.install',
    defaultMessage: 'install',
  },
  installingNewVersion: {
    id: 'instance.settings.tabs.installation.change-version.in-progress',
    defaultMessage: 'Installing new version',
  },
  minecraftVersion: {
    id: 'instance.settings.tabs.installation.minecraft-version',
    defaultMessage: 'Minecraft {version}',
  },
  noLoaderVersions: {
    id: 'instance.settings.tabs.installation.no-loader-versions',
    defaultMessage: '{loader} is not available for Minecraft {version}. Try another mod loader.',
  },
  noConnection: {
    id: 'instance.settings.tabs.installation.no-connection',
    defaultMessage: 'Cannot fetch linked modpack details. Please check your internet connection.',
  },
  noModpackFound: {
    id: 'instance.settings.tabs.installation.no-modpack-found',
    defaultMessage:
      'This instance is linked to a modpack, but the modpack could not be found on Modrinth.',
  },
  debugInformation: {
    id: 'instance.settings.tabs.installation.debug-information',
    defaultMessage: 'Debug information:',
  },
  fetchingModpackDetails: {
    id: 'instance.settings.tabs.installation.fetching-modpack-details',
    defaultMessage: 'Fetching modpack details',
  },
  unlinkInstanceTitle: {
    id: 'instance.settings.tabs.installation.unlink.title',
    defaultMessage: 'Unlink from modpack',
  },
  unlinkInstanceDescription: {
    id: 'instance.settings.tabs.installation.unlink.description',
    defaultMessage: `This instance is linked to a modpack, which means mods can't be updated and you can't change the mod loader or Minecraft version. Unlinking will permanently disconnect this instance from the modpack.`,
  },
  unlinkInstanceButton: {
    id: 'instance.settings.tabs.installation.unlink.button',
    defaultMessage: 'Unlink instance',
  },
  unlinkInstanceConfirmTitle: {
    id: 'instance.settings.tabs.installation.unlink.confirm.title',
    defaultMessage: 'Are you sure you want to unlink this instance?',
  },
  unlinkInstanceConfirmDescription: {
    id: 'instance.settings.tabs.installation.unlink.confirm.description',
    defaultMessage:
      'If you proceed, you will not be able to re-link it without creating an entirely new instance. You will no longer receive modpack updates and it will become a normal.',
  },
  reinstallModpackConfirmTitle: {
    id: 'instance.settings.tabs.installation.reinstall.confirm.title',
    defaultMessage: 'Are you sure you want to reinstall this instance?',
  },
  reinstallModpackConfirmDescription: {
    id: 'instance.settings.tabs.installation.reinstall.confirm.description',
    defaultMessage: `Reinstalling will reset all installed or modified content to what is provided by the modpack, removing any mods or content you have added on top of the original installation. This may fix unexpected behavior if changes have been made to the instance, but if your worlds now depend on additional installed content, it may break existing worlds.`,
  },
  reinstallModpackTitle: {
    id: 'instance.settings.tabs.installation.reinstall.title',
    defaultMessage: 'Reinstall modpack',
  },
  reinstallModpackDescription: {
    id: 'instance.settings.tabs.installation.reinstall.description',
    defaultMessage: `Resets the instance's content to its original state, removing any mods or content you have added on top of the original modpack.`,
  },
  reinstallModpackButton: {
    id: 'instance.settings.tabs.installation.reinstall.button',
    defaultMessage: 'Reinstall modpack',
  },
  reinstallingModpackButton: {
    id: 'instance.settings.tabs.installation.reinstall.button.reinstalling',
    defaultMessage: 'Reinstalling modpack',
  },
  reinstallAction: {
    id: 'instance.settings.tabs.installation.tooltip.action.reinstall',
    defaultMessage: 'reinstall',
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
    :show-ad-on-close="false"
    @proceed="() => repairProfile(true)"
  />
  <ModpackVersionModal
    v-if="instance.linked_data && modpackVersions"
    ref="modpackVersionModal"
    :instance="instance"
    :versions="modpackVersions"
    @finish-install="
      () => {
        changingVersion = false
        modpackVersion =
          modpackVersions?.find(
            (version: Version) => version.id === props.instance.linked_data?.version_id,
          ) ?? null
      }
    "
  />
  <ConfirmModalWrapper
    ref="modalConfirmUnpair"
    :title="formatMessage(messages.unlinkInstanceConfirmTitle)"
    :description="formatMessage(messages.unlinkInstanceConfirmDescription)"
    :proceed-icon="UnlinkIcon"
    :proceed-label="formatMessage(messages.unlinkInstanceButton)"
    :show-ad-on-close="false"
    @proceed="() => unpairProfile()"
  />
  <ConfirmModalWrapper
    ref="modalConfirmReinstall"
    :title="formatMessage(messages.reinstallModpackConfirmTitle)"
    :description="formatMessage(messages.reinstallModpackConfirmDescription)"
    :proceed-icon="DownloadIcon"
    :proceed-label="formatMessage(messages.reinstallModpackButton)"
    :show-ad-on-close="false"
    @proceed="() => repairModpack()"
  />
  <div>
    <h2 id="project-name" class="m-0 mb-1 text-lg font-extrabold text-contrast block">
      {{ formatMessage(messages.currentlyInstalled) }}
    </h2>
    <div
      v-if="!modpackProject && instance.linked_data && offline && !fetching"
      class="text-secondary font-medium mb-2"
    >
      <UnplugIcon class="top-[3px] relative" /> {{ formatMessage(messages.noConnection) }}
    </div>
    <div v-else-if="!modpackProject && instance.linked_data && !fetching" class="mb-2">
      <p class="text-brand-red font-medium mt-0">
        <IssuesIcon class="top-[3px] relative" /> {{ formatMessage(messages.noModpackFound) }}
      </p>
      <p>{{ formatMessage(messages.debugInformation) }}</p>
      <div class="bg-bg p-6 rounded-2xl mt-2 text-sm text-secondary">
        {{ instance.linked_data }}
      </div>
    </div>
    <div class="flex gap-4 items-center justify-between p-4 bg-bg rounded-2xl">
      <div v-if="fetching" class="flex items-center gap-2 h-10">
        <SpinnerIcon class="animate-spin" />
        {{ formatMessage(messages.fetchingModpackDetails) }}
      </div>
      <template v-else>
        <div class="flex gap-2 items-center">
          <Avatar v-if="modpackProject" :src="modpackProject?.icon_url" size="40px" />
          <div
            v-else
            class="w-10 h-10 flex items-center justify-center rounded-full bg-button-bg border-solid border-[1px] border-button-border p-2 [&_svg]:h-full [&_svg]:w-full"
          >
            <div v-if="!!currentLoaderIcon" class="contents" v-html="currentLoaderIcon" />
            <WrenchIcon v-else />
          </div>
          <div class="flex flex-col gap-2 justify-center">
            <span class="font-semibold leading-none">
              {{
                modpackProject
                  ? modpackProject.title
                  : formatMessage(messages.minecraftVersion, { version: instance.game_version })
              }}
            </span>
            <span class="text-sm text-secondary leading-none">
              {{
                modpackProject
                  ? modpackVersion
                    ? modpackVersion?.version_number
                    : 'Unknown version'
                  : formatCategory(instance.loader)
              }}
              <template v-if="instance.loader !== 'vanilla' && !modpackProject">
                {{ instance.loader_version || formatMessage(messages.unknownVersion) }}
              </template>
            </span>
          </div>
        </div>
        <div class="flex gap-1">
          <ButtonStyled color="orange" type="transparent" hover-color-fill="background">
            <button
              v-tooltip="
                repairing
                  ? formatMessage(messages.repairInProgress)
                  : installing || reinstalling
                    ? formatMessage(messages.cannotWhileInstalling, {
                        action: formatMessage(messages.repairAction),
                      })
                    : offline
                      ? formatMessage(messages.cannotWhileOffline, {
                          action: formatMessage(messages.repairAction),
                        })
                      : null
              "
              :disabled="installing || repairing || reinstalling || offline"
              @click="repairConfirmModal.show()"
            >
              <SpinnerIcon v-if="repairing" class="animate-spin" />
              <HammerIcon v-else />
              {{
                repairing
                  ? formatMessage(messages.repairingButton)
                  : formatMessage(messages.repairButton)
              }}
            </button>
          </ButtonStyled>
          <ButtonStyled v-if="modpackProject" hover-color-fill="background">
            <button
              v-tooltip="
                changingVersion
                  ? formatMessage(messages.installingNewVersion)
                  : repairing
                    ? formatMessage(messages.cannotWhileRepairing, {
                        action: formatMessage(messages.changeVersionAction),
                      })
                    : installing || reinstalling
                      ? formatMessage(messages.cannotWhileInstalling, {
                          action: formatMessage(messages.changeVersionAction),
                        })
                      : fetching && !modpackVersions
                        ? formatMessage(messages.changeVersionCannotWhileFetching)
                        : offline
                          ? formatMessage(messages.cannotWhileOffline, {
                              action: formatMessage(messages.changeVersionAction),
                            })
                          : null
              "
              :disabled="
                changingVersion ||
                repairing ||
                installing ||
                reinstalling ||
                offline ||
                fetching ||
                !modpackVersions
              "
              @click="
                () => {
                  changingVersion = true
                  modpackVersionModal.show()
                }
              "
            >
              <SpinnerIcon v-if="changingVersion" class="animate-spin" />
              <TransferIcon v-else />
              {{
                changingVersion
                  ? formatMessage(messages.installingButton)
                  : formatMessage(messages.changeVersionButton)
              }}
            </button>
          </ButtonStyled>
        </div>
      </template>
    </div>
    <template v-if="!instance.linked_data || !instance.linked_data.locked">
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
        <div v-else class="mt-2 text-brand-red flex gap-2 items-center">
          <IssuesIcon />
          {{ formatMessage(messages.noLoaderVersions, { loader: loader, version: gameVersion }) }}
        </div>
      </template>
      <div class="mt-4 flex flex-wrap gap-2">
        <ButtonStyled color="brand">
          <button
            v-tooltip="
              installing || reinstalling
                ? formatMessage(messages.installInProgress)
                : !isChanged
                  ? formatMessage(
                      loader === 'vanilla'
                        ? messages.alreadyInstalledVanilla
                        : messages.alreadyInstalledModded,
                      {
                        platform: formatCategory(loader),
                        version: instance.loader_version,
                        game_version: gameVersion,
                      },
                    )
                  : repairing
                    ? formatMessage(messages.cannotWhileRepairing, {
                        action: formatMessage(messages.installAction),
                      })
                    : offline
                      ? formatMessage(messages.cannotWhileOffline, {
                          action: formatMessage(messages.installAction),
                        })
                      : null
            "
            :disabled="!isValid || !isChanged || editing || offline || repairing"
            @click="saveGvLoaderEdits()"
          >
            <SpinnerIcon v-if="editing" class="animate-spin" />
            <DownloadIcon v-else />
            {{
              editing
                ? formatMessage(messages.installingButton)
                : formatMessage(messages.installButton)
            }}
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
    </template>
    <template v-else>
      <template v-if="instance.linked_data && instance.linked_data.locked">
        <h2 class="mt-4 mb-1 text-lg font-extrabold text-contrast block">
          {{ formatMessage(messages.unlinkInstanceTitle) }}
        </h2>
        <p class="m-0">
          {{ formatMessage(messages.unlinkInstanceDescription) }}
        </p>
        <ButtonStyled>
          <button class="mt-2" @click="modalConfirmUnpair.show()">
            <UnlinkIcon /> {{ formatMessage(messages.unlinkInstanceButton) }}
          </button>
        </ButtonStyled>
        <template v-if="modpackProject">
          <div>
            <h2 class="m-0 mb-1 text-lg font-extrabold text-contrast block mt-4">
              {{ formatMessage(messages.reinstallModpackTitle) }}
            </h2>
            <p class="m-0">
              {{ formatMessage(messages.reinstallModpackDescription) }}
            </p>
          </div>
          <ButtonStyled color="red" type="outlined">
            <button
              v-tooltip="
                reinstalling
                  ? formatMessage(messages.reinstallingModpackButton)
                  : repairing
                    ? formatMessage(messages.cannotWhileRepairing, {
                        action: formatMessage(messages.reinstallAction),
                      })
                    : installing
                      ? formatMessage(messages.cannotWhileInstalling, {
                          action: formatMessage(messages.reinstallAction),
                        })
                      : offline
                        ? formatMessage(messages.cannotWhileOffline, {
                            action: formatMessage(messages.reinstallAction),
                          })
                        : null
              "
              class="mt-2"
              :disabled="
                changingVersion ||
                repairing ||
                installing ||
                offline ||
                fetching ||
                !modpackVersions
              "
              @click="modalConfirmReinstall.show()"
            >
              <SpinnerIcon v-if="reinstalling" class="animate-spin" />
              <DownloadIcon v-else />
              {{
                reinstalling
                  ? formatMessage(messages.reinstallingModpackButton)
                  : formatMessage(messages.reinstallModpackButton)
              }}
            </button>
          </ButtonStyled>
        </template>
      </template>
    </template>
  </div>
</template>
