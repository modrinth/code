<template>
  <NewModal ref="modModal" header="Editing mod version">
    <div>
      <div class="mb-4 flex flex-col gap-4">
        <div class="inline-flex flex-wrap items-center">
          You're changing the version of
          <div class="inline-flex flex-wrap items-center gap-1 text-nowrap pl-2">
            <UiAvatar
              :src="currentMod?.icon_url"
              size="24px"
              class="inline-block"
              alt="Server Icon"
            />
            <strong>{{ currentMod?.name + "." }}</strong>
          </div>
        </div>
        <div>
          <div v-if="props.server.general?.upstream" class="flex items-center gap-2">
            <InfoIcon class="hidden sm:block" />
            <span class="text-sm text-secondary">
              Your server was created from a modpack. Changing the mod version may cause unexpected
              issues. You can update the modpack version in your server's Options > Platform
              settings.
            </span>
          </div>
        </div>
      </div>
      <div class="flex items-center gap-4">
        <UiServersTeleportDropdownMenu
          v-model="currentVersion"
          name="Project"
          :options="currentVersions"
          placeholder="Select project..."
          class="!w-full"
          :display-name="
            (version) => (typeof version === 'object' ? version?.version_number : version)
          "
        />
      </div>
      <div class="mt-4 flex flex-row items-center gap-4">
        <ButtonStyled color="brand">
          <button :disabled="currentMod.changing" @click="changeModVersion">
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

  <div v-if="server.general && localMods" class="relative isolate flex h-full w-full flex-col">
    <div ref="pyroContentSentinel" class="sentinel" data-pyro-content-sentinel />
    <div class="relative flex h-full w-full flex-col">
      <div class="sticky top-0 z-20 -mt-4 flex items-center justify-between bg-bg py-4">
        <div class="flex w-full flex-col items-center gap-2 sm:flex-row sm:gap-4">
          <div class="flex w-full items-center gap-2 sm:gap-4">
            <div class="relative flex-1 text-sm">
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
                :placeholder="`Search ${type.toLocaleLowerCase()}s...`"
                @input="debouncedSearch"
              />
            </div>
            <ButtonStyled>
              <UiServersTeleportOverflowMenu
                position="bottom"
                direction="left"
                :aria-label="`Filter ${type}s`"
                :options="[
                  { id: 'all', action: () => (filterMethod = 'all') },
                  { id: 'enabled', action: () => (filterMethod = 'enabled') },
                  { id: 'disabled', action: () => (filterMethod = 'disabled') },
                ]"
              >
                <span class="whitespace-pre text-sm font-medium">
                  {{ filterMethodLabel }}
                </span>
                <FilterIcon aria-hidden="true" />
                <DropdownIcon aria-hidden="true" class="h-5 w-5 text-secondary" />
                <template #all> All {{ type.toLocaleLowerCase() }}s </template>
                <template #enabled> Only enabled </template>
                <template #disabled> Only disabled </template>
              </UiServersTeleportOverflowMenu>
            </ButtonStyled>
          </div>
          <ButtonStyled v-if="hasMods" color="brand" type="outlined">
            <nuxt-link
              class="w-full text-nowrap sm:w-fit"
              :to="`/${type.toLocaleLowerCase()}s?sid=${props.server.serverId}`"
            >
              <PlusIcon />
              Add {{ type.toLocaleLowerCase() }}
            </nuxt-link>
          </ButtonStyled>
        </div>
      </div>
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
                    class="group flex min-w-0 items-center rounded-xl p-2"
                  >
                    <div class="flex min-w-0 items-center gap-2">
                      <UiAvatar
                        :src="mod.icon_url"
                        size="sm"
                        alt="Server Icon"
                        :class="mod.disabled ? 'grayscale' : ''"
                      />
                      <div class="flex min-w-0 flex-col">
                        <span class="flex min-w-0 items-center gap-2 text-lg font-bold">
                          <span class="truncate">{{
                            mod.name || mod.filename.replace(".disabled", "")
                          }}</span>
                          <span
                            v-if="mod.disabled"
                            class="hidden rounded-full bg-button-bg p-1 px-2 text-xs text-contrast sm:block"
                            >Disabled</span
                          >
                        </span>
                        <span class="min-w-0 text-xs text-secondary">{{
                          mod.version_number || "External mod"
                        }}</span>
                      </div>
                    </div>
                  </NuxtLink>
                  <div class="flex items-center gap-2 pr-4 font-semibold text-contrast">
                    <ButtonStyled v-if="mod.project_id" type="transparent">
                      <button
                        v-tooltip="'Edit mod version'"
                        :disabled="mod.changing"
                        class="!hidden sm:!block"
                        @click="beginChangeModVersion(mod)"
                      >
                        <template v-if="mod.changing">
                          <UiServersIconsLoadingIcon />
                        </template>
                        <template v-else>
                          <EditIcon />
                        </template>
                      </button>
                    </ButtonStyled>
                    <ButtonStyled type="transparent">
                      <button
                        v-tooltip="'Delete mod'"
                        :disabled="mod.changing"
                        class="!hidden sm:!block"
                        @click="removeMod(mod)"
                      >
                        <TrashIcon />
                      </button>
                    </ButtonStyled>

                    <!-- Dropdown for mobile -->
                    <div class="mr-2 flex items-center sm:hidden">
                      <UiServersIconsLoadingIcon
                        v-if="mod.changing"
                        class="mr-2 h-5 w-5 animate-spin"
                        style="color: var(--color-base)"
                      />
                      <ButtonStyled v-else circular type="transparent">
                        <UiServersTeleportOverflowMenu
                          :options="[
                            {
                              id: 'edit',
                              action: () => beginChangeModVersion(mod),
                              shown: !!(mod.project_id && !mod.changing),
                            },
                            {
                              id: 'delete',
                              action: () => removeMod(mod),
                            },
                          ]"
                        >
                          <MoreVerticalIcon aria-hidden="true" />
                          <template #edit>
                            <EditIcon class="h-5 w-5" />
                            <span>Edit</span>
                          </template>
                          <template #delete>
                            <TrashIcon class="h-5 w-5" />
                            <span>Delete</span>
                          </template>
                        </UiServersTeleportOverflowMenu>
                      </ButtonStyled>
                    </div>

                    <input
                      :id="`toggle-${mod.filename}`"
                      :checked="!mod.disabled"
                      :disabled="mod.changing"
                      class="switch stylized-toggle"
                      type="checkbox"
                      @change="toggleMod(mod)"
                    />
                  </div>
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>
      <!-- no mods has platform -->
      <div
        v-else-if="
          !hasMods &&
          props.server.general?.loader &&
          props.server.general?.loader.toLocaleLowerCase() !== 'vanilla'
        "
        class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
      >
        <PackageClosedIcon class="size-24" />
        <p class="m-0 font-bold text-contrast">No {{ type.toLocaleLowerCase() }}s found!</p>
        <p class="m-0">
          Add some {{ type.toLocaleLowerCase() }}s to your server to manage them here.
        </p>
        <ButtonStyled color="brand">
          <NuxtLink :to="`/${type.toLocaleLowerCase()}s?sid=${props.server.serverId}`">
            <PlusIcon />
            Add {{ type.toLocaleLowerCase() }}
          </NuxtLink>
        </ButtonStyled>
      </div>
      <div v-else class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center">
        <UiServersIconsLoaderIcon loader="Vanilla" class="size-24" />
        <p class="m-0 pt-3 font-bold text-contrast">Your server is running Vanilla Minecraft</p>
        <p class="m-0">
          Add content to your server by installing a modpack or choosing a different platform that
          supports {{ type }}s.
        </p>
        <div class="flex flex-row items-center gap-4">
          <ButtonStyled class="mt-8">
            <NuxtLink :to="`/modpacks?sid=${props.server.serverId}`">
              <CompassIcon />
              Find a modpack
            </NuxtLink>
          </ButtonStyled>
          <div>or</div>
          <ButtonStyled class="mt-8">
            <NuxtLink :to="`/${type}s?sid=${props.server.serverId}`">
              <WrenchIcon />
              Change platform
            </NuxtLink>
          </ButtonStyled>
        </div>
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
  MoreVerticalIcon,
  CompassIcon,
  WrenchIcon,
} from "@modrinth/assets";
import { ButtonStyled, NewModal } from "@modrinth/ui";
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import type { Server } from "~/composables/pyroServers";

