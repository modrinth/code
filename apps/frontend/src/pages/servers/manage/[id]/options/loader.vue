<template>
  <NewModal
    ref="versionSelectModal"
    :header="
      isSecondPhase
        ? 'Confirming reinstallation'
        : `${data?.loader === selectedLoader ? 'Reinstalling' : 'Installing'}
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
        This will reinstall your server and erase all data. You may want to back up your server
        before proceeding. Are you sure you want to continue?
      </p>
      <div v-if="!isSecondPhase" class="flex flex-col gap-4">
        <div class="mx-auto flex flex-row items-center gap-4">
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-button-bg shadow-sm"
          >
            <UiServersIconsLoaderIcon class="size-10" :loader="selectedLoader" />
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
          <div class="text-sm font-bold text-contrast">Minecraft version</div>
          <UiServersTeleportDropdownMenu
            v-model="selectedMCVersion"
            name="mcVersion"
            :options="mcVersions"
            class="w-full max-w-[100%]"
            placeholder="Select Minecraft version..."
          />
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
            <div class="text-sm font-bold text-contrast">{{ selectedLoader }} version</div>

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
                <UiServersIconsLoadingIcon class="mr-2 animate-spin" />
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

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="hard-reset">
              Erase all data
            </label>
            <input
              id="hard-reset"
              :checked="hardReset"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
              @change="hardReset = ($event.target as HTMLInputElement).checked"
            />
          </div>
          <div>
            Removes all data on your server, including your worlds, mods, and configuration files,
            then reinstalls it with the selected version.
          </div>
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="backup-server">
              Backup server
            </label>
            <input
              id="backup-server"
              :checked="backupServer"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
              @change="backupServer = ($event.target as HTMLInputElement).checked"
            />
          </div>
          <div>
            Creates a backup of your server before proceeding with the installation or
            reinstallation.
          </div>
        </div>
      </div>
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button :disabled="canInstall" @click="handleReinstall">
            <RightArrowIcon />
            {{
              isBackingUp
                ? "Backing up..."
                : isSecondPhase
                  ? "Erase and install"
                  : loadingServerCheck
                    ? "Loading..."
                    : isDangerous
                      ? "Erase and install"
                      : "Install"
            }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button
            :disabled="isLoading"
            @click="
              if (isSecondPhase) {
                isSecondPhase = false;
              } else {
                versionSelectModal?.hide();
              }
            "
          >
            <XIcon />
            {{ isSecondPhase ? "Go back" : "Cancel" }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <NewModal ref="mrpackModal" header="Uploading mrpack" @hide="onHide" @show="onShow">
    <div class="flex flex-col gap-4 md:w-[600px]">
      <p
        v-if="isSecondPhase"
        :style="{
          lineHeight: isSecondPhase ? '1.5' : undefined,
          marginBottom: isSecondPhase ? '-12px' : '0',
          marginTop: isSecondPhase ? '-4px' : '-2px',
        }"
      >
        This will reinstall your server and erase all data. You may want to back up your server
        before proceeding. Are you sure you want to continue?
      </p>
      <div v-if="!isSecondPhase" class="flex flex-col gap-4">
        <div class="mx-auto flex flex-row items-center gap-4">
          <div
            class="grid size-16 place-content-center rounded-2xl border-[2px] border-solid border-button-border bg-button-bg shadow-sm"
          >
            <UploadIcon class="size-10" />
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
          <div class="text-sm font-bold text-contrast">Upload mrpack</div>
          <input
            type="file"
            accept=".mrpack"
            class=""
            :disabled="isLoading"
            @change="uploadMrpack"
          />
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="hard-reset">
              Erase all data
            </label>
            <input
              id="hard-reset"
              :checked="hardReset"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
              @change="hardReset = ($event.target as HTMLInputElement).checked"
            />
          </div>
          <div>
            Removes all data on your server, including your worlds, mods, and configuration files,
            then reinstalls it with the selected version.
          </div>
        </div>

        <div class="flex w-full flex-col gap-2 rounded-2xl bg-table-alternateRow p-4">
          <div class="flex w-full flex-row items-center justify-between">
            <label class="w-full text-lg font-bold text-contrast" for="backup-server-mrpack">
              Backup server
            </label>
            <input
              id="backup-server-mrpack"
              :checked="backupServer"
              class="switch stylized-toggle shrink-0"
              type="checkbox"
              @change="backupServer = ($event.target as HTMLInputElement).checked"
            />
          </div>
          <div>Creates a backup of your server before proceeding.</div>
        </div>
      </div>
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button :disabled="canInstallUpload" @click="handleReinstallUpload">
            <RightArrowIcon />
            {{
              isBackingUp
                ? "Backing up..."
                : isSecondPhase
                  ? "Erase and install"
                  : loadingServerCheck
                    ? "Loading..."
                    : isDangerous
                      ? "Erase and install"
                      : "Install"
            }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button
            :disabled="isLoading"
            @click="
              if (isSecondPhase) {
                isSecondPhase = false;
              } else {
                mrpackModal?.hide();
              }
            "
          >
            <XIcon />
            {{ isSecondPhase ? "Go back" : "Cancel" }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <div class="flex h-full w-full flex-col">
    <div v-if="data && versions" class="flex w-full flex-col">
      <div class="card flex flex-col gap-4">
        <div class="flex flex-row items-center justify-between gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Modpack</h2>
          <div v-if="data.upstream" class="flex gap-4">
            <ButtonStyled>
              <nuxt-link
                :class="{
                  'looks-disabled': props.server.general?.status === 'installing' && isError,
                }"
                :to="`/modpacks?sid=${props.server.serverId}`"
              >
                <TransferIcon class="size-4" />
                Change modpack
              </nuxt-link>
            </ButtonStyled>
            <ButtonStyled>
              <button class="!w-full sm:!w-auto" @click="mrpackModal.show()">
                <UploadIcon class="size-4" /> Upload .mrpack file
              </button>
            </ButtonStyled>
            <ButtonStyled v-if="hasNewerVersion" color="brand">
              <button class="!w-full sm:!w-auto" @click="handleUpdateToLatest">
                <UploadIcon class="size-4" /> Update modpack
              </button>
            </ButtonStyled>
          </div>
        </div>
        <div v-if="data.upstream" class="contents">
          <div class="flex w-full justify-between gap-2 rounded-3xl bg-table-alternateRow p-4">
            <div class="flex flex-col gap-4 sm:flex-row">
              <UiAvatar :src="data.project?.icon_url" size="120px" />
              <div class="flex flex-col justify-between">
                <div class="flex flex-col gap-2">
                  <h1 class="m-0 flex gap-2 text-2xl font-extrabold leading-none text-contrast">
                    {{ data.project?.title }}
                  </h1>
                  <span class="text-md text-secondary">
                    {{
                      data.project?.description && data.project.description.length > 150
                        ? data.project.description.substring(0, 150) + "..."
                        : data.project?.description || ""
                    }}
                  </span>
                </div>
              </div>
            </div>
          </div>
          <div class="mt-4 flex flex-col gap-2">
            <h2 class="m-0 text-lg font-bold text-contrast">Change modpack version</h2>
            <p class="m-0">
              Select the version of {{ data.project?.title || "the modpack" }} you want to install
              on your server.
            </p>

            <div class="flex w-full flex-col items-center gap-2">
              <UiServersTeleportDropdownMenu
                v-if="versions && Array.isArray(versions) && versions.length > 0"
                v-model="version"
                :options="options"
                placeholder="Change version"
                name="version"
                class="w-full max-w-full"
              />
              <div class="flex w-full flex-col rounded-2xl bg-table-alternateRow p-4">
                <div class="flex w-full flex-row items-center justify-between">
                  <label class="w-full text-lg font-bold text-contrast" for="modpack-hard-reset">
                    Erase all data
                  </label>
                  <input
                    id="modpack-hard-reset"
                    :checked="hardReset"
                    class="switch stylized-toggle shrink-0"
                    type="checkbox"
                    @change="hardReset = ($event.target as HTMLInputElement).checked"
                  />
                </div>
                <p>
                  If enabled, existing mods, worlds, and configurations, will be deleted before
                  installing the new modpack version.
                </p>
                <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
                  <button
                    :disabled="
                      isLoading || (props.server.general?.status === 'installing' && isError)
                    "
                    class="ml-auto"
                    @click="reinstallCurrent"
                  >
                    <DownloadIcon class="size-4" />
                    {{ isDangerous ? "Erase and install" : "Install" }}
                  </button>
                </ButtonStyled>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="flex w-full flex-col items-center gap-2 sm:w-fit sm:flex-row">
          <ButtonStyled>
            <nuxt-link class="!w-full sm:!w-auto" :to="`/modpacks?sid=${props.server.serverId}`">
              <CompassIcon class="size-4" /> Find a modpack
            </nuxt-link>
          </ButtonStyled>
          <span class="hidden sm:block">or</span>
          <ButtonStyled>
            <button class="!w-full sm:!w-auto" @click="mrpackModal.show()">
              <UploadIcon class="size-4" /> Upload .mrpack file
            </button>
          </ButtonStyled>
        </div>
      </div>

      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Platform</h2>
          <p class="m-0">
            Your server's platform is the software that runs your server. Different platforms
            support different mods and plugins.
          </p>
          <div v-if="data.upstream" class="mt-2 flex items-center gap-2">
            <InfoIcon class="hidden sm:block" />
            <span class="text-sm text-secondary">
              Your server was installed from a modpack, which automatically chooses the appropriate
              platform.
            </span>
          </div>
        </div>
        <div
          class="flex w-full flex-col gap-1 rounded-2xl"
          :class="{
            'pointer-events-none cursor-not-allowed select-none opacity-50':
              props.server.general?.status === 'installing' && isError,
          }"
          :tabindex="props.server.general?.status === 'installing' ? -1 : 0"
        >
          <UiServersLoaderSelector :data="data" @select-loader="selectLoader" />
        </div>
      </div>
    </div>

    <div v-else />
  </div>
</template>

<script setup lang="ts">
import { ButtonStyled, NewModal } from "@modrinth/ui";
import {
  TransferIcon,
  DownloadIcon,
  UploadIcon,
  InfoIcon,
  RightArrowIcon,
  XIcon,
  CompassIcon,
  DropdownIcon,
  ServerIcon,
} from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";
import type { Loaders } from "~/types/servers";

const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const tags = useTags();

const isLoading = ref(false);

const hardReset = ref(false);
const backupServer = ref(false);

const isError = computed(() => props.server.general?.status === "error");
const isDangerous = computed(() => hardReset.value);
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const isBackupLimited = computed(() => (props.server.backups?.data?.length || 0) >= 15);
const isBackingUp = ref(false);

const versionStrings = ["forge", "fabric", "quilt", "neo"] as const;

const loaderVersions = (await Promise.all(
  versionStrings.map(async (loader) => {
    const runFetch = async (iterations: number) => {
      if (iterations > 5) {
        throw new Error("Failed to fetch loader versions");
      }
      try {
        // get our info
        const res = await $fetch(`/loader-versions?loader=${loader}`);
        return { [loader]: (res as any).gameVersions };
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
      } catch (_) {
        return await runFetch(iterations + 1);
      }
    };
    try {
      return await runFetch(0);
    } catch (e) {
      console.error(e);
    }
  }),
).then((res) => res.reduce((acc, val) => ({ ...acc, ...val }), {}))) as Record<
  string,
  {
    // eslint-disable-next-line no-template-curly-in-string
    id: "${modrinth.gameVersion}" | (string & {});
    stable: boolean;
    loaders: {
      id: string;
      url: string;
      stable: boolean;
    }[];
  }[]
>;

const versionSelectModal = ref();
const mrpackModal = ref();

const canInstall = computed(() => {
  const conds =
    !selectedMCVersion.value ||
    isLoading.value ||
    loadingServerCheck.value ||
    serverCheckError.value.trim().length > 0;

  if (selectedLoader.value.toLowerCase() === "vanilla") {
    return conds;
  }

  return conds || !selectedLoaderVersion.value;
});

const canInstallUpload = computed(() => {
  const conds =
    !mrpackFile.value ||
    isLoading.value ||
    loadingServerCheck.value ||
    serverCheckError.value.trim().length > 0;

  return conds;
});

const mcVersions = tags.value.gameVersions
  .filter((x) => x.version_type === "release")
  .map((x) => x.version)
  .filter((x) => {
    // const num = parseInt(x.replace(/\./g, ""), 10);
    // Versions 1.2.4 and below don't have server jars from Mojang
    // return isNaN(num) || num >= 125;
    // above code broke singular versions up until 1.24 (ie 1.25 showed)
    const segment = parseInt(x.split(".")[1], 10);
    return !isNaN(segment) && segment > 2;
  });

const selectedLoaderVersions = computed(() => {
  const loader = selectedLoader.value.toLowerCase();

  if (loader === "paper") {
    return paperVersions.value[selectedMCVersion.value] || [];
  }

  if (loader === "purpur") {
    return purpurVersions.value[selectedMCVersion.value] || [];
  }

  if (loader === "vanilla") {
    return [];
  }

  let apiLoader = loader;
  if (loader === "neoforge") {
    apiLoader = "neo";
  }

  const backwardsCompatibleVersion = loaderVersions[apiLoader]?.find(
    // eslint-disable-next-line no-template-curly-in-string
    (x) => x.id === "${modrinth.gameVersion}",
  );

  if (backwardsCompatibleVersion) {
    return backwardsCompatibleVersion.loaders.map((x) => x.id);
  }

  return (
    loaderVersions[apiLoader]
      ?.find((x) => x.id === selectedMCVersion.value)
      ?.loaders.map((x) => x.id) || []
  );
});

const data = computed(() => props.server.general);
watch(
  () => data.value?.loader,
  () => {
    console.log("Loader:", data.value?.loader);
  },
  {
    deep: true,
    immediate: true,
  },
);
const { data: versions } = data?.value?.upstream
  ? await useLazyAsyncData(
      `content-loader-versions`,
      () => useBaseFetch(`project/${data?.value?.upstream?.project_id}/version`) as any,
    )
  : { data: { value: [] } };

const options = computed(() => (versions?.value as any[]).map((x) => x.version_number));
const versionIds = computed(() =>
  (versions?.value as any[]).map((x) => {
    return { [x.version_number]: x.id };
  }),
);
const version = ref();
const currentVersion = ref();

const selectedLoader = ref<Loaders>("Vanilla");
const selectedMCVersion = ref("");
const selectedLoaderVersion = ref("");
const isSecondPhase = ref(false);

const updateData = async () => {
  if (!data.value?.upstream?.version_id) {
    return;
  }
  currentVersion.value = await useBaseFetch(`version/${data?.value?.upstream?.version_id}`);
  version.value = currentVersion.value.version_number;
};
updateData();

const latestVersion = computed(() => {
  if (!Array.isArray(versions?.value) || versions.value.length === 0) return null;
  return versions.value.reduce((latest: any, current: any) => {
    if (!latest) return current;
    return latest.version_number > current.version_number ? latest : current;
  }, null);
});

const hasNewerVersion = computed(() => {
  if (!currentVersion.value?.version_number || !latestVersion.value?.version_number) return false;
  return latestVersion.value.version_number > currentVersion.value.version_number;
});

const handleUpdateToLatest = async () => {
  if (!latestVersion.value) return;

  version.value = latestVersion.value.version_number;
  hardReset.value = false;
  await reinstallCurrent();
};

const paperVersions = ref<Record<string, number[]>>({});
const purpurVersions = ref<Record<string, string[]>>({});

const fetchPaperVersions = async (mcVersion: string) => {
  try {
    const res = await $fetch(`https://api.papermc.io/v2/projects/paper/versions/${mcVersion}`);
    paperVersions.value[mcVersion] = (res as any).builds.sort((a: number, b: number) => b - a);
    return res;
  } catch (e) {
    console.error(e);
    return null;
  }
};

