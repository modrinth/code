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
        <div class="flex w-full max-w-lg flex-row items-center gap-4">
          <div class="relative w-full text-sm">
            <label class="sr-only" for="search">Search</label>
            <SearchIcon
              class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2"
              aria-hidden="true"
            />
            <input
              id="search"
              v-model="searchInput"
              class="!h-9 !min-h-0 w-full border-[1px] border-solid border-button-border pl-9"
              type="search"
              name="search"
              autocomplete="off"
              placeholder="Search content..."
              @input="debouncedSearch"
            />
          </div>
          <ButtonStyled>
            <UiServersTeleportOverflowMenu
              position="bottom"
              direction="left"
              aria-label="Filter mods"
              :options="[
                { id: 'all', action: () => filterMods('all') },
                { id: 'enabled', action: () => filterMods('enabled') },
                { id: 'disabled', action: () => filterMods('disabled') },
              ]"
            >
              <span class="whitespace-pre text-sm font-medium">
                {{ filterMethodLabel }}
              </span>
              <FilterIcon aria-hidden="true" />
              <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
              <template #all> All mods </template>
              <template #enabled> Enabled mods </template>
              <template #disabled> Disabled mods </template>
            </UiServersTeleportOverflowMenu>
          </ButtonStyled>
        </div>
      </div>

      <div v-if="hasMods" class="flex flex-col gap-2">
        <div ref="listContainer" class="relative w-full">
          <div :style="{ position: 'relative', height: `${totalHeight}px` }">
            <div :style="{ position: 'absolute', top: `${visibleTop}px`, width: '100%' }">
              <!-- Modrinth Section -->
              <div class="universal-card">
                <template v-if="visibleItems.modrinth.header">
                  <div class="h-[72px]">
                    <h2 class="mb-0 mt-8 text-xl font-bold text-contrast">Modrinth mods</h2>
                    <p class="mb-4 mt-3">
                      These are mods installed on your server that are listed on Modrinth.
                    </p>
                  </div>
                </template>

                <template v-for="mod in visibleItems.modrinth.items" :key="mod.name">
                  <div
                    class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-table-alternateRow"
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
                        <button
                          v-if="mod.project_id"
                          v-tooltip="'Edit mod version'"
                          :disabled="
                            isFetchingVersionsForMod[mod.project_id] ||
                            modActionsInProgress[mod.project_id]
                          "
                          @click="showEditModModal(mod)"
                        >
                          <template v-if="isFetchingVersionsForMod[mod.project_id]">
                            <UiServersLoadingIcon />
                          </template>
                          <template v-else>
                            <EditIcon />
                          </template>
                        </button>
                      </ButtonStyled>
                      <ButtonStyled type="transparent">
                        <button
                          v-tooltip="'Delete mod'"
                          :disabled="mod.project_id ? modActionsInProgress[mod.project_id] : false"
                          @click="removeModOptimistic(mod)"
                        >
                          <TrashIcon />
                        </button>
                      </ButtonStyled>
                      <input
                        :id="`toggle-${mod.project_id}`"
                        :checked="!mod.disabled"
                        class="switch stylized-toggle"
                        type="checkbox"
                        @change="toggleModOptimistic(mod)"
                      />
                    </div>
                  </div>
                </template>
              </div>

              <!-- External Section -->
              <div class="universal-card">
                <template v-if="visibleItems.external.header">
                  <div class="h-[72px]">
                    <h2 class="mb-0 mt-8 text-xl font-bold text-contrast">External mods</h2>
                    <p class="mb-4 mt-3">
                      External mods are mods that are directly uploaded to your server or are part
                      of a modpack, but are not listed on Modrinth.
                    </p>
                  </div>
                </template>

                <template v-for="mod in visibleItems.external.items" :key="mod.filename">
                  <div
                    class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-table-alternateRow"
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
                          size="sm"
                          alt="Server Icon"
                          :class="mod.disabled ? 'grayscale' : ''"
                        />
                        <div class="flex flex-col">
                          <span class="flex items-center gap-2 text-lg font-bold">
                            {{ mod.filename.replace(".disabled", "") }}
                            <span
                              v-if="mod.disabled"
                              class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
                              >Disabled</span
                            >
                          </span>
                          <span class="text-xs text-secondary"> External Mod </span>
                        </div>
                      </div>
                    </NuxtLink>
                    <div
                      class="absolute right-2 flex gap-2 rounded-xl p-1 font-semibold text-contrast"
                    >
                      <ButtonStyled type="transparent">
                        <button
                          v-tooltip="'Delete mod'"
                          :disabled="modActionsInProgress[mod.filename]"
                          @click="removeModOptimistic(mod)"
                        >
                          <TrashIcon />
                        </button>
                      </ButtonStyled>
                      <input
                        :id="`toggle-${mod.filename}`"
                        :checked="!mod.disabled"
                        :disabled="modActionsInProgress[mod.filename]"
                        class="switch stylized-toggle"
                        type="checkbox"
                        @change="toggleModOptimistic(mod)"
                      />
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-else class="mt-4 flex h-full flex-col items-center justify-center">
        <PackageClosedIcon class="size-24 text-neutral-500" />
        <p class="m-0 pb-2 pt-3 text-neutral-200">No mods found!</p>
        <p class="m-0 pb-3 text-neutral-400">Add some mods to your server to manage them here.</p>
        <ButtonStyled color="brand" class="mt-8">
          <button @click="gotoManageModPage">Manage mods</button>
        </ButtonStyled>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  SearchIcon,
  EditIcon,
  TrashIcon,
  PackageClosedIcon,
  FilterIcon,
  DropdownIcon,
} from "@modrinth/assets";
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

