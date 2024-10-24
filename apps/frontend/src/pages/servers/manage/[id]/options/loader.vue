<template>
  <NewModal ref="editModal" header="Select modpack">
    <UiServersProjectSelect type="modpack" @select="reinstallNew" />
  </NewModal>

  <NewModal
    ref="versionSelectModal"
    :header="isSecondPhase ? 'Confirm reinstallation' : 'Select version'"
    @hide="onHide"
    @show="onShow"
  >
    <div class="flex flex-col gap-4 md:w-[600px]">
      <p
        :style="{
          lineHeight: isSecondPhase ? '1.5' : undefined,
          marginBottom: isSecondPhase ? '-12px' : '0',
          marginTop: isSecondPhase ? '-4px' : '-2px',
        }"
      >
        {{
          isSecondPhase
            ? "You are attempting to delete all of your files without backing up. Are you sure this is what you're intending to do?"
            : "Choose the version of Minecraft you want to use for this server."
        }}
      </p>
      <div v-if="!isSecondPhase" class="flex flex-col gap-2">
        <UiServersTeleportDropdownMenu
          v-model="selectedMCVersion"
          name="mcVersion"
          :options="mcVersions"
          placeholder="Select Minecraft version..."
        />
        <UiServersTeleportDropdownMenu
          v-if="selectedMCVersion && selectedLoader.toLowerCase() !== 'vanilla'"
          v-model="selectedLoaderVersion"
          name="loaderVersion"
          :options="selectedLoaderVersions"
          placeholder="Select loader version..."
        />
        <div class="mt-2 flex items-center gap-2">
          <input
            id="hard-reset"
            :checked="hardReset"
            class="switch stylized-toggle"
            type="checkbox"
            @change="hardReset = ($event.target as HTMLInputElement).checked"
          />
          <label for="hard-reset">Clean reinstall</label>
        </div>
      </div>
      <div class="mt-4 flex justify-end gap-4">
        <Button
          transparent
          :disabled="isLoading"
          @click="
            if (isSecondPhase) {
              isSecondPhase = false;
            } else {
              versionSelectModal?.hide();
            }
          "
        >
          {{ isSecondPhase ? "No" : "Cancel" }}
        </Button>
        <Button
          :color="isDangerous ? 'danger' : 'primary'"
          :disabled="canInstall"
          @click="handleReinstall"
        >
          {{
            isSecondPhase
              ? "Yes"
              : loadingServerCheck
                ? "Loading..."
                : isDangerous
                  ? "Erase and install"
                  : "Install"
          }}
        </Button>
      </div>
    </div>
  </NewModal>

  <div class="flex h-full w-full flex-col">
    <div v-if="data && versions" class="flex w-full flex-col">
      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Modpack</h2>
        </div>
        <div
          v-if="data.upstream"
          class="flex w-full justify-between gap-2 rounded-xl bg-table-alternateRow p-4"
        >
          <div class="flex gap-4">
            <div v-tooltip="'Change the installed modpack'" class="group relative">
              <UiAvatar :src="data.project?.icon_url" size="120px" />
              <div
                class="absolute top-0 hidden size-[120px] flex-col items-center justify-center rounded-xl bg-button-bg p-2 opacity-80 group-hover:flex"
                @click="editModal.show()"
              >
                <EditIcon class="h-8 w-8 text-contrast" />
              </div>
            </div>
            <div class="flex flex-col justify-between">
              <div class="flex flex-col gap-2">
                <h1 class="m-0 flex gap-2 text-2xl font-extrabold leading-none text-contrast">
                  {{ data.project?.title }}
                  <span
                    v-tooltip="'Current installed Modpack version'"
                    class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
                  >
                    {{ currentVersion?.version_number }}
                  </span>
                </h1>
                <span class="text-md font-semibold text-secondary">
                  {{
                    data.project?.description && data.project.description.length > 150
                      ? data.project.description.substring(0, 150) + "..."
                      : data.project?.description || ""
                  }}
                </span>
              </div>
              <div class="flex w-full max-w-[18rem] items-center gap-2">
                <UiServersTeleportDropdownMenu
                  v-if="versions && Array.isArray(versions) && versions.length > 0"
                  v-model="version"
                  :options="options"
                  placeholder="Change version"
                  name="version"
                />
                <Button
                  v-tooltip="'Reinstall current pack with selected version'"
                  icon-only
                  @click="reinstallCurrent"
                >
                  <div class="mx-4">Reinstall</div>
                </Button>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="flex items-center gap-2">
          <Button @click="editModal.show()">
            <DownloadIcon class="size-4" /> Install modpack
          </Button>
          or
          <Button> <UploadIcon class="size-4" /> Upload mrpack </Button>
        </div>
      </div>

      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-lg font-bold text-contrast">Mod loader</h2>
          <p class="m-0">
            The mod loader that provides the ability to load mods into your server. Changing this
            will reinstall the server.
          </p>
        </div>
        <div class="flex w-full flex-col gap-4 rounded-xl bg-table-alternateRow p-4 pr-4">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div
                class="rounded-xl bg-button-bg p-2"
                :class="data.loader === 'Vanilla' || !data.loader ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Vanilla"
                  class="[&&]:size-10"
                  :class="data.loader === 'Vanilla' || !data.loader ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Vanilla</h1>
              <span
                v-if="data.loader === 'Vanilla' || !data.loader"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Vanilla')">
              {{ data.loader === "Vanilla" || !data.loader ? "Reinstall" : "Install" }}
              <ChevronRightIcon />
            </Button>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div
                class="rounded-xl bg-button-bg p-2"
                :class="data.loader === 'Fabric' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Fabric"
                  class="[&&]:size-10"
                  :class="data.loader === 'Fabric' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Fabric</h1>
              <span
                v-if="data.loader === 'Fabric'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Fabric')">
              {{ data.loader === "Fabric" ? "Reinstall" : "Install" }}
              <ChevronRightIcon />
            </Button>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div
                class="rounded-xl bg-button-bg p-2"
                :class="data.loader === 'Quilt' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Quilt"
                  class="[&&]:size-10"
                  :class="data.loader === 'Quilt' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Quilt</h1>
              <span
                v-if="data.loader === 'Quilt'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Quilt')">
              {{ data.loader === "Quilt" ? "Reinstall" : "Install" }}
              <ChevronRightIcon />
            </Button>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div
                class="rounded-xl bg-button-bg p-2"
                :class="data.loader === 'Forge' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Forge"
                  class="[&&]:size-10"
                  :class="data.loader === 'Forge' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Forge</h1>
              <span
                v-if="data.loader === 'Forge'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Forge')">
              {{ data.loader === "Forge" ? "Reinstall" : "Install" }}
              <ChevronRightIcon />
            </Button>
          </div>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <div
                class="rounded-xl bg-button-bg p-2"
                :class="data.loader?.toLowerCase() === 'neoforge' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="NeoForge"
                  class="[&&]:size-10"
                  :class="data.loader?.toLowerCase() === 'neoforge' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">NeoForge</h1>
              <span
                v-if="data.loader?.toLowerCase() === 'neoforge'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('NeoForge')">
              {{ data.loader?.toLowerCase() === "neoforge" ? "Reinstall" : "Install" }}
              <ChevronRightIcon />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { Button, NewModal } from "@modrinth/ui";
