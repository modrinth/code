<template>
  <div class="flex flex-col gap-3">
    <div class="flex flex-col justify-between gap-3 lg:flex-row">
      <div class="iconified-input flex-1 lg:max-w-md">
        <SearchIcon aria-hidden="true" class="text-lg" />
        <input
          v-model="query"
          class="h-[40px]"
          autocomplete="off"
          spellcheck="false"
          type="text"
          :placeholder="formatMessage(messages.searchPlaceholder)"
          @input="goToPage(1)"
        />
        <Button v-if="query" class="r-btn" @click="() => (query = '')">
          <XIcon />
        </Button>
      </div>

      <div v-if="totalPages > 1" class="hidden flex-1 justify-center lg:flex">
        <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
        <ConfettiExplosion v-if="visible" />
      </div>

      <div class="flex flex-col justify-end gap-2 sm:flex-row lg:flex-shrink-0">
        <div class="flex flex-col gap-2 sm:flex-row">
          <DropdownSelect
            v-slot="{ selected }"
            v-model="currentFilterType"
            class="!w-full flex-grow sm:!w-[280px] sm:flex-grow-0 lg:!w-[280px]"
            :name="formatMessage(messages.filterBy)"
            :options="filterTypes as unknown[]"
            @change="goToPage(1)"
          >
            <span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
              <FilterIcon class="size-4 flex-shrink-0" />
              <span class="truncate">{{ selected }} ({{ filteredProjects.length }})</span>
            </span>
          </DropdownSelect>

          <DropdownSelect
            v-slot="{ selected }"
            v-model="currentSortType"
            class="!w-full flex-grow sm:!w-[150px] sm:flex-grow-0 lg:!w-[150px]"
            :name="formatMessage(messages.sortBy)"
            :options="sortTypes as unknown[]"
            @change="goToPage(1)"
          >
            <span class="flex flex-row gap-2 align-middle font-semibold text-secondary">
              <SortAscIcon v-if="selected === 'Oldest'" class="size-4 flex-shrink-0" />
              <SortDescIcon v-else class="size-4 flex-shrink-0" />
              <span class="truncate">{{ selected }}</span>
            </span>
          </DropdownSelect>
        </div>

        <ButtonStyled color="orange" class="w-full sm:w-auto">
          <button
            class="flex !h-[40px] w-full items-center justify-center gap-2 sm:w-auto"
            @click="moderateAllInFilter()"
          >
            <ScaleIcon class="size-4 flex-shrink-0" />
            <span class="hidden sm:inline">{{ formatMessage(messages.moderate) }}</span>
            <span class="sm:hidden">Moderate</span>
          </button>
        </ButtonStyled>
      </div>
    </div>

    <div v-if="totalPages > 1" class="flex justify-center lg:hidden">
      <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
      <ConfettiExplosion v-if="visible" />
    </div>

    <div class="mt-4 flex flex-col gap-2">
      <div v-if="paginatedProjects.length === 0" class="universal-card h-24 animate-pulse"></div>
      <ModerationQueueCard
        v-for="item in paginatedProjects"
        v-else
        :key="item.project.id"
        :queue-entry="item"
        :owner="item.owner"
        :org="item.org"
      />
    </div>

    <div v-if="totalPages > 1" class="mt-4 flex justify-center">
      <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { DropdownSelect, Button, ButtonStyled, Pagination } from "@modrinth/ui";
import {
  XIcon,
  SearchIcon,
  SortAscIcon,
  SortDescIcon,
  FilterIcon,
  ScaleIcon,
} from "@modrinth/assets";
import { defineMessages, useVIntl } from "@vintl/vintl";
import { useLocalStorage } from "@vueuse/core";
import ConfettiExplosion from "vue-confetti-explosion";
import Fuse from "fuse.js";
import ModerationQueueCard from "~/components/ui/moderation/ModerationQueueCard.vue";
import { useModerationStore } from "~/store/moderation.ts";
import { enrichProjectBatch, type ModerationProject } from "~/helpers/moderation.ts";

const { formatMessage } = useVIntl();
const moderationStore = useModerationStore();
const route = useRoute();
const router = useRouter();

const visible = ref(false);
if (import.meta.client && history && history.state && history.state.confetti) {
  setTimeout(async () => {
    history.state.confetti = false;
    visible.value = true;
    await nextTick();
    setTimeout(() => {
      visible.value = false;
    }, 5000);
  }, 1000);
}

const messages = defineMessages({
  searchPlaceholder: {
    id: "moderation.search.placeholder",
    defaultMessage: "Search...",
  },
  filterBy: {
    id: "moderation.filter.by",
    defaultMessage: "Filter by",
  },
  sortBy: {
    id: "moderation.sort.by",
    defaultMessage: "Sort by",
  },
  moderate: {
    id: "moderation.moderate",
    defaultMessage: "Moderate",
  },
});

