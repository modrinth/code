<template>
  <Modal ref="modModal" header="">
    <div class="h-[500px]">
      <UiServersPyroModal @modal="modModal.hide()" :header="modalHeader" :data="data">
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
        <UiServersProjectSelect type="mod" @select="handleModAction" v-else />
      </UiServersPyroModal>
    </div>
  </Modal>

  <div class="flex h-full w-full flex-col" v-if="data">
    <div
      class="flex items-center justify-between gap-2 border-0 border-b border-solid border-bg-raised p-3"
    >
      <h2 class="m-0 text-2xl font-bold text-contrast">Mods</h2>
      <Button icon-only transparent @click="showAddModModal">
        <PlusIcon class="h-10 w-10 text-contrast" />
      </Button>
    </div>
    <div
      class="flex h-full w-full flex-col overflow-y-scroll"
      v-if="mods && modsStatus == 'success'"
    >
      <UiServersContentItem
        v-for="mod in sortedMods"
        :key="mod.name"
        :data="mod"
        @toggle="toggleMod"
        @delete="removeMod"
        @edit="showEditModModal"
      />
    </div>
    <UiServersPyroLoading v-else />
  </div>
  <UiServersPyroLoading v-else />
</template>

<script setup lang="ts">
import { PlusIcon, ChevronRightIcon } from "@modrinth/assets";
import { Button, Modal, DropdownSelect } from "@modrinth/ui";
import { ref, computed } from "vue";

interface Mod {
  name?: string;
  filename: string;
  project_id?: string;
  version_id?: string;
  version_number?: string;
  icon_url?: string;
  disabled: boolean;
}

const serverStore = useServerStore();
const route = useNativeRoute();
const config = useRuntimeConfig();
const prodOverride = await PyroAuthOverride();
const serverId = route.params.id as string;

const modModal = ref();
const isEditMode = ref(false);
const modalHeader = ref("");
const selectedMod = ref<Mod | null>(null);
const newModVersion = ref("");
const versions = ref<Record<string, any[]>>({});

const data = computed(() => serverStore.serverData[serverId]);

const { data: mods, status: modsStatus } = await useLazyAsyncData("content-mods-mods", async () => {
  return (await serverStore.getMods(serverId)) as Mod[];
});

const sortedMods = computed(() => {
  return mods.value?.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? "")) || [];
});

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
    await serverStore.renameFileOrFolder(
      serverId,
      `/mods/${mod.filename}`,
      mod.filename.includes("disabled")
        ? mod.filename.replace(".disabled", "")
        : `${mod.filename}.disabled`,
    );
    new Promise((resolve) =>
      setTimeout(() => {
        refreshNuxtData("content-mods-mods");
      }, 1000),
    );
  } catch (error) {
    console.error("Error disabling mod:", error);
  }
};

const removeMod = async (mod: Mod) => {
  try {
    if (!mod.project_id) {
      throw new Error("Mod project_id is undefined");
    }
    await serverStore.removeMod(serverId, mod.project_id);
    new Promise((resolve) =>
      setTimeout(() => {
        refreshNuxtData("content-mods-mods");
      }, 1000),
    );
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

const handleModAction = async (mod: Mod, version_number?: string) => {
  try {
    if (!mod.project_id) {
      throw new Error("Mod project_id is undefined");
    }
    const versionList = await fetchVersions(mod.project_id);
    const version_id = versionList.find((x: any) =>
      x.version_number === version_number ? version_number : mod.version_number,
    )?.id;
    if (isEditMode.value) {
      await serverStore.reinstallMod(serverId, mod.project_id, version_id);
    } else {
      await serverStore.installMod(serverId, mod.project_id, version_id);
    }
    await refreshNuxtData("content-mods-mods");
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