const props = defineProps<{
  server: Server<["general", "content", "backups", "network", "startup", "ws", "fs"]>;
}>();

const type = computed(() => {
  const loader = props.server.general?.loader?.toLowerCase();
  return loader === "paper" || loader === "purpur" ? "Plugin" : "Mod";
});

interface Mod {
  name?: string;
  filename: string;
  project_id?: string;
  version_id?: string;
  version_number?: string;
  icon_url?: string;
  disabled: boolean;
  changing?: boolean;
}

const ITEM_HEIGHT = 72;
const BUFFER_SIZE = 5;

const listContainer = ref<HTMLElement | null>(null);
const windowScrollY = ref(0);
const windowHeight = ref(0);

const localMods = ref<Mod[]>([]);

const searchInput = ref("");
const modSearchInput = ref("");
const filterMethod = ref("all");

const filterMethodLabel = computed(() => {
  switch (filterMethod.value) {
    case "disabled":
      return "Only disabled";
    case "enabled":
      return "Only enabled";
    default:
      return `All ${type.value.toLocaleLowerCase()}s`;
  }
});

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

watch(
  () => props.server.content?.data,
  (newMods) => {
    if (newMods) {
      localMods.value = [...newMods];
    }
  },
  { immediate: true },
);

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

const pyroContentSentinel = ref<HTMLElement | null>(null);
const debouncedSearch = debounce(() => {
  modSearchInput.value = searchInput.value;

  if (pyroContentSentinel.value) {
    pyroContentSentinel.value.scrollIntoView({
      behavior: "smooth",
      block: "start",
    });
  }
}, 300);

