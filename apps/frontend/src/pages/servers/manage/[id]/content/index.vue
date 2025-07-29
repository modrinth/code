<template>
  <UiServersContentVersionEditModal
    v-if="!invalidModal"
    ref="versionEditModal"
    :type="type"
    :mod-pack="Boolean(props.server.general?.upstream)"
    :game-version="props.server.general?.mc_version ?? ''"
    :loader="props.server.general?.loader?.toLowerCase() ?? ''"
    :server-id="props.server.serverId"
    @change-version="changeModVersion($event)"
  />

  <div
    v-if="server.moduleErrors.content"
    class="flex w-full flex-col items-center justify-center gap-4 p-4"
  >
    <div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
      <div class="flex flex-col items-center text-center">
        <div class="flex flex-col items-center gap-4">
          <div class="grid place-content-center rounded-full bg-bg-orange p-4">
            <IssuesIcon class="size-12 text-orange" />
          </div>
          <h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load content</h1>
        </div>
        <p class="text-lg text-secondary">
          We couldn't load your server's {{ type.toLowerCase() }}s. Here's what we know:
          <span class="break-all font-mono">{{
            JSON.stringify(server.moduleErrors.content.error)
          }}</span>
        </p>
        <ButtonStyled size="large" color="brand" @click="() => server.refresh(['content'])">
          <button class="mt-6 !w-full">Retry</button>
        </ButtonStyled>
      </div>
    </div>
  </div>

  <div v-else-if="server.general && localMods" class="relative isolate flex h-full w-full flex-col">
    <div ref="pyroContentSentinel" class="sentinel" data-pyro-content-sentinel />
    <div class="relative flex h-full w-full flex-col">
      <div class="sticky top-0 z-20 -mt-3 flex items-center justify-between bg-bg py-3">
        <div class="flex w-full flex-col-reverse items-center gap-2 sm:flex-row">
          <div class="flex w-full items-center gap-2">
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
                :placeholder="`Search ${localMods.length} ${type.toLocaleLowerCase()}s...`"
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
                <span class="hidden whitespace-pre sm:block">
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
          <div v-if="hasMods" class="flex w-full items-center gap-2 sm:w-fit">
            <ButtonStyled>
              <button class="w-full text-nowrap sm:w-fit" @click="initiateFileUpload">
                <FileIcon />
                Add file
              </button>
            </ButtonStyled>
            <ButtonStyled color="brand">
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
      </div>
      <FilesUploadDropdown
        v-if="props.server.fs"
        ref="uploadDropdownRef"
        class="rounded-xl bg-bg-raised"
        :margin-bottom="16"
        :file-type="type"
        :current-path="`/${type.toLocaleLowerCase()}s`"
        :fs="props.server.fs"
        :accepted-types="acceptFileFromProjectType(type.toLocaleLowerCase()).split(',')"
        @upload-complete="() => props.server.refresh(['content'])"
      />
      <FilesUploadDragAndDrop
        v-if="server.general && localMods"
        class="relative min-h-[50vh]"
        overlay-class="rounded-xl border-2 border-dashed border-secondary"
        :type="type"
        @files-dropped="handleDroppedFiles"
      >
        <div v-if="hasFilteredMods" class="flex flex-col gap-2 transition-all">
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
                          : `files?path=${type.toLocaleLowerCase()}s`
                      "
                      class="flex min-w-0 flex-1 items-center gap-2 rounded-xl p-2"
                      draggable="false"
                    >
                      <Avatar
                        :src="mod.icon_url"
                        size="sm"
                        alt="Server Icon"
                        :class="mod.disabled ? 'opacity-75 grayscale' : ''"
                      />
                      <div class="flex min-w-0 flex-col gap-1">
                        <span class="text-md flex min-w-0 items-center gap-2 font-bold">
                          <span class="truncate text-contrast">{{ friendlyModName(mod) }}</span>
                          <span
                            v-if="mod.disabled"
                            class="hidden rounded-full bg-button-bg p-1 px-2 text-xs text-contrast sm:block"
                            >Disabled</span
                          >
                        </span>
                        <div class="min-w-0 text-xs text-secondary">
                          <span v-if="mod.owner" class="hidden sm:block"> by {{ mod.owner }} </span>
                          <span class="block font-semibold sm:hidden">
                            {{ mod.version_number || `External ${type.toLocaleLowerCase()}` }}
                          </span>
                        </div>
                      </div>
                    </NuxtLink>
                    <div class="ml-2 hidden min-w-0 flex-1 flex-col text-sm sm:flex">
                      <div class="truncate font-semibold text-contrast">
                        <span v-tooltip="`${type} version`">{{
                          mod.version_number || `External ${type.toLocaleLowerCase()}`
                        }}</span>
                      </div>
                      <div class="truncate">
                        <span v-tooltip="`${type} file name`">
                          {{ mod.filename }}
                        </span>
                      </div>
                    </div>
                    <div
                      class="flex items-center justify-end gap-2 pr-4 font-semibold text-contrast sm:min-w-44"
                    >
                      <ButtonStyled color="red" type="transparent">
                        <button
                          v-tooltip="`Delete ${type.toLocaleLowerCase()}`"
                          :disabled="mod.changing"
                          class="!hidden sm:!block"
                          @click="removeMod(mod)"
                        >
                          <TrashIcon />
                        </button>
                      </ButtonStyled>
                      <ButtonStyled type="transparent">
                        <button
                          v-tooltip="
                            mod.project_id
                              ? `Edit ${type.toLocaleLowerCase()} version`
                              : `External ${type.toLocaleLowerCase()}s cannot be edited`
                          "
                          :disabled="mod.changing || !mod.project_id"
                          class="!hidden sm:!block"
                          @click="showVersionModal(mod)"
                        >
                          <template v-if="mod.changing">
                            <UiServersIconsLoadingIcon class="animate-spin" />
                          </template>
                          <template v-else>
                            <EditIcon />
                          </template>
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
                                action: () => showVersionModal(mod),
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
            props.server.general?.loader &&
            props.server.general?.loader.toLocaleLowerCase() !== 'vanilla'
          "
          class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
        >
          <div
            v-if="!hasFilteredMods && hasMods"
            class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
          >
            <SearchIcon class="size-24" />
            <p class="m-0 font-bold text-contrast">
              No {{ type.toLocaleLowerCase() }}s found for your query!
            </p>
            <p class="m-0">Try another query, or show everything.</p>
            <ButtonStyled>
              <button @click="showAll">
                <ListIcon />
                Show everything
              </button>
            </ButtonStyled>
          </div>
          <div
            v-else
            class="mt-4 flex h-full flex-col items-center justify-center gap-4 text-center"
          >
            <PackageClosedIcon class="size-24" />
            <p class="m-0 font-bold text-contrast">No {{ type.toLocaleLowerCase() }}s found!</p>
            <p class="m-0">
              Add some {{ type.toLocaleLowerCase() }}s to your server to manage them here.
            </p>
            <div class="flex flex-row items-center gap-4">
              <ButtonStyled type="outlined">
                <button class="w-full text-nowrap sm:w-fit" @click="initiateFileUpload">
                  <FileIcon />
                  Add file
                </button>
              </ButtonStyled>
              <ButtonStyled color="brand">
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
              <NuxtLink :to="`/servers/manage/${props.server.serverId}/options/loader`">
                <WrenchIcon />
                Change platform
              </NuxtLink>
            </ButtonStyled>
          </div>
        </div>
      </FilesUploadDragAndDrop>
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
  PlusIcon,
  MoreVerticalIcon,
  CompassIcon,
  WrenchIcon,
  ListIcon,
  FileIcon,
  IssuesIcon,
} from "@modrinth/assets";
import { Avatar, ButtonStyled } from "@modrinth/ui";
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import type { Mod } from "@modrinth/utils";
import FilesUploadDragAndDrop from "~/components/ui/servers/FilesUploadDragAndDrop.vue";
import FilesUploadDropdown from "~/components/ui/servers/FilesUploadDropdown.vue";
import { acceptFileFromProjectType } from "~/helpers/fileUtils.js";
import { ModrinthServer } from "~/composables/servers/modrinth-servers.ts";

