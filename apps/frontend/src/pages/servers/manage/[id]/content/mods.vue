<template>
  <Modal ref="modModal" header="">
    <div class="h-[500px]">
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
    </div>
  </Modal>

  <div v-if="data" class="flex h-full w-full flex-col gap-2 px-8 py-4">
    <div class="flex items-center justify-between gap-2 px-3">
      <h2 class="m-0 text-3xl font-bold text-contrast">Mods</h2>
      <Button icon-only transparent @click="showAddModModal">
        <PlusIcon class="h-10 w-10 text-contrast" />
      </Button>
    </div>
    <div
      v-if="mods && modsStatus == 'success'"
      class="flex h-full w-full flex-col overflow-y-scroll"
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
  const modsValue = mods.value;
  if (modsValue) {
    return [...modsValue].sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? ""));
  }
  return [];
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
    refreshNuxtData("content-mods-mods");
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
    refreshNuxtData("content-mods-mods");
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
      await serverStore.reinstallMod(serverId, mod.project_id, versionId);
    } else {
      await serverStore.installMod(serverId, mod.project_id, versionId);
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