const { data: allProjects } = await useLazyAsyncData("moderation-projects", async () => {
  const startTime = performance.now();
  let currentOffset = 0;
  const PROJECT_ENDPOINT_COUNT = 350;
  const allProjects: ModerationProject[] = [];

  const enrichmentPromises: Promise<ModerationProject[]>[] = [];

  while (true) {
    const projects = (await useBaseFetch(
      `moderation/projects?count=${PROJECT_ENDPOINT_COUNT}&offset=${currentOffset}`,
      { internal: true },
    )) as any[];

    if (projects.length === 0) break;

    const enrichmentPromise = enrichProjectBatch(projects);
    enrichmentPromises.push(enrichmentPromise);

    currentOffset += projects.length;

    if (enrichmentPromises.length >= 3) {
      const completed = await Promise.all(enrichmentPromises.splice(0, 2));
      allProjects.push(...completed.flat());
    }

    if (projects.length < PROJECT_ENDPOINT_COUNT) break;
  }

  const remainingBatches = await Promise.all(enrichmentPromises);
  allProjects.push(...remainingBatches.flat());

  const endTime = performance.now();
  const duration = endTime - startTime;

  console.debug(
    `Projects fetched and processed in ${duration.toFixed(2)}ms (${(duration / 1000).toFixed(2)}s)`,
  );

  return allProjects;
});

const query = ref(route.query.q?.toString() || "");

watch(
  query,
  (newQuery) => {
    const currentQuery = { ...route.query };
    if (newQuery) {
      currentQuery.q = newQuery;
    } else {
      delete currentQuery.q;
    }

    router.replace({
      path: route.path,
      query: currentQuery,
    });
  },
  { immediate: false },
);

watch(
  () => route.query.q,
  (newQueryParam) => {
    const newValue = newQueryParam?.toString() || "";
    if (query.value !== newValue) {
      query.value = newValue;
    }
  },
);

const currentFilterType = useLocalStorage("moderation-current-filter-type", () => "All projects");
const filterTypes: readonly string[] = readonly([
  "All projects",
  "Modpacks",
  "Mods",
  "Resource Packs",
  "Data Packs",
  "Plugins",
  "Shaders",
]);

const currentSortType = useLocalStorage("moderation-current-sort-type", () => "Oldest");
const sortTypes: readonly string[] = readonly(["Oldest", "Newest"]);

const currentPage = ref(1);
const itemsPerPage = 15;
const totalPages = computed(() => Math.ceil((filteredProjects.value?.length || 0) / itemsPerPage));

const fuse = computed(() => {
  if (!allProjects.value || allProjects.value.length === 0) return null;
  return new Fuse(allProjects.value, {
    keys: [
      {
        name: "project.title",
        weight: 3,
      },
      {
        name: "project.slug",
        weight: 2,
      },
      {
        name: "project.description",
        weight: 2,
      },
      {
        name: "project.project_type",
        weight: 1,
      },
      "owner.user.username",
      "org.name",
      "org.slug",
    ],
    includeScore: true,
    threshold: 0.4,
  });
});

const searchResults = computed(() => {
  if (!query.value || !fuse.value) return null;
  return fuse.value.search(query.value).map((result) => result.item);
});

const baseFiltered = computed(() => {
  if (!allProjects.value) return [];
  return query.value && searchResults.value ? searchResults.value : [...allProjects.value];
});

const typeFiltered = computed(() => {
  if (currentFilterType.value === "All projects") return baseFiltered.value;

  const filterMap: Record<string, string> = {
    Modpacks: "modpack",
    Mods: "mod",
    "Resource Packs": "resourcepack",
    "Data Packs": "datapack",
    Plugins: "plugin",
    Shaders: "shader",
  };

  const projectType = filterMap[currentFilterType.value];
  if (!projectType) return baseFiltered.value;

  return baseFiltered.value.filter((queueItem) =>
    queueItem.project.project_types.includes(projectType),
  );
});

const filteredProjects = computed(() => {
  const filtered = [...typeFiltered.value];

  if (currentSortType.value === "Oldest") {
    filtered.sort((a, b) => {
      const dateA = new Date(a.project.queued || a.project.published || 0).getTime();
      const dateB = new Date(b.project.queued || b.project.published || 0).getTime();
      return dateA - dateB;
    });
  } else {
    filtered.sort((a, b) => {
      const dateA = new Date(a.project.queued || a.project.published || 0).getTime();
      const dateB = new Date(b.project.queued || b.project.published || 0).getTime();
      return dateB - dateA;
    });
  }

  return filtered;
});

const paginatedProjects = computed(() => {
  if (!filteredProjects.value) return [];
  const start = (currentPage.value - 1) * itemsPerPage;
  const end = start + itemsPerPage;
  return filteredProjects.value.slice(start, end);
});

function goToPage(page: number) {
  currentPage.value = page;
}

function moderateAllInFilter() {
  moderationStore.setQueue(filteredProjects.value.map((queueItem) => queueItem.project.id));
  navigateTo({
    name: "type-id",
    params: {
      type: "project",
      id: moderationStore.getCurrentProjectId(),
    },
    state: {
      showChecklist: true,
    },
  });
}
</script>
