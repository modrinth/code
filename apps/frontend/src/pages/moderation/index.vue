<template>
  <div class="flex flex-row justify-between gap-3">
    <div class="iconified-input">
      <SearchIcon aria-hidden="true" class="text-lg" />
      <input
        v-model="query"
        class="h-[40px]"
        autocomplete="off"
        spellcheck="false"
        type="text"
        :placeholder="formatMessage(messages.searchPlaceholder)"
        @input="updateSearchResults()"
      />
      <Button v-if="query" class="r-btn" @click="() => (query = '')">
        <XIcon />
      </Button>
    </div>
    <div v-if="totalPages > 1" class="flex justify-center">
      <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
    </div>
    <div class="flex flex-row justify-end gap-2">
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentFilterType"
        class="!w-[250px] flex-grow md:flex-grow-0"
        :name="formatMessage(messages.filterBy)"
        :options="filterTypes as unknown[]"
        @change="updateSearchResults()"
      >
        <span class="flex flex-row gap-2 align-middle font-semibold text-secondary"
          ><FilterIcon class="size-4" />{{ selected }}</span
        >
      </DropdownSelect>
      <DropdownSelect
        v-slot="{ selected }"
        v-model="currentSortType"
        class="!w-[150px] flex-grow md:flex-grow-0"
        :name="formatMessage(messages.sortBy)"
        :options="sortTypes as unknown[]"
        @change="updateSearchResults()"
      >
        <span class="flex flex-row gap-2 align-middle font-semibold text-secondary"
          ><SortAscIcon class="size-4" v-if="selected === 'Oldest'" />
          <SortDescIcon class="size-4" v-else />{{ selected }}</span
        >
      </DropdownSelect>
      <ButtonStyled color="orange">
        <button class="!h-[40px]">
          <ScaleIcon class="size-4" /> {{ formatMessage(messages.moderate) }}
        </button>
      </ButtonStyled>
    </div>
  </div>
  <div class="mt-4 flex flex-col gap-2">
    <ModerationQueueCard
      v-for="project in paginatedProjects"
      :key="project.id"
      :project="project"
    />
    <div
      v-if="!paginatedProjects || paginatedProjects.length === 0"
      class="universal-card h-24 animate-pulse"
    ></div>
  </div>
  <div v-if="totalPages > 1" class="mt-4 flex justify-center">
    <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
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
import type { Project } from "@modrinth/utils";
import ModerationQueueCard from "~/components/ui/moderation/ModerationQueueCard.vue";
import { asEncodedJsonArray, fetchSegmented } from "~/utils/fetch-helpers.ts";

const { formatMessage } = useVIntl();

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

const { data: allProjects } = await useAsyncData(
  "moderation-projects",
  async () => (await useBaseFetch("moderation/projects?count=10000")) as Project[],
);

const query = useLocalStorage("moderation-query", "");
const currentFilterType = useLocalStorage("moderation-current-filter-type", () => "All");
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
const itemsPerPage = 5;
const totalPages = computed(() => Math.ceil((filteredProjects.value?.length || 0) / itemsPerPage));

const filteredProjects = computed(() => {
  if (!allProjects.value) return [];

  let filtered = [...allProjects.value];

  if (query.value) {
    filtered = filtered.filter(
      (project) =>
        project.title?.toLowerCase().includes(query.value.toLowerCase()) ||
        project.description?.toLowerCase().includes(query.value.toLowerCase()),
    );
  }

  if (currentFilterType.value !== "All projects") {
    const filterMap: Record<string, string> = {
      Modpacks: "modpack",
      Mods: "mod",
      "Resource Packs": "resourcepack",
      "Data Packs": "datapack",
      Plugins: "plugin",
      Shaders: "shader",
    };

    const projectType = filterMap[currentFilterType.value];
    if (projectType) {
      filtered = filtered.filter((project) => project.project_type === projectType);
    }
  }

  if (currentSortType.value === "Oldest") {
    filtered.sort((a, b) => {
      const dateA = new Date(a.queued || a.published || 0).getTime();
      const dateB = new Date(b.queued || b.published || 0).getTime();
      return dateA - dateB;
    });
  } else {
    filtered.sort((a, b) => {
      const dateA = new Date(a.queued || a.published || 0).getTime();
      const dateB = new Date(b.queued || b.published || 0).getTime();
      return dateB - dateA;
    });
  }

  return filtered;
});

const paginatedProjectsBase = computed(() => {
  if (!filteredProjects.value) return [];
  const start = (currentPage.value - 1) * itemsPerPage;
  const end = start + itemsPerPage;
  return filteredProjects.value.slice(start, end);
});

const { data: paginatedProjects } = await useAsyncData(
  `moderation-projects-page-${currentPage.value}-${query.value}-${currentFilterType.value}-${currentSortType.value}`,
  async () => {
    const projects = paginatedProjectsBase.value;
    if (!projects || projects.length === 0) return [];

    const teamIds = projects.map((x: any) => x.team_id).filter(Boolean);
    const orgIds = projects.filter((x: any) => x.organization).map((x: any) => x.organization);

    const [teams, orgs] = await Promise.all([
      teamIds.length > 0
        ? fetchSegmented(teamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
        : Promise.resolve([]),
      orgIds.length > 0
        ? fetchSegmented(orgIds, (ids) => `organizations?ids=${asEncodedJsonArray(ids)}`, {
            apiVersion: 3,
          })
        : Promise.resolve([]),
    ]);

    return projects.map((project: any) => {
      const owner =
        teams.length > 0
          ? teams.flat().find((x: any) => x.team_id === project.team_id && x.role === "Owner")
          : null;
      const org = orgs.length > 0 ? orgs.find((x: any) => x.id === project.organization) : null;

      return {
        ...project,
        owner,
        org,
        inferred_project_type: project.project_type,
      };
    });
  },
  {
    watch: [paginatedProjectsBase, currentPage, query, currentFilterType, currentSortType],
  },
);

function updateSearchResults() {
  currentPage.value = 1;
}

function goToPage(page: number) {
  currentPage.value = page;
}
</script>
