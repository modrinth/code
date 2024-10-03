<template>
  <Modal ref="editModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal @modal="editModal.hide()" header="Edit project" :data="data">
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
    <div v-if="data && versions" class="flex w-full flex-col gap-6 p-8">
      <div class="flex w-full justify-between gap-2 rounded-xl bg-bg-raised p-4">
        <div class="flex gap-4">
          <UiAvatar :src="data.project.icon_url" size="120px" />
          <div class="flex flex-col justify-between">
            <div class="flex flex-col gap-2">
              <h1 class="m-0 text-2xl font-extrabold leading-none text-contrast">
                {{ data.project.title }}
                <span class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand">
                  v{{ currentVersion?.version_number }}
                </span>
              </h1>
              <span class="text-md font-semibold text-secondary">
                {{
                  data.project.description.length > 150
                    ? data.project.description.substring(0, 150) + "..."
                    : data.project.description
                }}
              </span>
            </div>
            <div class="flex w-[16rem] items-center gap-2">
              <DropdownSelect
                v-if="versions.length > 0"
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
        <div
          class="flex flex-col items-center justify-center rounded-xl bg-button-bg p-2 hover:bg-button-bgActive"
          @click="editModal.show()"
        >
          <EditIcon class="h-6 w-6 text-contrast" />
        </div>
      </div>
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
const config = useRuntimeConfig();
const prodOverride = await PyroAuthOverride();

const editModal = ref();
const versionSelectModal = ref();

const mcVersions = tags.value.gameVersions
  .filter((x) => x.version_type === "release")
  .map((x) => x.version);

const data = ref();
const versions = ref();
const options = ref();
const version = ref();
const currentVersion = ref();
const version_ids = ref();

const selectedLoader = ref("");
const selectedMCVersion = ref("");

const updateData = async () => {
  await serverStore.fetchServerData(serverId);
  const d = await serverStore.getServerData(serverId);
  data.value = d;
  const v = (await useBaseFetch(
    `project/${d?.upstream?.project_id}/version`,
    {},
    false,
    prodOverride,
  )) as any;
  versions.value = v;
  version_ids.value = v.map((x: any) => {
    return { [x.version_number]: x.id };
  });
  options.value = v.map((x: any) => x.version_number);
  currentVersion.value = await useBaseFetch(
    `version/${d?.upstream?.version_id}`,
    {},
    false,
    prodOverride,
  );
  version.value = currentVersion.value.version_number;
};
updateData();

const reinstallCurrent = async () => {
  const projectId = data.value.upstream.project_id;
  const versionId = version_ids.value[0][version.value];
  console.log(projectId, versionId);
  await serverStore.reinstallServer(serverId, false, projectId, versionId);
};

const selectLoader = async (loader: string) => {
  selectedLoader.value = loader;
  versionSelectModal.value.show();
};

const reinstallLoader = async (loader: string) => {
  await serverStore.reinstallServer(serverId, true, loader, selectedMCVersion.value);
};

const reinstallNew = async (project_id: string, version_number: string) => {
  editModal.value.hide();
  const versions = (await useBaseFetch(
    `project/${project_id}/version`,
    {},
    false,
    prodOverride,
  )) as any;
  console.log(version_number);
  const version_id = versions.find((x: any) => x.version_number === version_number)?.id;
  console.log(version_id);

  if (!version_id) {
    throw new Error("Version not found");
  }

  await serverStore.reinstallServer(serverId, false, project_id, version_id);
};
</script>
