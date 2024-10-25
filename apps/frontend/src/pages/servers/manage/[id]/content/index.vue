<template>
  <NewModal ref="modModal" :header="modalHeader">
    <div v-if="isEditMode">
      <div class="mb-4 flex flex-col gap-4">
        <div class="inline-flex items-center">
          You're changing the version of
          <div class="ml-2 inline-flex items-center gap-1">
            <UiAvatar
              :src="selectedMod?.icon_url"
              size="24px"
              class="inline-block"
              no-shadow
              alt="Server Icon"
            />
            <strong>{{ selectedMod?.name }}</strong>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-4">
        <UiServersTeleportDropdownMenu
          v-model="newModVersion"
          name="Project"
          :options="versionOptions"
          placeholder="Select project..."
          class="!w-full"
        />
        <ButtonStyled color="brand">
          <button @click="handleModAction(selectedMod!)">Install</button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <div v-if="data && localMods" class="flex h-full w-full flex-col">
    <div class="flex h-full w-full flex-col">
      <div class="flex items-center justify-between">
        <div class="flex w-full flex-row items-center justify-end gap-4">
          <div class="relative w-full text-sm">
            <label class="sr-only" for="search">Search</label>
            <SearchIcon
              class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
              aria-hidden="true"
            />
            <input
              id="search"
              v-model="searchInput"
              class="w-full border-[1px] border-solid border-button-border pl-9"
              type="search"
              name="search"
              autocomplete="off"
              placeholder="Search content..."
              @input="debouncedSearch"
            />
          </div>
        </div>
      </div>

      <div v-if="hasMods" class="flex flex-col gap-2">
        <div v-if="filteredModrinthMods.length">
          <h2 class="mt-8 text-xl font-bold text-contrast">Modrinth mods</h2>
          <p>
            These are mods installed on your server that are listed on Modrinth. You can manage them
            here.
          </p>
          <div ref="modrinthListContainer" class="relative w-full">
            <div :style="{ position: 'relative', height: `${modrinthTotalHeight}px` }">
              <div :style="{ position: 'absolute', top: `${modrinthVisibleTop}px`, width: '100%' }">
                <div
                  v-for="mod in modrinthVisibleItems"
                  :key="mod.name"
                  class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-bg-raised hover:bg-table-alternateRow"
                  :class="mod.disabled ? 'bg-table-alternateRow text-secondary' : ''"
                  style="height: 64px"
                >
                  <NuxtLink
                    :to="`/project/${mod.project_id}/version/${mod.version_id}`"
                    class="group flex w-full items-center rounded-xl p-2"
                  >
                    <div class="flex items-center gap-2">
                      <UiAvatar
                        :src="mod.icon_url"
                        no-shadow
                        size="sm"
                        alt="Server Icon"
                        :class="mod.disabled ? 'grayscale' : ''"
                      />
                      <div class="flex flex-col">
                        <span class="flex items-center gap-2 text-lg font-bold">
                          {{ mod.name }}
                          <span
                            v-if="mod.disabled"
                            class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
                            >Disabled</span
                          >
                        </span>
                        <span class="text-xs text-secondary">{{ mod.version_number }}</span>
                      </div>
                    </div>
                  </NuxtLink>
                  <div
                    class="absolute right-2 flex gap-2 rounded-xl p-1 font-semibold text-contrast"
                  >
                    <ButtonStyled type="transparent">
                      <button v-if="mod.project_id" @click="showEditModModal(mod)">
                        <EditIcon />
                      </button>
                    </ButtonStyled>
                    <ButtonStyled type="transparent">
                      <button
                        :disabled="mod.project_id ? modActionsInProgress[mod.project_id] : true"
                        @click="removeModOptimistic(mod)"
                      >
                        <TrashIcon />
                      </button>
                    </ButtonStyled>
                    <input
                      :id="`toggle-${mod.project_id}`"
                      :checked="!mod.disabled"
                      :disabled="mod.project_id ? modActionsInProgress[mod.project_id] : true"
                      class="switch stylized-toggle"
                      type="checkbox"
                      @change="toggleModOptimistic(mod)"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-if="filteredExternalMods.length">
          <h2 class="mt-8 text-xl font-bold text-contrast">External mods</h2>
          <p>
            External mods are mods that are directly uploaded to your server or are part of a
            modpack, but are not listed on Modrinth. You can manage them via the Files tab.
          </p>
          <div ref="externalListContainer" class="relative w-full">
            <div :style="{ position: 'relative', height: `${externalTotalHeight}px` }">
              <div :style="{ position: 'absolute', top: `${externalVisibleTop}px`, width: '100%' }">
                <div
                  v-for="mod in externalVisibleItems"
                  :key="mod.name"
                  class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-bg-raised hover:bg-table-alternateRow"
                  :class="mod.disabled ? 'bg-table-alternateRow text-secondary' : ''"
                  style="height: 64px"
                >
                  <NuxtLink
                    :to="`files?path=mods`"
                    class="group flex w-full items-center rounded-xl p-2"
                  >
                    <div class="flex items-center gap-2">
                      <UiAvatar
                        :src="mod.icon_url"
                        no-shadow
                        size="sm"
                        alt="Server Icon"
                        :class="mod.disabled ? 'grayscale' : ''"
                      />
                      <div class="flex flex-col">
                        <span class="flex items-center gap-2 text-lg font-bold">
                          External Mod
                          <span
                            v-if="mod.disabled"
                            class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
                            >Disabled</span
                          >
                        </span>
                        <span class="text-xs text-secondary">{{
                          mod.filename.replace(".disabled", "")
                        }}</span>
                      </div>
                    </div>
                  </NuxtLink>
                  <div
                    class="absolute right-2 flex gap-2 rounded-xl p-1 font-semibold text-contrast"
                  >
                    <input
                      :id="`toggle-${mod.filename}`"
                      :checked="!mod.disabled"
                      :disabled="true"
                      class="switch stylized-toggle"
                      type="checkbox"
                      @change="toggleModOptimistic(mod)"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="mt-4 select-none text-center opacity-50">
        You haven't added any mods yet, time to add some!
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { SearchIcon, EditIcon, TrashIcon } from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
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

