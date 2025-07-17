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
        class="!w-[280px] flex-grow md:flex-grow-0"
        :name="formatMessage(messages.filterBy)"
        :options="filterTypes as unknown[]"
        @change="updateSearchResults()"
      >
        <span class="flex flex-row gap-2 align-middle font-semibold text-secondary"
          ><FilterIcon class="size-4" />{{ selected }} ({{ filteredReports.length }})</span
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
          ><SortAscIcon v-if="selected === 'Oldest'" class="size-4" />
          <SortDescIcon v-else class="size-4" />{{ selected }}</span
        >
      </DropdownSelect>
    </div>
  </div>
  <div class="mt-4 flex flex-col gap-2">
    <ReportCard :key="report.id" v-for="report in paginatedReports" :report="report" />
    <div
      v-if="!paginatedReports || paginatedReports.length === 0"
      class="universal-card h-24 animate-pulse"
    ></div>
  </div>
  <div v-if="totalPages > 1" class="mt-4 flex justify-center">
    <Pagination :page="currentPage" :count="totalPages" @switch-page="goToPage" />
  </div>
</template>

<script setup lang="ts">
import { DropdownSelect, Button, Pagination } from "@modrinth/ui";
import { XIcon, SearchIcon, SortAscIcon, SortDescIcon, FilterIcon } from "@modrinth/assets";
import { defineMessages, useVIntl } from "@vintl/vintl";
import { useLocalStorage } from "@vueuse/core";
import ReportCard from "~/components/ui/moderation/ReportCard.vue";
import type {
  Project,
  Report,
  Thread,
  User,
  Version,
  TeamMember,
  Organization,
} from "@modrinth/utils";
import { asEncodedJsonArray, fetchSegmented } from "~/utils/fetch-helpers.ts";
import Fuse from "fuse.js";

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
});

export interface OwnershipTarget {
  name: string;
  slug: string;
  avatar_url?: string;
  type: "user" | "organization";
}

export interface ExtendedReport extends Report {
  thread: Thread;
  reporter_user: User;
  project?: Project;
  user?: User;
  version?: Version;
  target?: OwnershipTarget;
}