async function toggleMod(mod: Mod) {
  mod.changing = true;

  const originalFilename = mod.filename;
  try {
    const newFilename = mod.filename.endsWith(".disabled")
      ? mod.filename.replace(".disabled", "")
      : `${mod.filename}.disabled`;

    const sourcePath = `/mods/${mod.filename}`;
    const destinationPath = `/mods/${newFilename}`;

    mod.disabled = newFilename.endsWith(".disabled");
    mod.filename = newFilename;

    await props.server.fs?.moveFileOrFolder(sourcePath, destinationPath);

    await props.server.refresh(["general", "content"]);
  } catch (error) {
    mod.filename = originalFilename;
    mod.disabled = originalFilename.endsWith(".disabled");

    console.error("Error toggling mod:", error);
    addNotification({
      text: `Something went wrong toggling ${mod.name || mod.filename.replace(".disabled", "")}`,
      type: "error",
    });
  }

  mod.changing = false;
}

async function removeMod(mod: Mod) {
  mod.changing = true;

  try {
    await props.server.content?.remove(
      type.value as "Mod" | "Plugin",
      `/${type.value.toLowerCase()}s/${mod.filename}`,
    );
    await props.server.refresh(["general", "content"]);
  } catch (error) {
    console.error("Error removing mod:", error);

    addNotification({
      text: `couldn't remove ${mod.name || mod.filename}`,
      type: "error",
    });
  }

  mod.changing = false;
}

const modModal = ref();
const currentMod = ref();
const currentVersions = ref();
const currentVersion = ref();

async function beginChangeModVersion(mod: Mod) {
  currentMod.value = mod;
  currentVersions.value = await useBaseFetch(`project/${mod.project_id}/version`, {}, false);

  currentVersions.value = currentVersions.value.filter((version: any) =>
    version.loaders.includes(props.server.general?.loader?.toLowerCase()),
  );

  currentVersion.value = currentVersions.value.find(
    (version: any) => version.id === mod.version_id,
  );
  modModal.value.show();
}

async function changeModVersion() {
  currentMod.value.changing = true;
  try {
    modModal.value.hide();
    await props.server.content?.reinstall(
      type.value,
      currentMod.value.version_id,
      currentVersion.value.id,
    );
    await props.server.refresh(["general", "content"]);
  } catch (error) {
    console.error("Error changing mod version:", error);
  }
  currentMod.value.changing = false;
}

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
</script>

<style scoped>
.sentinel {
  position: absolute;
  top: -1rem;
  left: 0;
  right: 0;
  height: 1px;
  visibility: hidden;
}

.stylized-toggle:checked::after {
  background: var(--color-accent-contrast) !important;
}
</style>