const ITEM_HEIGHT = 68;
const BUFFER_SIZE = 5;

const modrinthListContainer = ref<HTMLElement | null>(null);
const externalListContainer = ref<HTMLElement | null>(null);
const windowScrollY = ref(0);
const windowHeight = ref(0);

const modModal = ref();
const isEditMode = ref(false);
const modalHeader = ref("");
const selectedMod = ref<Mod | null>(null);
const newModVersion = ref("");
const versions = ref<Record<string, any[]>>({});
const localMods = ref<Mod[]>([]);
const modActionsInProgress = ref<Record<string, boolean>>({});
const modSearchInput = ref("");
const searchInput = ref("");

const modrinthTotalHeight = computed(() => filteredModrinthMods.value.length * ITEM_HEIGHT);
const externalTotalHeight = computed(() => filteredExternalMods.value.length * ITEM_HEIGHT);

const getVisibleRange = (containerTop: number, itemCount: number) => {
  const relativeScrollTop = Math.max(0, windowScrollY.value - containerTop);
  const start = Math.floor(relativeScrollTop / ITEM_HEIGHT);
  const visibleCount = Math.ceil(windowHeight.value / ITEM_HEIGHT);
  return {
    start: Math.max(0, start - BUFFER_SIZE),
    end: Math.min(itemCount, start + visibleCount + BUFFER_SIZE * 2),
  };
};

const modrinthVisibleRange = computed(() => {
  if (!modrinthListContainer.value) return { start: 0, end: 0 };
  const containerTop = modrinthListContainer.value.getBoundingClientRect().top + window.scrollY;
  return getVisibleRange(containerTop, filteredModrinthMods.value.length);
});

const externalVisibleRange = computed(() => {
  if (!externalListContainer.value) return { start: 0, end: 0 };
  const containerTop = externalListContainer.value.getBoundingClientRect().top + window.scrollY;
  return getVisibleRange(containerTop, filteredExternalMods.value.length);
});

const modrinthVisibleTop = computed(() => modrinthVisibleRange.value.start * ITEM_HEIGHT);
const externalVisibleTop = computed(() => externalVisibleRange.value.start * ITEM_HEIGHT);

const modrinthVisibleItems = computed(() => {
  return filteredModrinthMods.value.slice(
    modrinthVisibleRange.value.start,
    modrinthVisibleRange.value.end,
  );
});