const fetchPurpurVersions = async (mcVersion: string) => {
  try {
    const res = await $fetch(`https://api.purpurmc.org/v2/purpur/${mcVersion}`);
    purpurVersions.value[mcVersion] = (res as any).builds.all.sort(
      (a: string, b: string) => parseInt(b) - parseInt(a),
    );
    return res;
  } catch (e) {
    console.error(e);
    return null;
  }
};

const selectLoader = (loader: string) => {
  selectedLoader.value = loader as Loaders;
  versionSelectModal.value.show();
};

const loadingServerCheck = ref(false);
const serverCheckError = ref("");

const cachedVersions: Record<string, any> = {};

watch(selectedMCVersion, async () => {
  if (selectedMCVersion.value.trim().length < 3) return;

  isLoading.value = true;
  loadingServerCheck.value = true;

  try {
    // Check if Minecraft version exists
    const mcRes =
      cachedVersions[selectedMCVersion.value] ||
      (await $fetch(`/loader-versions?loader=minecraft&version=${selectedMCVersion.value}`));

    cachedVersions[selectedMCVersion.value] = mcRes;

    if (!mcRes.downloads?.server) {
      serverCheckError.value =
        "We couldn't find a server.jar for this version. Please pick another one.";
      return;
    }

    // Fetch Paper/Purpur versions if needed
    if (selectedLoader.value.toLowerCase() === "paper") {
      const paperRes = await fetchPaperVersions(selectedMCVersion.value);
      if (!paperRes) {
        serverCheckError.value = "This Minecraft version is not supported by Paper.";
        return;
      }
    }

    if (selectedLoader.value.toLowerCase() === "purpur") {
      const purpurRes = await fetchPurpurVersions(selectedMCVersion.value);
      if (!purpurRes) {
        serverCheckError.value = "This Minecraft version is not supported by Purpur.";
        return;
      }
    }

    serverCheckError.value = "";
  } catch (error) {
    console.error(error);
    serverCheckError.value = "Failed to fetch versions. Please try again.";
  } finally {
    loadingServerCheck.value = false;
    isLoading.value = false;
  }
});

