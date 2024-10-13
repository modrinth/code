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
      <p>
        Choose the version of Minecraft you want to use for this server. You can change this later.
      </p>
      <div class="flex flex-col gap-2">
        <DropdownSelect
          v-model="selectedMCVersion"
          :options="mcVersions"
          placeholder="Select version..."
        />
      </div>
      <div class="mt-4 flex justify-end gap-4">
        <Button transparent @click="versionSelectModal?.hide()"> Cancel </Button>
        <Button color="primary" @click="reinstallLoader(selectedLoader)"> Reinstall </Button>
      </div>
    </div>
  </Modal>

  <div class="flex h-full w-full flex-col">
    <div v-if="data && versions" class="flex w-full flex-col px-4">
      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-2xl font-bold">Modpack</h2>
          <p class="m-0">The modpack that is currently installed on your server.</p>
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
              <div class="flex w-full max-w-[18rem] items-center gap-2">
                <DropdownSelect
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
        <div
          v-else
          class="flex w-full items-center justify-center rounded-xl bg-button-bg p-2 hover:bg-button-bgActive"
        >
          <div class="flex items-center gap-2">
            <UiServersLoaderIcon :loader="data.loader || 'Vanilla'" class="[&&]:size-10" />
            <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">
              {{ data.loader || "Vanilla" }}
            </h1>
          </div>
        </div>
      </div>

      <div class="card flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <h2 class="m-0 text-2xl font-bold">Mod Loader</h2>
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
                :class="data.loader === 'Vanilla' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Vanilla"
                  class="[&&]:size-10"
                  :class="data.loader === 'Vanilla' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Vanilla</h1>
              <span
                v-if="data.loader === 'Vanilla'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Vanilla')">
              {{ data.loader === "Vanilla" ? "Reinstall" : "Install" }}
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
                :class="data.loader === 'Neoforge' ? '[&&]:bg-bg-green' : ''"
              >
                <UiServersLoaderIcon
                  loader="Neoforge"
                  class="[&&]:size-10"
                  :class="data.loader === 'Neoforge' ? 'text-brand' : ''"
                />
              </div>
              <h1 class="m-0 text-xl font-extrabold leading-none text-contrast">Neoforge</h1>
              <span
                v-if="data.loader === 'Neoforge'"
                class="rounded-full bg-bg-green p-1 px-2 text-sm font-semibold text-brand"
              >
                Current
              </span>
            </div>

            <Button @click="selectLoader('Neoforge')">
              {{ data.loader === "Neoforge" ? "Reinstall" : "Install" }}
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
import { DropdownSelect, Button, Modal } from "@modrinth/ui";
import { ChevronRightIcon, EditIcon, XIcon } from "@modrinth/assets";
import type { Server } from "~/composables/pyroServers";

const route = useNativeRoute();
const serverId = route.params.id as string;

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

const emit = defineEmits<{
  reinstall: [];
}>();

const tags = useTags();
const prodOverride = await PyroAuthOverride();

const editModal = ref();
const versionSelectModal = ref();

const mcVersions = tags.value.gameVersions
  .filter((x) => x.version_type === "release")
  .map((x) => x.version);

const data = computed(() => props.server.general);
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

const reinstallCurrent = async () => {
  const projectId = data.value?.upstream?.project_id;
  if (!projectId) {
    throw new Error("Project ID not found");
  }
  const resolvedVersionIds = versionIds.value;
  const versionId = resolvedVersionIds.find((entry: any) => entry[version.value])?.[version.value];
  // get the [id] url param
  console.log(projectId, versionId);
  await props.server.general?.reinstall(serverId, false, projectId, versionId);
};

const selectLoader = (loader: string) => {
  selectedLoader.value = loader;
  versionSelectModal.value.show();
};

const reinstallLoader = async (loader: string) => {
  await props.server.general?.reinstall(serverId, true, loader, selectedMCVersion.value);
  emit("reinstall");
  if (data.value) {
    data.value.loader = loader;
  }
  await nextTick();
  // scroll to top
  window.scrollTo(0, 0);
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

  await props.server.general?.reinstall(serverId, false, project.project_id, versionId);
};
</script>
