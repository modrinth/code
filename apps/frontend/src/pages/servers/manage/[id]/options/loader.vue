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
            ? "This will reinstall your server and erase all data. You may want to back up your server before proceeding. Are you sure you want to continue?"
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
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button :disabled="canInstall" @click="handleReinstall">
            <RightArrowIcon />
            {{
              isSecondPhase
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
            {{ isSecondPhase ? "No" : "Cancel" }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <NewModal ref="mrpackModal" header="Upload mrpack" @hide="onHide" @show="onShow">
    <div>
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
      <input
        type="file"
        accept=".mrpack"
        class="mt-4"
        :disabled="isLoading"
        @change="uploadMrpack"
      />
      <div class="mt-4 flex justify-start gap-4">
        <ButtonStyled :color="isDangerous ? 'red' : 'brand'">
          <button :disabled="!mrpackFile || isLoading" @click="reinstallMrpack">
            <RightArrowIcon />
            {{
              isSecondPhase
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
          <button :disabled="isLoading" @click="mrpackModal?.hide">
            <XIcon />
            Cancel
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
          </div>
        </div>
        <div
          v-if="data.upstream"
          class="flex w-full justify-between gap-2 rounded-3xl bg-table-alternateRow p-4"
        >
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
              <div
                class="mt-2 flex w-full max-w-[24rem] flex-col items-center gap-2 sm:mt-0 sm:flex-row"
              >
                <UiServersTeleportDropdownMenu
                  v-if="versions && Array.isArray(versions) && versions.length > 0"
                  v-model="version"
                  :options="options"
                  placeholder="Change version"
                  name="version"
                />
                <ButtonStyled>
                  <button
                    :disabled="
                      isLoading || (props.server.general?.status === 'installing' && isError)
                    "
                    class="!w-full sm:!w-auto"
                    @click="reinstallCurrent"
                  >
                    <DownloadIcon class="size-4" />
                    Reinstall
                  </button>
                </ButtonStyled>
              </div>
            </div>
          </div>
        </div>
        <div v-else class="flex w-full flex-col items-center gap-2 sm:w-fit sm:flex-row">
          <ButtonStyled>
            <nuxt-link class="!w-full sm:!w-auto" :to="`/modpacks?sid=${props.server.serverId}`">
              <DownloadIcon class="size-4" /> Install a modpack
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
          <h2 class="m-0 text-lg font-bold text-contrast">Mod loader</h2>
          <p class="m-0">Mod loaders allow you to run mods on your server.</p>
          <div v-if="data.upstream" class="mt-2 flex items-center gap-2">
            <InfoIcon class="hidden sm:block" />
            <span class="text-sm text-secondary">
              Your server was installed from a modpack, which automatically chooses the appropriate
              mod loader.
            </span>
          </div>
        </div>
        <div
          class="flex w-full flex-col gap-1 rounded-2xl bg-table-alternateRow p-2"
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

    <UiServersPyroLoading v-else />
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
} from "@modrinth/assets";
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

const isLoading = ref(false);

const hardReset = ref(false);
const backupServer = ref(false);

const isError = computed(() => props.server.general?.status === "error");
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
const mrpackModal = ref();

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

const selectedLoader = ref("");
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
    await props.server.general?.reinstall(serverId, false, projectId, versionId);
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
    const versions = (await useBaseFetch(`project/${project.project_id}/version`)) as any;
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

const mrpackFile = ref<File | null>(null);

const uploadMrpack = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files || target.files.length === 0) {
    return;
  }
  mrpackFile.value = target.files[0];
};

const reinstallMrpack = async () => {
  if (!mrpackFile.value) {
    return;
  }

  const mrpack = new File([mrpackFile.value], mrpackFile.value.name, {
    type: mrpackFile.value.type,
  });

  try {
    isLoading.value = true;
    await props.server.general?.reinstallFromMrpack(mrpack, hardReset.value);
    emit("reinstall");
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