const onShow = () => {
  selectedMCVersion.value = props.server.general?.mc_version || "";
  selectedLoaderVersion.value = "";
  hardReset.value = false;
};

const onHide = () => {
  hardReset.value = false;
  backupServer.value = false;
  isSecondPhase.value = false;
  serverCheckError.value = "";
  loadingServerCheck.value = false;
  isLoading.value = false;
  mrpackFile.value = null;
};

const handleReinstallError = (error: any) => {
  if (error instanceof PyroFetchError && error.statusCode === 429) {
    addNotification({
      group: "server",
      title: "Cannot reinstall server",
      text: "You are being rate limited. Please try again later.",
      type: "error",
    });
  } else {
    addNotification({
      group: "server",
      title: "Reinstall Failed",
      text: "An unexpected error occurred while reinstalling. Please try again later.",
      type: "error",
    });
  }
};

const reinstallCurrent = async () => {
  const projectId = data.value?.upstream?.project_id;
  if (!projectId) {
    throw new Error("Project ID not found");
  }
  const resolvedVersionIds = versionIds.value;
  const versionId = resolvedVersionIds.find((entry: any) => entry[version.value])?.[version.value];
  try {
    await props.server.general?.reinstall(
      serverId,
      false,
      projectId,
      versionId,
      undefined,
      hardReset.value,
    );
    emit("reinstall");
  } catch (error) {
    handleReinstallError(error);
  }
};