const props = defineProps<{
  server: ModrinthServer;
}>();

const type = computed(() => {
  const loader = props.server.general?.loader?.toLowerCase();
  return loader === "paper" || loader === "purpur" ? "Plugin" : "Mod";
});

interface ContentItem extends Mod {
  changing?: boolean;
}

const ITEM_HEIGHT = 72;
const BUFFER_SIZE = 5;

const listContainer = ref<HTMLElement | null>(null);
const windowScrollY = ref(0);
const windowHeight = ref(0);

const localMods = ref<ContentItem[]>([]);

const searchInput = ref("");
const modSearchInput = ref("");
const filterMethod = ref("all");

const uploadDropdownRef = ref();

const versionEditModal = ref();
const currentEditMod = ref<ContentItem | null>(null);
const invalidModal = computed(
  () => !props.server.general?.mc_version || !props.server.general?.loader,
);
async function changeModVersion(event: string) {
  const mod = currentEditMod.value;

  if (mod) mod.changing = true;

  try {
    versionEditModal.value.hide();

    // This will be used instead once backend implementation is done
    // await props.server.content?.reinstall(
    //   `/${type.value.toLowerCase()}s/${event.fileName}`,
    //   currentMod.value.project_id,
    //   currentVersion.value.id,
    // );

    await props.server.content?.install(
      type.value.toLowerCase() as "mod" | "plugin",
      mod?.project_id || "",
      event,
    );

    await props.server.content?.remove(`/${type.value.toLowerCase()}s/${mod?.filename}`);

    await props.server.refresh(["general", "content"]);
  } catch (error) {
    const errmsg = `Error changing mod version: ${error}`;
    console.error(errmsg);
    addNotification({
      text: errmsg,
      type: "error",
    });
    return;
  }
  if (mod) mod.changing = false;
}

