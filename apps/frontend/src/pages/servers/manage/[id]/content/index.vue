<template>
  <NewModal ref="modModal" :header="modalHeader">
    <div v-if="isEditMode">
      <div class="mb-4 flex flex-col gap-4">
        <div class="inline-flex flex-wrap items-center">
          You're changing the version of
          <div class="ml-2 inline-flex items-center gap-1">
            <UiAvatar
              :src="selectedMod?.icon_url"
              size="24px"
              class="inline-block"
              alt="Server Icon"
            />
            <strong>{{ selectedMod?.name + "." }}</strong> The latest version is
            <strong>{{ versionOptions[0] + "." }}</strong>
          </div>
        </div>
        <div>
          <div v-if="props.server.general?.upstream" class="flex items-center gap-2">
            <InfoIcon />
            <span class="text-sm text-secondary">
              Your server was created from a modpack. Changing the mod version may cause unexpected
              issues.
            </span>
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
      </div>
      <div class="mt-4 flex flex-row items-center gap-4">
        <ButtonStyled color="brand">
          <button @click="handleModAction(selectedMod!)">
            <PlusIcon />
            Install
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="modModal.value.hide()">
            <XIcon />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>

  <div v-if="data && localMods" class="flex h-full w-full flex-col">
    <div class="relative flex h-full w-full flex-col">
      <div class="mb-4 flex items-center justify-between">
        <div class="flex w-full flex-row items-center gap-4">
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
              placeholder="Search mods..."
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
              <template #enabled> Only enabled </template>
              <template #disabled> Only disabled </template>
            </UiServersTeleportOverflowMenu>
          </ButtonStyled>
        </div>
      </div>

      <Transition name="sync-banner">
        <div
          v-if="status === 'pending'"
          class="fixed bottom-8 left-4 right-4 z-50 mx-auto flex h-fit w-full max-w-4xl flex-row items-center gap-4 rounded-2xl border-2 border-solid border-button-border bg-bg-raised p-2 shadow-2xl transition-all duration-300"
        >
          <div
            class="grid size-12 place-content-center overflow-hidden rounded-xl border-[1px] border-solid border-button-border bg-button-bg shadow-sm"
          >
            <UiServersLoadingIcon class="size-6 animate-spin" />
          </div>
          <div class="flex flex-col gap-0.5">
            <p class="m-0 text-sm font-bold text-contrast">Working on it...</p>
            <p class="m-0 text-sm">We're syncing changes to your server.</p>
          </div>
        </div>
      </Transition>

      <!-- :class="{
          'mt-[68px] opacity-50': status === 'pending',
        }" -->
      <div v-if="hasMods" class="flex flex-col gap-2 transition-all">
        <div ref="listContainer" class="relative w-full">
          <div :style="{ position: 'relative', height: `${totalHeight}px` }">
            <div :style="{ position: 'absolute', top: `${visibleTop}px`, width: '100%' }">
              <template v-for="mod in visibleItems.items" :key="mod.filename">
                <div
                  class="relative mb-2 flex w-full items-center justify-between rounded-xl bg-bg-raised"
                  :class="mod.disabled ? 'bg-table-alternateRow text-secondary' : ''"
                  style="height: 64px"
                >
                  <NuxtLink
                    :to="
                      mod.project_id
                        ? `/project/${mod.project_id}/version/${mod.version_id}`
                        : `files?path=mods`
                    "
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
                          {{ mod.name || mod.filename.replace(".disabled", "") }}
                          <span
                            v-if="mod.disabled"
                            class="rounded-full bg-button-bg p-1 px-2 text-xs text-contrast"
                            >Disabled</span
                          >
                        </span>
                        <span class="text-xs text-secondary">{{
                          mod.version_number || "External mod"
                        }}</span>
                      </div>
                    </div>
                  </NuxtLink>
                  <div
                    class="absolute right-2 flex items-center gap-2 pr-2 font-semibold text-contrast"
                  >
                    <ButtonStyled v-if="mod.project_id" type="transparent">
                      <button
                        v-tooltip="'Edit mod version'"
                        :disabled="
                          isFetchingVersionsForMod[mod.project_id] ||
                          modActionsInProgress[getIdentifier(mod)]
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
                        :disabled="modActionsInProgress[getIdentifier(mod)]"
                        @click="removeModOptimistic(mod)"
                      >
                        <TrashIcon />
                      </button>
                    </ButtonStyled>
                    <input
                      :id="`toggle-${getIdentifier(mod)}`"
                      :checked="!mod.disabled"
                      :disabled="modActionsInProgress[getIdentifier(mod)]"
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
  InfoIcon,
  XIcon,
  PlusIcon,
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