const handleReinstall = async () => {
  if (hardReset.value && !backupServer.value && !isSecondPhase.value) {
    isSecondPhase.value = true;
    return;
  }

  if (backupServer.value) {
    try {
      const date = new Date();
      const format = date.toLocaleString(navigator.language || "en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "numeric",
        minute: "numeric",
        second: "numeric",
        timeZoneName: "short",
      });
      const backupName = `Reinstallation - ${format}`;
      isLoading.value = true;
      const backupId = (await props.server.backups?.create(backupName)) as unknown as string;

      isBackingUp.value = true;

      let attempts = 0;

      while (true) {
        attempts += 1;

        if (attempts > 100) {
          addNotification({
            group: "server",
            title: "Backup Failed",
            text: "An unexpected error occurred while backing up. Please try again later.",
            type: "error",
          });
          isLoading.value = false;
          return;
        }

        await props.server.refresh(["backups"]);
        const backups = await props.server.backups?.data;
        const backup = backupId ? backups?.find((x) => x.id === backupId) : undefined;
        if (backup && !backup.ongoing) {
          console.log("Backup Finished");
          isBackingUp.value = false;
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 3000));
      }
    } catch {
      addNotification({
        group: "server",
        title: "Backup Failed",
        text: "An unexpected error occurred while backing up. Please try again later.",
        type: "error",
      });
      isLoading.value = false;
      return;
    }
  }

  isLoading.value = true;

  try {
    await props.server.general?.reinstall(
      serverId,
      true,
      selectedLoader.value,
      selectedMCVersion.value,
      selectedLoader.value === "Vanilla" ? "" : selectedLoaderVersion.value,
      hardReset.value,
    );

    emit("reinstall", {
      loader: selectedLoader.value,
      lVersion: selectedLoaderVersion.value,
      mVersion: selectedMCVersion.value,
    });

    await nextTick();
    window.scrollTo(0, 0);
  } catch (error) {
    handleReinstallError(error);
  } finally {
    isLoading.value = false;
    versionSelectModal.value.hide();
  }
};

