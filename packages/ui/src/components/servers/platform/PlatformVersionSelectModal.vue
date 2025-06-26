<template>
  <NewModal
    ref="versionSelectModal"
    :header="
      isSecondPhase
        ? 'Confirming reinstallation'
        : `${props.currentLoader === selectedLoader ? 'Reinstalling' : 'Installing'}
      ${selectedLoader.toLowerCase() === 'vanilla' ? 'Vanilla Minecraft' : selectedLoader}`
    "
    @hide="onHide"
    @show="onShow"
  >
    <div class="flex flex-col gap-4 md:w-[600px]">
      <p
        v-if="isSecondPhase"
        :style="{
          lineHeight: isSecondPhase ? '1.5' : undefined,
          marginBottom: isSecondPhase ? '-12px' : '0',
          marginTop: isSecondPhase ? '-4px' : '-2px',
        }"
      >
        {{
          'This will reinstall your server and erase all data. Are you sure you want to continue?'
        }}
      </p>
      <div v-if="!isSecondPhase" class="flex flex-col gap-4">
        <div class="mx-auto flex flex-row items-center gap-4">
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-button-bg shadow-sm"
          >
            <LoaderIcon class="size-10" :loader="selectedLoader" />
          </div>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            class="size-10"
          >
            <path d="M5 9v6" />
            <path d="M9 9h3V5l7 7-7 7v-4H9V9z" />
          </svg>
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-table-alternateRow shadow-sm"
          >
            <ServerIcon class="size-10" />
          </div>
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="text-lg font-bold text-contrast">Minecraft version</div>
          <UiServersTeleportDropdownMenu
            v-model="selectedMCVersion"
            name="mcVersion"
            :options="mcVersions"
            class="w-full max-w-[100%]"
            placeholder="Select Minecraft version..."
          />
          <div class="mt-2 flex items-center justify-between gap-2">
            <label for="toggle-snapshots" class="font-semibold"> Show snapshot versions </label>
            <div
              v-tooltip="
                isSnapshotSelected ? 'A snapshot version is currently selected.' : undefined
              "
            >
              <Toggle
                id="toggle-snapshots"
                v-model="showSnapshots"
                :disabled="isSnapshotSelected"
              />
            </div>
          </div>
        </div>

        <div
          v-if="selectedLoader.toLowerCase() !== 'vanilla'"
          class="flex w-full flex-col gap-2 rounded-2xl p-4"
          :class="{
            'bg-table-alternateRow':
              !selectedMCVersion || isLoading || selectedLoaderVersions.length > 0,
            'bg-highlight-red':
              selectedMCVersion && !isLoading && selectedLoaderVersions.length === 0,
          }"
        >
          <div class="flex flex-col gap-2">
            <div class="text-lg font-bold text-contrast">{{ selectedLoader }} version</div>

            <template v-if="!selectedMCVersion">
              <div
                class="relative flex h-9 w-full select-none items-center rounded-xl bg-button-bg px-4 opacity-50"
              >
                Select a Minecraft version to see available versions
                <DropdownIcon class="absolute right-4" />
              </div>
            </template>
            <template v-else-if="isLoading">
              <div
                class="relative flex h-9 w-full items-center rounded-xl bg-button-bg px-4 opacity-50"
              >
                <RefreshClockwiseIcon class="mr-2 animate-spin" />
                Loading versions...
                <DropdownIcon class="absolute right-4" />
              </div>
            </template>
            <template v-else-if="selectedLoaderVersions.length > 0">
              <UiServersTeleportDropdownMenu
                v-model="selectedLoaderVersion"
                name="loaderVersion"
                :options="selectedLoaderVersions"
                class="w-full max-w-[100%]"
                :placeholder="
                  selectedLoader.toLowerCase() === 'paper' ||
                  selectedLoader.toLowerCase() === 'purpur'
                    ? `Select build number...`
                    : `Select loader version...`
                "
              />
            </template>
            <template v-else>
              <div>No versions available for Minecraft {{ selectedMCVersion }}.</div>
            </template>
          </div>
        </div>

        <div
          v-if="!initialSetup"
          class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4"
        >
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="hard-reset">
              Erase all data
            </label>
            <input
              id="hard-reset"
              v-model="hardReset"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
            />
          </div>
          <div>
            Removes all data on your server, including your worlds, mods, and configuration files,
            then reinstalls it with the selected version.
          </div>
          <div class="font-bold">This does not affect your backups, which are stored off-site.</div>
        </div>

        <BackupWarning
          v-if="!initialSetup"
          :backup-link="`/servers/manage/${props.server?.serverId}/backups`"
        />
      </div>

      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button
            v-tooltip="backupInProgress ? formatMessage(backupInProgress.tooltip) : undefined"
            :disabled="canInstall || !!backupInProgress"
            @click="handleReinstall"
          >
            <RightArrowIcon />
            {{
              isLoading
                ? 'Installing...'
                : isSecondPhase
                  ? 'Erase and install'
                  : hardReset
                    ? 'Continue'
                    : 'Install'
            }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button :disabled="isLoading" @click="handleCancel">
            <XIcon />
            {{ isSecondPhase ? 'Go back' : 'Cancel' }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import {
  BackupWarning,
  ButtonStyled,
  LoaderIcon,
  NewModal,
  Toggle,
  injectNotificationManager,
  type ModrinthServer,
  type LoaderTag,
} from '@modrinth/ui'
import {
  DropdownIcon,
  RefreshClockwiseIcon,
  RightArrowIcon,
  ServerIcon,
  XIcon,
} from '@modrinth/assets'
import { $fetch } from 'ofetch'
import {
  type Loaders,
  ModrinthServersFetchError,
  type BackupInProgressReason,
} from '@modrinth/utils'
import { useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()

interface MinecraftVersion {
  downloads?: {
    server?: string
  }
}

interface PaperApiResponse {
  builds: number[]
}

interface PurpurApiResponse {
  builds: {
    all: string[]
  }
}

interface LoaderManifest {
  gameVersions: string[]
}

type VersionCache = Record<string, MinecraftVersion>

const props = defineProps<{
  server: ModrinthServer
  currentLoader: Loaders | undefined
  backupInProgress?: BackupInProgressReason
  initialSetup?: boolean
  tags: {
    loaders: LoaderTag[]
    gameVersions: Array<{ version: string; version_type: string }>
  }
}>()

const emit = defineEmits<{
  reinstall: [{ loader: string; lVersion: string; mVersion: string }?]
}>()

const { addNotification } = injectNotificationManager()

const versionSelectModal = ref<InstanceType<typeof NewModal>>()
const isSecondPhase = ref<boolean>(false)
const hardReset = ref<boolean>(false)
const isLoading = ref<boolean>(false)
const loadingServerCheck = ref<boolean>(false)
const serverCheckError = ref<string>('')
const showSnapshots = ref<boolean>(false)

const selectedLoader = ref<Loaders>('Vanilla')
const selectedMCVersion = ref<string>('')
const selectedLoaderVersion = ref<string>('')

const paperVersions = ref<Record<string, number[]>>({})
const purpurVersions = ref<Record<string, string[]>>({})
const gameVersionsCache = ref<Record<string, string[]>>({})
const cachedVersions = ref<VersionCache>({})

const versionStrings = ['forge', 'fabric', 'quilt', 'neo'] as const

const isSnapshotSelected = computed(() => {
  if (selectedMCVersion.value) {
    const selected = props.tags.gameVersions.find((x) => x.version === selectedMCVersion.value)
    if (selected?.version_type !== 'release') {
      return true
    }
  }
  return false
})

async function getLoaderVersions(loader: string): Promise<LoaderManifest> {
  return await $fetch(
    `https://launcher-meta.modrinth.com/${loader?.toLowerCase()}/v0/manifest.json`,
  )
}

async function fetchLoaderVersions(): Promise<void> {
  const versions = await Promise.all(
    versionStrings.map(async (loader) => {
      const runFetch = async (iterations: number): Promise<Record<string, string[]>> => {
        if (iterations > 5) {
          throw new Error('Failed to fetch loader versions')
        }
        try {
          const res = await getLoaderVersions(loader)
          return { [loader]: res.gameVersions }
          // eslint-disable-next-line @typescript-eslint/no-unused-vars
        } catch (_) {
          return await runFetch(iterations + 1)
        }
      }
      try {
        return await runFetch(0)
      } catch (e) {
        console.error(e)
        return { [loader]: [] }
      }
    }),
  )

  gameVersionsCache.value = versions.reduce((acc, val) => ({ ...acc, ...val }), {})
}

async function fetchPaperVersions(mcVersion: string): Promise<PaperApiResponse | null> {
  try {
    const res = await $fetch<PaperApiResponse>(
      `https://api.papermc.io/v2/projects/paper/versions/${mcVersion}`,
    )
    paperVersions.value[mcVersion] = res.builds.sort((a: number, b: number) => b - a)
    return res
  } catch (e) {
    console.error(e)
    return null
  }
}

async function fetchPurpurVersions(mcVersion: string): Promise<PurpurApiResponse | null> {
  try {
    const res = await $fetch<PurpurApiResponse>(`https://api.purpurmc.org/v2/purpur/${mcVersion}`)
    purpurVersions.value[mcVersion] = res.builds.all.sort(
      (a: string, b: string) => parseInt(b) - parseInt(a),
    )
    return res
  } catch (e) {
    console.error(e)
    return null
  }
}

const selectedLoaderVersions = computed<string[]>(() => {
  const loader = selectedLoader.value.toLowerCase()

  if (loader === 'paper') {
    return paperVersions.value[selectedMCVersion.value]?.map((x) => `${x}`) || []
  }

  if (loader === 'purpur') {
    return purpurVersions.value[selectedMCVersion.value] || []
  }

  if (loader === 'vanilla') {
    return []
  }

  let apiLoader = loader
  if (loader === 'neoforge') {
    apiLoader = 'neo'
  }
  const supportedVersions = gameVersionsCache.value[apiLoader] || []
  return supportedVersions.includes(selectedMCVersion.value) ? ['latest'] : []
})

watch(selectedLoader, async () => {
  if (selectedMCVersion.value) {
    selectedLoaderVersion.value = ''
    serverCheckError.value = ''

    await checkVersionAvailability(selectedMCVersion.value)
  }
})

watch(
  selectedLoaderVersions,
  (newVersions) => {
    if (
      newVersions.length > 0 &&
      (!selectedLoaderVersion.value || !newVersions.includes(selectedLoaderVersion.value))
    ) {
      selectedLoaderVersion.value = String(newVersions[0])
    }
  },
  { immediate: true },
)

async function getLoaderVersion(loader: string, version: string): Promise<MinecraftVersion> {
  return await $fetch(
    `https://launcher-meta.modrinth.com/${loader?.toLowerCase()}/v0/versions/${version}.json`,
  )
}

async function checkVersionAvailability(version: string): Promise<void> {
  if (!version || version.trim().length < 3) return

  isLoading.value = true
  loadingServerCheck.value = true

  try {
    const mcRes = cachedVersions.value[version] || (await getLoaderVersion('minecraft', version))

    cachedVersions.value[version] = mcRes

    if (!mcRes.downloads?.server) {
      serverCheckError.value = "We couldn't find a server.jar for this version."
      return
    }

    const loader = selectedLoader.value.toLowerCase()
    if (loader === 'paper' || loader === 'purpur') {
      const fetchFn = loader === 'paper' ? fetchPaperVersions : fetchPurpurVersions
      const result = await fetchFn(version)
      if (!result) {
        serverCheckError.value = `This Minecraft version is not supported by ${loader}.`
        return
      }
    }

    serverCheckError.value = ''
  } catch (error) {
    console.error(error)
    serverCheckError.value = 'Failed to fetch versions.'
  } finally {
    loadingServerCheck.value = false
    isLoading.value = false
  }
}

watch(selectedMCVersion, checkVersionAvailability)

onMounted(() => {
  fetchLoaderVersions()
})

const mcVersions = computed(() =>
  props.tags.gameVersions
    .filter((x) =>
      showSnapshots.value
        ? x.version_type === 'snapshot' || x.version_type === 'release'
        : x.version_type === 'release',
    )
    .map((x) => x.version),
)

const isDangerous = computed(() => hardReset.value)
const canInstall = computed(() => {
  const conds =
    !selectedMCVersion.value ||
    isLoading.value ||
    loadingServerCheck.value ||
    serverCheckError.value.trim().length > 0

  if (selectedLoader.value.toLowerCase() === 'vanilla') {
    return conds
  }

  return conds || !selectedLoaderVersion.value
})

async function handleReinstall(): Promise<void> {
  if (hardReset.value && !isSecondPhase.value) {
    isSecondPhase.value = true
    return
  }

  isLoading.value = true

  try {
    await props.server.general?.reinstall(
      true,
      selectedLoader.value,
      selectedMCVersion.value,
      selectedLoader.value === 'Vanilla' ? '' : selectedLoaderVersion.value,
      props.initialSetup ? true : hardReset.value,
    )

    emit('reinstall', {
      loader: selectedLoader.value,
      lVersion: selectedLoaderVersion.value,
      mVersion: selectedMCVersion.value,
    })

    hide()
  } catch (error) {
    if (
      error instanceof ModrinthServersFetchError &&
      'statusCode' in error &&
      error.statusCode === 429
    ) {
      addNotification({
        title: 'Cannot reinstall server',
        text: 'You are being rate limited. Please try again later.',
        type: 'error',
      })
    } else {
      addNotification({
        title: 'Reinstall Failed',
        text: 'An unexpected error occurred while reinstalling. Please try again later.',
        type: 'error',
      })
    }
  } finally {
    isLoading.value = false
  }
}

function handleCancel(): void {
  if (isSecondPhase.value) {
    isSecondPhase.value = false
  } else {
    hide()
  }
}

function onShow(): void {
  selectedMCVersion.value = props.server.general?.mc_version || ''
  if (isSnapshotSelected.value) {
    showSnapshots.value = true
  }
}

function onHide(): void {
  hardReset.value = false
  isSecondPhase.value = false
  serverCheckError.value = ''
  loadingServerCheck.value = false
  isLoading.value = false
  selectedMCVersion.value = ''
  serverCheckError.value = ''
  paperVersions.value = {}
  purpurVersions.value = {}
}

function show(loader: Loaders): void {
  if (selectedLoader.value !== loader) {
    selectedLoaderVersion.value = ''
  }
  selectedLoader.value = loader
  selectedMCVersion.value = props.server.general?.mc_version || ''
  versionSelectModal.value?.show()
}

function hide(): void {
  versionSelectModal.value?.hide()
}

defineExpose({ show, hide })
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