const { data: allReports } = await useAsyncData("moderation-reports", async () => {
  const reports = (await useBaseFetch("report?all=true&count=10000", {
    apiVersion: 3,
  })) as Report[];

  const threadIDs = reports.map((report) => report.thread_id).filter(Boolean);
  const threads = (await fetchSegmented(
    threadIDs,
    (ids) => `threads?ids=${asEncodedJsonArray(ids)}`,
  )) as Thread[];

  const userIDs = reports
    .filter((report) => report.item_type === "user")
    .map((report) => report.item_id);
  const versionIDs = reports
    .filter((report) => report.item_type === "version")
    .map((report) => report.item_id);
  const projectIDs = reports
    .filter((report) => report.item_type === "project")
    .map((report) => report.item_id);

  const versions = (await fetchSegmented(
    versionIDs,
    (ids) => `versions?ids=${asEncodedJsonArray(ids)}`,
  )) as Version[];

  const fullProjectIds = new Set([
    ...projectIDs,
    ...versions.map((v) => v.project_id).filter(Boolean),
  ]);

  const projects = (await fetchSegmented(
    Array.from(fullProjectIds),
    (ids) => `projects?ids=${asEncodedJsonArray(ids)}`,
  )) as Project[];

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

  const orgTeamIds = orgsData.map((org) => org.team_id).filter(Boolean);
  const orgTeamsData: TeamMember[][] =
    orgTeamIds.length > 0
      ? await fetchSegmented(orgTeamIds, (ids) => `teams?ids=${asEncodedJsonArray(ids)}`)
      : [];

  const ownerUserIds = new Set<string>();

  teamsData.flat().forEach((member) => {
    if (member.role === "Owner") {
      ownerUserIds.add(member.user.id);
    }
  });

  orgTeamsData.flat().forEach((member) => {
    if (member.role === "Owner") {
      ownerUserIds.add(member.user.id);
    }
  });

  const fullUserIds = new Set([
    ...userIDs,
    ...reports.map((report) => report.reporter),
    ...ownerUserIds,
  ]);

  const users = (await fetchSegmented(
    Array.from(fullUserIds),
    (ids) => `users?ids=${asEncodedJsonArray(ids)}`,
  )) as User[];

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

  orgTeamsData.forEach((team) => {
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

  const extendedReports: ExtendedReport[] = reports.map((report) => {
    const thread = threads.find((t) => t.id === report.thread_id) || ({} as Thread);
    const version =
      report.item_type === "version"
        ? versions.find((v: { id: string }) => v.id === report.item_id)
        : undefined;

    const project =
      report.item_type === "project"
        ? projects.find((p: { id: string }) => p.id === report.item_id)
        : report.item_type === "version" && version
          ? projects.find((p: { id: string }) => p.id === version.project_id)
          : undefined;

    let target: OwnershipTarget | undefined;

    if (report.item_type === "user") {
      const targetUser = users.find((u: { id: string }) => u.id === report.item_id);
      if (targetUser) {
        target = {
          name: targetUser.username,
          slug: targetUser.username,
          avatar_url: targetUser.avatar_url,
          type: "user",
        };
      }
    } else if (project) {
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

      // Prioritize organization over individual owner
      if (org) {
        target = {
          name: org.name,
          avatar_url: org.icon_url,
          type: "organization",
          slug: org.slug,
        };
      } else if (owner) {
        target = {
          name: owner.user.username,
          avatar_url: owner.user.avatar_url,
          type: "user",
          slug: owner.user.username,
        };
      }
    }

    return {
      ...report,
      thread,
      reporter_user: users.find((user) => user.id === report.reporter) || ({} as User),
      project,
      user:
        report.item_type === "user"
          ? users.find((u: { id: string }) => u.id === report.item_id)
          : undefined,
      version,
      target,
    };
  });

  return extendedReports;
});

const query = useLocalStorage("moderation-reports-query", "");
const currentFilterType = useLocalStorage("moderation-reports-filter-type", () => "All");
const filterTypes: readonly string[] = readonly(["All", "Unread", "Read"]);

const currentSortType = useLocalStorage("moderation-reports-sort-type", () => "Oldest");
const sortTypes: readonly string[] = readonly(["Oldest", "Newest"]);

const currentPage = ref(1);
const itemsPerPage = 15;
const totalPages = computed(() => Math.ceil((filteredReports.value?.length || 0) / itemsPerPage));

const fuse = computed(() => {
  if (!allReports.value || allReports.value.length === 0) return null;
  return new Fuse(allReports.value, {
    keys: [
      "body",
      "report_type",
      "reporter_user.username",
      "item_id",
      "project.name",
      "user.username",
      "version.name",
    ],
    includeScore: true,
    threshold: 0.4,
  });
});

const filteredReports = computed(() => {
  if (!allReports.value) return [];

  let filtered;

  if (query.value && fuse.value) {
    const results = fuse.value.search(query.value);
    filtered = results.map((result) => result.item);
  } else {
    filtered = [...allReports.value];
  }

  if (currentFilterType.value !== "All") {
    filtered = filtered.filter((report) => {
      const messages = report.thread?.messages ?? [];
      if (messages.length === 0) return false;
      const lastMessage = messages[messages.length - 1];
      if (currentFilterType.value === "Read") {
        return (
          lastMessage.author_id &&
          report.thread.members.some(
            (member) => member.id === lastMessage.author_id && member.role === "moderator",
          )
        );
      } else if (currentFilterType.value === "Unread") {
        return (
          lastMessage.author_id &&
          report.thread.members.some(
            (member) => member.id === lastMessage.author_id && member.role !== "moderator",
          )
        );
      }
      return true;
    });
  }

  if (currentSortType.value === "Oldest") {
    filtered.sort((a, b) => {
      const dateA = new Date(a.created).getTime();
      const dateB = new Date(b.created).getTime();
      return dateA - dateB;
    });
  } else {
    filtered.sort((a, b) => {
      const dateA = new Date(a.created).getTime();
      const dateB = new Date(b.created).getTime();
      return dateB - dateA;
    });
  }

  return filtered;
});

const paginatedReports = computed(() => {
  if (!filteredReports.value) return [];
  const start = (currentPage.value - 1) * itemsPerPage;
  const end = start + itemsPerPage;
  return filteredReports.value.slice(start, end);
});

function updateSearchResults() {
  currentPage.value = 1;
}

function goToPage(page: number) {
  currentPage.value = page;
}
</script>