const externalVisibleItems = computed(() => {
  return filteredExternalMods.value.slice(
    externalVisibleRange.value.start,
    externalVisibleRange.value.end,
  );
});

const handleScroll = () => {
  windowScrollY.value = window.scrollY;
};

const handleResize = () => {
  windowHeight.value = window.innerHeight;
};

onMounted(() => {
  windowHeight.value = window.innerHeight;
  window.addEventListener("scroll", handleScroll, { passive: true });
  window.addEventListener("resize", handleResize, { passive: true });
  handleScroll();
});

onUnmounted(() => {
  window.removeEventListener("scroll", handleScroll);
  window.removeEventListener("resize", handleResize);
});

const { refresh: refreshData } = await useAsyncData("serverData", async () => {
  await props.server.refresh(["general", "mods"]);
  return true;
});

const data = computed(() => props.server.general);

watch(
  () => props.server.mods?.data,
  (newMods) => {
    if (newMods) {
      localMods.value = [...newMods];
    }
  },
  { immediate: true },
);

const hasMods = computed(() => {
  return filteredMods.value?.length > 0;
});

const filteredMods = computed(() => {
  if (!modSearchInput.value.trim()) {
    return localMods.value;
  }
  return localMods.value.filter(
    (mod) =>
      mod.name?.toLowerCase().includes(modSearchInput.value.toLowerCase()) ||
      mod.filename.toLowerCase().includes(modSearchInput.value.toLowerCase()),
  );
});

const filteredExternalMods = computed(() => {
  return filteredMods.value.filter((mod) => !mod.project_id);
});

const filteredModrinthMods = computed(() => {
  return filteredMods.value.filter((mod) => mod.project_id);
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

const debounce = <T extends (...args: any[]) => void>(
  func: T,
  wait: number,
): ((...args: Parameters<T>) => void) => {
  let timeout: ReturnType<typeof setTimeout>;
  return function (...args: Parameters<T>): void {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
};

const debouncedSearch = debounce(() => {
  modSearchInput.value = searchInput.value;
}, 300);
const toggleModOptimistic = async (mod: Mod) => {
  if (!mod.project_id || modActionsInProgress.value[mod.project_id]) return;

  modActionsInProgress.value[mod.project_id] = true;

  const originalMods = [...localMods.value];
  const originalDisabled = mod.disabled;

  mod.disabled = !mod.disabled;

  try {
    const newFilename = mod.disabled
      ? `${mod.filename}.disabled`
      : mod.filename.replace(".disabled", "");

    await props.server.fs?.renameFileOrFolder(`/mods/${mod.filename}`, newFilename);

    mod.filename = newFilename;

    await refreshData();

    addNotification({
      text: `${mod.disabled ? "Disabled" : "Enabled"} ${mod.name}. Restart your server for changes to take effect.`,
      type: "success",
    });
  } catch (error) {
    console.error("Error toggling mod:", error);
    localMods.value = originalMods;
    mod.disabled = originalDisabled;

    addNotification({
      text: `Something went wrong toggling ${mod.name}`,
      type: "error",
    });
  } finally {
    modActionsInProgress.value[mod.project_id] = false;
  }
};

const removeModOptimistic = async (mod: Mod) => {
  if (!mod.filename || modActionsInProgress.value[mod.filename]) return;

  modActionsInProgress.value[mod.filename] = true;

  const originalMods = [...localMods.value];

  localMods.value = localMods.value.filter((m) => m.filename !== mod.filename);

  try {
    await props.server.mods?.remove(`/mods/${mod.filename}`);
    await refreshData();

    addNotification({
      text: `Successfully removed ${mod.name}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    localMods.value = originalMods;
    addNotification({
      text: `couldn't remove ${mod.name}`,
      type: "error",
    });
  } finally {
    modActionsInProgress.value[mod.filename] = false;
  }
};

// const showAddModModal = () => {
//   isEditMode.value = false;
//   modalHeader.value = "Add mod";
//   modModal.value.show();
// };

const showEditModModal = async (mod: Mod) => {
  if (!mod.project_id) {
    throw new Error("Mod project_id is undefined");
  }
  isEditMode.value = true;
  modalHeader.value = "Editing mod version";
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
    await refreshData();
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

<style scoped>
.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