const ITEM_HEIGHT = 72;
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
      return "Only disabled";
    case "enabled":
      return "Only enabled";
    default:
      return "All mods";
  }
});

const filterMods = (method: string) => {
  filterMethod.value = method;
};

const getIdentifier = (mod: Mod) => {
  return mod.project_id || mod.filename.replace(".disabled", "");
};

const totalHeight = computed(() => {
  const itemsHeight = filteredMods.value.length * ITEM_HEIGHT;
  return itemsHeight;
});

const getVisibleRange = () => {
  if (!listContainer.value) return { start: 0, end: 0 };

  const containerTop = listContainer.value.getBoundingClientRect().top + window.scrollY;
  const scrollTop = Math.max(0, windowScrollY.value - containerTop);

  const start = Math.floor(scrollTop / ITEM_HEIGHT);
  const visibleCount = Math.ceil(windowHeight.value / ITEM_HEIGHT);

  return {
    start: Math.max(0, start - BUFFER_SIZE),
    end: Math.min(filteredMods.value.length, start + visibleCount + BUFFER_SIZE * 2),
  };
};

const visibleTop = computed(() => {
  const range = getVisibleRange();
  return range.start * ITEM_HEIGHT;
});

const visibleItems = computed(() => {
  const range = getVisibleRange();
  const items = filteredMods.value;

  return {
    items: items.slice(Math.max(0, range.start), Math.min(items.length, range.end)),
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
const { refresh: refreshData, status } = await useLazyAsyncData("serverData", async () => {
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
  const identifier = mod.project_id || mod.filename.replace(".disabled", "");
  if (modActionsInProgress.value[identifier]) return;

  modActionsInProgress.value[identifier] = true;

  const originalMods = [...localMods.value];
  const originalDisabled = mod.disabled;

  try {
    const newFilename = mod.disabled
      ? mod.filename.replace(".disabled", "")
      : `${mod.filename}.disabled`;

    const sourcePath = `/mods/${mod.filename}`;
    const destinationPath = `/mods/${newFilename}`;

    await props.server.fs?.moveFileOrFolder(sourcePath, destinationPath);

    const modIndex = localMods.value.findIndex((m) => m.filename === mod.filename);
    if (modIndex !== -1) {
      const updatedMod = { ...mod, disabled: !mod.disabled, filename: newFilename };
      localMods.value = [
        ...localMods.value.slice(0, modIndex),
        updatedMod,
        ...localMods.value.slice(modIndex + 1),
      ];
    }

    await refreshData();

    addNotification({
      text: `${originalDisabled ? "Enabled" : "Disabled"} ${
        mod.name || mod.filename.replace(".disabled", "")
      }. Restart your server for changes to take effect.`,
      type: "success",
    });
  } catch (error) {
    console.error("Error toggling mod:", error);
    localMods.value = originalMods;
    mod.disabled = originalDisabled;

    addNotification({
      text: `Something went wrong toggling ${mod.name || mod.filename.replace(".disabled", "")}`,
      type: "error",
    });
  } finally {
    modActionsInProgress.value[identifier] = false;
  }
};

const removeModOptimistic = async (mod: Mod) => {
  const identifier = getIdentifier(mod);
  if (modActionsInProgress.value[identifier]) return;

  modActionsInProgress.value[identifier] = true;

  const originalMods = [...localMods.value];

  localMods.value = localMods.value.filter((m) => m.filename !== mod.filename);

  try {
    await props.server.mods?.remove(`/mods/${mod.filename}`);
    await refreshData();

    addNotification({
      text: `Successfully removed ${mod.name || mod.filename.replace(".disabled", "")}`,
      type: "success",
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    localMods.value = originalMods;
    addNotification({
      text: `couldn't remove ${mod.name || mod.filename.replace(".disabled", "")}`,
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

  const statusFilteredMods = (() => {
    switch (filterMethod.value) {
      case "disabled":
        return mods.filter((mod) => mod.disabled);
      case "enabled":
        return mods.filter((mod) => !mod.disabled);
      default:
        return mods;
    }
  })();

  return statusFilteredMods.sort((a, b) => {
    const aName = a.name || a.filename.replace(".disabled", "");
    const bName = b.name || b.filename.replace(".disabled", "");
    return aName.localeCompare(bName);
  });
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

.sync-banner-enter-active {
  transition:
    opacity 300ms,
    transform 300ms;
}

.sync-banner-leave-active {
  transition:
    opacity 200ms,
    transform 200ms;
}

.sync-banner-enter-from,
.sync-banner-leave-to {
  opacity: 0;
  transform: translateY(100%) scale(0.98);
}

.sync-banner-enter-to,
.sync-banner-leave-from {
  opacity: 1;
  transform: none;
}
</style>
