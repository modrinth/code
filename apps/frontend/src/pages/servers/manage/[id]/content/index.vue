<template>
  <Modal ref="modModal" header="">
    <UiServersPyroModal :header="modalHeader" :data="data" @modal="modModal.hide()">
      <div v-if="isEditMode">
        <div class="flex items-center gap-4">
          <DropdownSelect
            v-model="newModVersion"
            name="Project"
            :options="versionOptions"
            placeholder="Select project..."
          />
          <Button icon-only @click="handleModAction(selectedMod!)">
            <ChevronRightIcon />
          </Button>
        </div>
      </div>
      <UiServersProjectSelect v-else type="mod" @select="handleModAction" />
    </UiServersPyroModal>
  </Modal>

  <div v-if="data && mods" class="flex h-full w-full flex-col">
    <div class="flex items-center justify-between">
      <h1 class="my-4 text-2xl font-extrabold text-contrast">Mods</h1>
      <div class="flex gap-2">
        <Button color="green" outline @click="showAddModModal">
          <PlusIcon />
          Add mod
        </Button>
      </div>
    </div>
    <div
      class="mb-4 flex h-full w-full items-center gap-2 rounded-xl border border-solid border-blue bg-bg-blue p-4 text-contrast"
    >
      <UnknownIcon class="h-8 w-8 text-blue" />
      This page is under construction some features may not be available yet, or are subject to
      change.
    </div>
    <div class="flex h-full w-full flex-col overflow-y-scroll">
      <div v-if="hasMods(mods)" class="flex flex-col gap-2">
        <UiServersContentItem
          v-for="mod in mods"
          :key="mod.name"
          :data="mod"
          @toggle="toggleMod"
          @delete="removeMod"
          @edit="showEditModModal"
        />
      </div>
      <div v-else>You haven't added any mods yet, time to add some!</div>
    </div>
  </div>
  <UiServersPyroLoading v-else />
</template>

<script setup lang="ts">
import { PlusIcon, ChevronRightIcon, UnknownIcon } from "@modrinth/assets";
import { Modal, DropdownSelect, Button } from "@modrinth/ui";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "mods", "backups", "network", "startup", "ws", "fs"]>;
}>();

interface Mod {
  name?: string;
  filename: string;
  project_id?: string;
  version_id?: string;
  version_number?: string;
  icon_url?: string;
  disabled: boolean;
}

const prodOverride = await PyroAuthOverride();
const router = useRouter();
const route = useNativeRoute();
const serverId = route.params.id as string;

const modModal = ref();
const isEditMode = ref(false);
const modalHeader = ref("");
const selectedMod = ref<Mod | null>(null);
const newModVersion = ref("");
const versions = ref<Record<string, any[]>>({});

const data = computed(() => props.server.general);
const mods = computed(() => props.server.mods?.data);

if (data.value?.loader === "Vanilla") {
  router.push(`/servers/manage/${serverId}/content/datapacks`);
}

const hasMods = (mods: Mod[]) => {
  if (mods.length > 0) {
    return true;
  } else {
    return false;
  }
};

const fetchVersions = async (projectId: string) => {
  if (!versions.value[projectId]) {
    versions.value[projectId] = (await useBaseFetch(
      `project/${projectId}/version`,
      {},
      false,
      prodOverride,
    )) as any[];
  }
  return versions.value[projectId];
};

const toggleMod = async (mod: Mod) => {
  try {
    await props.server.fs?.renameFileOrFolder(
      `/mods/${mod.filename}`,
      mod.filename.includes("disabled")
        ? mod.filename.replace(".disabled", "")
        : `${mod.filename}.disabled`,
    );
    await props.server.refresh();
  } catch (error) {
    console.error("Error disabling mod:", error);
  }
};

const removeMod = async (mod: Mod) => {
  try {
    if (!mod.project_id) {
      throw new Error("Mod project_id is undefined");
    }
    await props.server.mods?.remove(mod.project_id);
    await props.server.refresh();
  } catch (error) {
    console.error("Error removing mod:", error);
  }
};

const showAddModModal = () => {
  isEditMode.value = false;
  modalHeader.value = "Add mod";
  modModal.value.show();
};

const showEditModModal = async (mod: Mod) => {
  if (!mod.project_id) {
    throw new Error("Mod project_id is undefined");
  }
  isEditMode.value = true;
  modalHeader.value = "Edit mod";
  selectedMod.value = mod;
  newModVersion.value = mod.version_number || "";
  await fetchVersions(mod.project_id);
  modModal.value.show();
};

const handleModAction = async (mod: Mod, versionNumber?: string) => {
  try {
    if (!mod.project_id) {
      throw new Error("Mod project_id is undefined");
    }
    const versionList = await fetchVersions(mod.project_id);
    const versionId = versionList.find((x: any) =>
      x.version_number === versionNumber ? versionNumber : mod.version_number,
    )?.id;
    if (isEditMode.value) {
      await props.server.mods?.reinstall(mod.project_id, versionId);
    } else {
      await props.server.mods?.install(mod.project_id, versionId);
    }
    await props.server.refresh();
    modModal.value.hide();
  } catch (error) {
    console.error("Error handling mod action:", error);
  }
};

const versionOptions = computed(() => {
  return selectedMod.value && selectedMod.value.project_id
    ? versions.value[selectedMod.value.project_id]?.map((version: any) => version.version_number) ||
        []
    : [];
});
</script>
