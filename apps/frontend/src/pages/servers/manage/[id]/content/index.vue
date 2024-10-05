<template>
  <Modal ref="editModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal header="Edit project" :data="data" @modal="editModal.hide()">
        <UiServersProjectSelect type="modpack" @select="reinstallNew" />
      </UiServersPyroModal>
    </div>
  </Modal>

  <Modal ref="versionSelectModal" header="">
    <div class="flex flex-col gap-4 p-6">
      <div class="flex items-center justify-between gap-4">
        <div class="flex items-center gap-4">
          <div class="text-2xl font-extrabold text-contrast">Select version</div>
        </div>
        <button
          class="h-8 w-8 rounded-full bg-[#ffffff10] p-2 text-contrast"
          @click="versionSelectModal?.hide()"
        >
          <XIcon class="h-4 w-4" />
        </button>
      </div>
      <div class="mb-3 mt-3">
        <p>
          Choose the version of Minecraft you want to use for this server. You can change this
          later.
        </p>
      </div>
      <div class="flex flex-col gap-2">
        <DropdownSelect
          v-model="selectedMCVersion"
          :options="mcVersions"
          placeholder="Select version..."
        />
      </div>
      <div class="mb-4 mt-4 flex justify-end gap-4">
        <Button transparent @click="versionSelectModal?.hide()"> Cancel </Button>
        <Button color="primary" @click="reinstallLoader(selectedLoader)"> Reinstall </Button>
      </div>
    </div>
  </Modal>

  <div class="flex h-full w-full flex-col">
    <div v-if="data && versions" class="flex w-full flex-col gap-6 px-8 py-4">
      <h2 class="m-0 text-3xl font-bold">Modpack</h2>
      <p class="m-0">This is the modpack that is currently installed on your server.</p>
      <div
        v-if="data.upstream"
        class="flex w-full justify-between gap-2 rounded-xl bg-bg-raised p-4"
      >
        <div class="group flex gap-4">
          <div class="relative">
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
              <h1 class="m-0 text-2xl font-extrabold leading-none text-contrast">
                {{ data.project?.title }}
                <span class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand">
                  v{{ currentVersion?.version_number }}
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
            <div class="flex w-[16rem] items-center gap-2">
              <DropdownSelect
                v-if="versions && Array.isArray(versions) && versions.length > 0"
                v-model="version"
                :options="options"
                placeholder="Change version"
              />
              <Button icon-only @click="reinstallCurrent">
                <ChevronRightIcon />
              </Button>
            </div>
          </div>
        </div>
      </div>
      <div
        v-else
        class="flex w-full items-center justify-center rounded-xl bg-button-bg p-2 hover:bg-button-bgActive"
      >
        <div class="flex items-center gap-2">
          <UiServersLoaderIcon v-if="data.loader" :loader="data.loader" class="[&&]:size-10" />
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">{{ data.loader }}</h1>
        </div>
      </div>

      <h2 class="m-0 text-3xl font-bold">Mod Loader</h2>
      <p class="m-0">This is the modpack that is currently installed on your server.</p>
      <div class="flex w-full items-center justify-between rounded-xl bg-bg-raised p-2 pr-4">
        <div class="flex items-center gap-2">
          <Button
            icon-only
            :class="data.loader === 'Vanilla' ? '[&&]:bg-bg-green' : ''"
            @click="selectLoader('Vanilla')"
          >
            <UiServersLoaderIcon
              loader="Vanilla"
              class="[&&]:size-10"
              :class="data.loader === 'Vanilla' ? 'text-brand' : ''"
            />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Vanilla</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button
            icon-only
            :class="data.loader === 'Fabric' ? '[&&]:bg-bg-green' : ''"
            @click="selectLoader('Fabric')"
          >
            <UiServersLoaderIcon
              loader="Fabric"
              class="[&&]:size-10"
              :class="data.loader === 'Fabric' ? 'text-brand' : ''"
            />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Fabric</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button
            icon-only
            :class="data.loader === 'Quilt' ? '[&&]:bg-bg-green' : ''"
            @click="selectLoader('Quilt')"
          >
            <UiServersLoaderIcon
              loader="Quilt"
              class="[&&]:size-10"
              :class="data.loader === 'Quilt' ? 'text-brand' : ''"
            />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Quilt</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button
            icon-only
            :class="data.loader === 'Forge' ? '[&&]:bg-bg-green' : ''"
            @click="selectLoader('Forge')"
          >
            <UiServersLoaderIcon
              loader="Forge"
              class="[&&]:size-10"
              :class="data.loader === 'Forge' ? 'text-brand' : ''"
            />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Forge</h1>
        </div>
        <div class="flex items-center gap-2">
          <Button
            icon-only
            :class="data.loader === 'Neoforge' ? '[&&]:bg-bg-green' : ''"
            @click="selectLoader('Neoforge')"
          >
            <UiServersLoaderIcon
              loader="Neoforge"
              class="[&&]:size-10"
              :class="data.loader === 'Neoforge' ? 'text-brand' : ''"
            />
          </Button>
          <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Neoforge</h1>
        </div>
      </div>
    </div>

    <UiServersPyroLoading v-else />
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect, Button, Modal } from "@modrinth/ui";
import { ChevronRightIcon, EditIcon, XIcon } from "@modrinth/assets";

const route = useNativeRoute();
const serverId = route.params.id as string;
const serverStore = useServerStore();
const tags = useTags();
const prodOverride = await PyroAuthOverride();

const editModal = ref();
const versionSelectModal = ref();

const mcVersions = tags.value.gameVersions
  .filter((x) => x.version_type === "release")
  .map((x) => x.version);

const data = computed(() => serverStore.serverData[serverId]);

const { data: versions } = await useLazyAsyncData(
  `content-loader-versions`,
  () =>
    useBaseFetch(
      `project/${data?.value.upstream?.project_id}/version`,
      {},
      false,
      prodOverride,
    ) as any,
);

const options = computed(() => (versions.value as any[]).map((x) => x.version_number));
const versionIds = computed(() =>
  (versions.value as any[]).map((x) => {
    return { [x.version_number]: x.id };
  }),
);
const version = ref();
const currentVersion = ref();

const selectedLoader = ref("");
const selectedMCVersion = ref("");

const updateData = async () => {
  currentVersion.value = await useBaseFetch(
    `version/${data?.value.upstream?.version_id}`,
    {},
    false,
    prodOverride,
  );
  version.value = currentVersion.value.version_number;
};
updateData();

const reinstallCurrent = async () => {
  const projectId = data.value.upstream?.project_id;
  if (!projectId) {
    throw new Error("Project ID not found");
  }
  const resolvedVersionIds = versionIds.value;
  const versionId = resolvedVersionIds.find((entry: any) => entry[version.value])?.[version.value];
  console.log(projectId, versionId);
  await serverStore.reinstallServer(serverId, false, projectId, versionId);
};

const selectLoader = (loader: string) => {
  selectedLoader.value = loader;
  versionSelectModal.value.show();
};

const reinstallLoader = async (loader: string) => {
  await serverStore.reinstallServer(serverId, true, loader, selectedMCVersion.value);
};

const reinstallNew = async (project: any, versionNumber: string) => {
  editModal.value.hide();
  const versions = (await useBaseFetch(
    `project/${project.project_id}/version`,
    {},
    false,
    prodOverride,
  )) as any;
  const versionId = versions.find((x: any) => x.version_number === versionNumber)?.id;

  if (!versionId) {
    throw new Error("Version not found");
  }

  await serverStore.reinstallServer(serverId, false, project.project_id, versionId);
};
</script>
