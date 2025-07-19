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
          @input="updateSearchResults()"
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
            @change="updateSearchResults()"
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
            @change="updateSearchResults()"
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
      <ModerationQueueCard
        v-for="item in paginatedProjects"
        :key="item.project.id"
        :project="item.project"
        :owner="item.owner"
        :org="item.org"
      />
      <div
        v-if="!paginatedProjects || paginatedProjects.length === 0"
        class="universal-card h-24 animate-pulse"
      ></div>
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
import type { Project, TeamMember, Organization } from "@modrinth/utils";
import ConfettiExplosion from "vue-confetti-explosion";
import Fuse from "fuse.js";
import ModerationQueueCard from "~/components/ui/moderation/ModerationQueueCard.vue";
import { asEncodedJsonArray, fetchSegmented } from "~/utils/fetch-helpers.ts";
import { useModerationStore } from "~/store/moderation.ts";

const { formatMessage } = useVIntl();
const moderationStore = useModerationStore();

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

interface ModerationProject {
  project: Project;
  owner: TeamMember | null;
  org: Organization | null;
}

const { data: allProjects } = await useAsyncData("moderation-projects", async () => {
  const projects = (await useBaseFetch("moderation/projects?count=10000")) as Project[];

  const teamIds = [...new Set(projects.map((p) => p.team).filter(Boolean))];
  const orgIds = [...new Set(projects.map((p) => p.organization).filter(Boolean))];

  const [teamsData, orgsData]: [TeamMember[][], Organization[]] = await Promise.all([
    teamIds.length > 0
      ? fetchSegmented(teamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
      : Promise.resolve([]),
    orgIds.length > 0
      ? fetchSegmented(orgIds, (ids) => `organizations?ids=${asEncodedJsonArray(ids)}`, {
          apiVersion: 3,
        })
      : Promise.resolve([]),
  ]);

  const teamMap = new Map<string, TeamMember[]>();
  const orgMap = new Map<string, Organization>();

  teamsData.forEach((team) => {
    let teamId = null;
    for (const member of team) {
      teamId = member.team_id;
      if (!teamMap.has(teamId)) {
        teamMap.set(teamId, team);
        break;
      }
    }
  });

  orgsData.forEach((org: Organization) => {
    orgMap.set(org.id, org);
  });

  return projects.map((project) => {
    let owner: TeamMember | null = null;
    let org: Organization | null = null;

    if (project.team) {
      const teamMembers = teamMap.get(project.team);
      if (teamMembers) {
        owner = teamMembers.find((member) => member.role === "Owner") || null;
      }
    }

    if (project.organization) {
      org = orgMap.get(project.organization) || null;
    }

    return {
      project,
      owner,
      org,
    } as ModerationProject;
  });
});

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
const itemsPerPage = 15;
const totalPages = computed(() => Math.ceil((filteredProjects.value?.length || 0) / itemsPerPage));

const fuse = computed(() => {
  if (!allProjects.value || allProjects.value.length === 0) return null;
  return new Fuse(allProjects.value, {
    keys: ["title", "description", "project_type", "slug"],
    includeScore: true,
    threshold: 0.4,
  });
});

const filteredProjects = computed(() => {
  if (!allProjects.value) return [];

  let filtered;

  if (query.value && fuse.value) {
    const results = fuse.value.search(query.value);
    filtered = results.map((result) => result.item);
  } else {
    filtered = [...allProjects.value];
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
      filtered = filtered.filter((queueItem) => queueItem.project.project_type === projectType);
    }
  }

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

function updateSearchResults() {
  currentPage.value = 1;
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

function goToPage(page: number) {
  currentPage.value = page;
}
</script>