function showVersionModal(mod: ContentItem) {
  if (invalidModal.value || !mod?.project_id || !mod?.filename) {
    const errmsg = invalidModal.value
      ? "Data required for changing mod version was not found."
      : `${!mod?.project_id ? "No mod project ID found" : "No mod filename found"} for ${friendlyModName(mod!)}`;
    console.error(errmsg);
    addNotification({
      text: errmsg,
      type: "error",
    });
    return;
  }

  currentEditMod.value = mod;
  versionEditModal.value.show(mod);
}

const handleDroppedFiles = (files: File[]) => {
  files.forEach((file) => {
    uploadDropdownRef.value?.uploadFile(file);
  });
};

const initiateFileUpload = () => {
  const input = document.createElement("input");
  input.type = "file";
  input.accept = acceptFileFromProjectType(type.value.toLowerCase());
  input.multiple = true;
  input.onchange = () => {
    if (input.files) {
      Array.from(input.files).forEach((file) => {
        uploadDropdownRef.value?.uploadFile(file);
      });
    }
  };
  input.click();
};

const showAll = () => {
  searchInput.value = "";
  modSearchInput.value = "";
  filterMethod.value = "all";
};

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
    const sentinelRect = pyroContentSentinel.value.getBoundingClientRect();
    if (sentinelRect.top < 0 || sentinelRect.bottom > window.innerHeight) {
      pyroContentSentinel.value.scrollIntoView({
        // behavior: "smooth",
        block: "start",
      });
    }
  }
}, 300);

function friendlyModName(mod: ContentItem) {
  if (mod.name) return mod.name;

  // remove .disabled if at the end of the filename
  let cleanName = mod.filename.endsWith(".disabled") ? mod.filename.slice(0, -9) : mod.filename;

  // remove everything after the last dot
  const lastDotIndex = cleanName.lastIndexOf(".");
  if (lastDotIndex !== -1) cleanName = cleanName.substring(0, lastDotIndex);
  return cleanName;
}

async function toggleMod(mod: ContentItem) {
  mod.changing = true;

  const originalFilename = mod.filename;
  try {
    const newFilename = mod.filename.endsWith(".disabled")
      ? mod.filename.slice(0, -9)
      : `${mod.filename}.disabled`;

    const folder = `${type.value.toLocaleLowerCase()}s`;
    const sourcePath = `/${folder}/${mod.filename}`;
    const destinationPath = `/${folder}/${newFilename}`;

    mod.disabled = newFilename.endsWith(".disabled");
    mod.filename = newFilename;

    await props.server.fs?.moveFileOrFolder(sourcePath, destinationPath);

    await props.server.refresh(["general", "content"]);
  } catch (error) {
    mod.filename = originalFilename;
    mod.disabled = originalFilename.endsWith(".disabled");

    console.error("Error toggling mod:", error);
    addNotification({
      text: `Something went wrong toggling ${friendlyModName(mod)}`,
      type: "error",
    });
  }

  mod.changing = false;
}

async function removeMod(mod: ContentItem) {
  mod.changing = true;

  try {
    await props.server.content?.remove(`/${type.value.toLowerCase()}s/${mod.filename}`);
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

const hasMods = computed(() => {
  return localMods.value?.length > 0;
});

const hasFilteredMods = computed(() => {
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
    return friendlyModName(a).localeCompare(friendlyModName(b));
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