const ITEM_HEIGHT = 64;
const HEADER_HEIGHT = 72;
const BUFFER_SIZE = 5;

const listContainer = ref<HTMLElement | null>(null);
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
const searchInput = ref("");
const modSearchInput = ref("");
const isFetchingVersionsForMod = ref<Record<string, boolean>>({});

const filterMethod = ref("all");

const filterMethodLabel = computed(() => {
  switch (filterMethod.value) {
    case "disabled":
      return "Disabled";
    case "enabled":
      return "Enabled";
    default:
      return "All";
  }
});

const filterMods = (method: string) => {
  filterMethod.value = method;
};

const totalHeight = computed(() => {
  const modrinthHeight = filteredModrinthMods.value.length * ITEM_HEIGHT;
  const externalHeight = filteredExternalMods.value.length * ITEM_HEIGHT;
  const headerHeights =
    (filteredModrinthMods.value.length > 0 ? HEADER_HEIGHT : 0) +
    (filteredExternalMods.value.length > 0 ? HEADER_HEIGHT : 0);
  return modrinthHeight + externalHeight + headerHeights;
});

const getVisibleRange = () => {
  if (!listContainer.value) return { start: 0, end: 0 };

  const containerTop = listContainer.value.getBoundingClientRect().top + window.scrollY;
  const scrollTop = Math.max(0, windowScrollY.value - containerTop);

  const start = Math.floor(scrollTop / ITEM_HEIGHT);
  const visibleCount = Math.ceil(windowHeight.value / ITEM_HEIGHT);

  return {
    start: Math.max(0, start - BUFFER_SIZE),
    end: Math.min(
      filteredModrinthMods.value.length + filteredExternalMods.value.length,
      start + visibleCount + BUFFER_SIZE * 2,
    ),
  };
};

const visibleTop = computed(() => {
  const range = getVisibleRange();
  return range.start * ITEM_HEIGHT;
});

const visibleItems = computed(() => {
  const range = getVisibleRange();
  const modrinthStart = 0;
  const externalStart = filteredModrinthMods.value.length;

  return {
    modrinth: {
      header: range.start <= modrinthStart && filteredModrinthMods.value.length > 0,
      items: filteredModrinthMods.value.slice(
        Math.max(0, range.start - modrinthStart),
        Math.min(filteredModrinthMods.value.length, range.end - modrinthStart),
      ),
    },
    external: {
      header: range.start <= externalStart && filteredExternalMods.value.length > 0,
      items: filteredExternalMods.value.slice(
        Math.max(0, range.start - externalStart),
        Math.min(filteredExternalMods.value.length, range.end - externalStart),
      ),
    },
  };
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

const prodOverride = await PyroAuthOverride();
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
  const identifier = mod.project_id || mod.filename;
  if (modActionsInProgress.value[identifier]) return;

  modActionsInProgress.value[identifier] = true;

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
      text: `${mod.disabled ? "Disabled" : "Enabled"} ${mod.name || "External mod"}. Restart your server for changes to take effect.`,
      type: "success",
    });
  } catch (error) {
    console.error("Error toggling mod:", error);
    localMods.value = originalMods;
    mod.disabled = originalDisabled;

    addNotification({
      text: `Something went wrong toggling ${mod.name || "External mod"}`,
      type: "error",
    });
  } finally {
    modActionsInProgress.value[identifier] = false;
  }
};

const removeModOptimistic = async (mod: Mod) => {
  const identifier = mod.project_id || mod.filename;
  if (modActionsInProgress.value[identifier]) return;

  modActionsInProgress.value[identifier] = true;

  const originalMods = [...localMods.value];

  localMods.value = localMods.value.filter((m) => m.filename !== mod.filename);

  try {
    await props.server.mods?.remove(`/mods/${mod.filename}`);
    await refreshData();

    addNotification({
      text: `Successfully removed ${mod.name || "External mod"}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    localMods.value = originalMods;
    addNotification({
      text: `couldn't remove ${mod.name || "External mod"}`,
      type: "error",
    });
  } finally {
    modActionsInProgress.value[identifier] = false;
  }
};

const showEditModModal = async (mod: Mod) => {
  if (!mod.project_id) {
    throw new Error("Mod project_id is undefined");
  }
  isFetchingVersionsForMod.value[mod.project_id] = true;
  isEditMode.value = true;
  modalHeader.value = "Editing mod version";
  selectedMod.value = mod;
  newModVersion.value = mod.version_number || "";
  try {
    await fetchVersions(mod.project_id);
    modModal.value.show();
  } catch (error) {
    console.error("Error fetching versions:", error);
  } finally {
    isFetchingVersionsForMod.value[mod.project_id] = false;
  }
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

const gotoManageModPage = () => {
  navigateTo(`/servers/manage/${props.server.serverId}/options/loader`);
};

const hasMods = computed(() => {
  return filteredMods.value?.length > 0;
});

const filteredMods = computed(() => {
  const mods = modSearchInput.value.trim()
    ? localMods.value.filter(
        (mod) =>
          mod.name?.toLowerCase().includes(modSearchInput.value.toLowerCase()) ||
          mod.filename.toLowerCase().includes(modSearchInput.value.toLowerCase()),
      )
    : localMods.value;

  switch (filterMethod.value) {
    case "disabled":
      return mods.filter((mod) => mod.disabled);
    case "enabled":
      return mods.filter((mod) => !mod.disabled);
    default:
      return mods;
  }
});

const filteredExternalMods = computed(() => {
  return filteredMods.value.filter((mod) => !mod.project_id);
});

const filteredModrinthMods = computed(() => {
  return filteredMods.value.filter((mod) => mod.project_id);
});

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