import { ChevronRightIcon, DownloadIcon, EditIcon, UploadIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  reinstall: [any?];
}>();

const tags = useTags();
const prodOverride = await PyroAuthOverride();

const isLoading = ref(false);

const hardReset = ref(false);
const backupServer = ref(false);

const isDangerous = computed(() => hardReset.value);
const isBackupLimited = computed(() => (props.server.backups?.data?.length || 0) >= 15);

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

const editModal = ref();
const versionSelectModal = ref();

const canInstall = computed(() => {
  const conds =
    !selectedMCVersion.value ||
    isBackupLimited.value ||
    isLoading.value ||
    loadingServerCheck.value ||
    serverCheckError.value.trim().length > 0;

  if (selectedLoader.value.toLowerCase() === "vanilla") {
    return conds;
  }

  return conds || !selectedLoaderVersion.value;
});

const mcVersions = tags.value.gameVersions
  .filter((x) => x.version_type === "release")
  .map((x) => x.version);

const selectedLoaderVersions = computed(() => {
  /*
      loaderVersions[
      selectedLoader.value.toLowerCase() === "neoforge" ? "neo" : selectedLoader.toLowerCase()
    ]
      .find((x) => x.id === selectedMCVersion)
      ?.loaders.map((x) => x.id) || []
      */
  let loader = selectedLoader.value.toLowerCase();
  if (loader === "neoforge") {
    loader = "neo";
  }
  const backwardsCompatibleVersion = loaderVersions[loader].find(
    // eslint-disable-next-line no-template-curly-in-string
    (x) => x.id === "${modrinth.gameVersion}",
  );
  if (backwardsCompatibleVersion) {
    return backwardsCompatibleVersion.loaders.map((x) => x.id);
  }
  return (
    loaderVersions[loader]
      .find((x) => x.id === selectedMCVersion.value)
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
      () =>
        useBaseFetch(
          `project/${data?.value?.upstream?.project_id}/version`,
          {},
          false,
          prodOverride,
        ) as any,
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

const selectedLoader = ref("");
const selectedMCVersion = ref("");
const selectedLoaderVersion = ref("");
const isSecondPhase = ref(false);

const updateData = async () => {
  if (!data.value?.upstream?.version_id) {
    return;
  }
  currentVersion.value = await useBaseFetch(
    `version/${data?.value?.upstream?.version_id}`,
    {},
    false,
    prodOverride,
  );
  version.value = currentVersion.value.version_number;
};
updateData();

const selectLoader = (loader: string) => {
  selectedLoader.value = loader;
  versionSelectModal.value.show();
};

const loadingServerCheck = ref(false);
const serverCheckError = ref("");

const cachedVersions: Record<string, any> = {};

watch(selectedMCVersion, async () => {
  if (selectedMCVersion.value.trim().length < 3) return;
  // const res = await fetch(
  //   `/loader-versions?loader=minecraft&version=${selectedMCVersion.value}`,
  // ).then((r) => r.json());

  loadingServerCheck.value = true;
  const res =
    cachedVersions[selectedMCVersion.value] ||
    (await $fetch(`/loader-versions?loader=minecraft&version=${selectedMCVersion.value}`));

  cachedVersions[selectedMCVersion.value] = res;

  loadingServerCheck.value = false;

  if (res.downloads.server) {
    serverCheckError.value = "";
  } else {
    serverCheckError.value =
      "We couldn't find a server.jar for this version. Please pick another one.";
  }
});

const onShow = () => {
  selectedMCVersion.value = "";
  selectedLoaderVersion.value = "";
};

const onHide = () => {
  hardReset.value = false;
  backupServer.value = false;
  isSecondPhase.value = false;
  serverCheckError.value = "";
  loadingServerCheck.value = false;
  isLoading.value = false;
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
    await props.server.general?.reinstall(serverId, false, projectId, versionId);
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
      await props.server.backups?.create(backupName);
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

const reinstallNew = async (project: any, versionNumber: string) => {
  editModal.value.hide();
  try {
    const versions = (await useBaseFetch(
      `project/${project.project_id}/version`,
      {},
      false,
      prodOverride,
    )) as any;
    const version = versions.find((x: any) => x.version_number === versionNumber);

    if (!version?.id) {
      throw new Error("Version not found");
    }
    await props.server.general?.reinstall(serverId, false, project.project_id, version.id);
    emit("reinstall");
    await nextTick();
    window.scrollTo(0, 0);
  } catch (error) {
    handleReinstallError(error);
  }
};
</script>

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