const mrpackFile = ref<File | null>(null);

const uploadMrpack = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) {
    return;
  }
  mrpackFile.value = target.files[0];
};

const handleReinstallUpload = async () => {
  if (hardReset.value && !backupServer.value && !isSecondPhase.value) {
    isSecondPhase.value = true;
    return;
  }

  if (backupServer.value) {
    try {
      const date = new Date();
      const format = date.toLocaleString(navigator.language || "en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "numeric",
        minute: "numeric",
        second: "numeric",
        timeZoneName: "short",
      });
      const backupName = `Reinstallation - ${format}`;
      isLoading.value = true;
      const backupId = (await props.server.backups?.create(backupName)) as unknown as string;

      isBackingUp.value = true;

      let attempts = 0;

      while (true) {
        attempts += 1;

        if (attempts > 100) {
          addNotification({
            group: "server",
            title: "Backup Failed",
            text: "An unexpected error occurred while backing up. Please try again later.",
            type: "error",
          });
          isLoading.value = false;
          return;
        }

        await props.server.refresh(["backups"]);
        const backups = await props.server.backups?.data;
        const backup = backupId ? backups?.find((x) => x.id === backupId) : undefined;
        if (backup && !backup.ongoing) {
          console.log("Backup Finished");
          isBackingUp.value = false;
          break;
        }
        await new Promise((resolve) => setTimeout(resolve, 3000));
      }
    } catch {
      addNotification({
        group: "server",
        title: "Backup Failed",
        text: "An unexpected error occurred while backing up. Please try again later.",
        type: "error",
      });
      isLoading.value = false;
      return;
    }
  }

  isLoading.value = true;

  try {
    if (!mrpackFile.value) {
      throw new Error("No mrpack file selected");
    }
    const mrpack = new File([mrpackFile.value], mrpackFile.value.name, {
      type: mrpackFile.value.type,
    });

    await props.server.general?.reinstallFromMrpack(mrpack, hardReset.value);

    emit("reinstall", {
      loader: "mrpack",
      lVersion: "",
      mVersion: "",
    });

    await nextTick();
    window.scrollTo(0, 0);
  } catch (error) {
    handleReinstallError(error);
  } finally {
    isLoading.value = false;
    mrpackModal.value.hide();
  }
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
