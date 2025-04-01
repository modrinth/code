<template>
  <div class="content px-0">
    <div class="mb-3 flex">
      <VersionFilterControl
        :versions="projectVersions"
        :game-versions="tags.gameVersions"
        @update:query="updateQuery"
      />
      <Pagination
        :page="currentPage"
        :count="pageCount"
        class="ml-auto mt-auto"
        :link-function="(page) => `?page=${page}`"
        @switch-page="switchPage"
      />
    </div>
    <div class="universal-card">
      <div v-if="filteredVersions.length" class="display-mode--list">
        <ProjectCard
          v-for="project in sortedAndPaginatedMods"
          :id="project.slug"
          :key="project.id"
          :type="project.project_type"
          :name="project.title"
          :description="project.description"
          :icon-url="project.icon_url"
          :downloads="project.downloads?.toString()"
          :follows="project.followers?.toString()"
          :client-side="project.client_side"
          :server-side="project.server_side"
          :categories="project.categories"
          :status="project.status"
          :created-at="project.published"
          :updated-at="project.updated"
          :color="project.color"
        />
      </div>
      <div v-else class="empty-state">
        {{ formatMessage(messages.noResults) }}
      </div>
    </div>
    <Pagination
      :page="currentPage"
      :count="pageCount"
      class="mb-2 flex justify-end"
      :link-function="(page) => `?page=${page}`"
      @switch-page="switchPage"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { Pagination, VersionFilterControl } from "@modrinth/ui";
import type { Project, Version, VersionFileHash, VersionChannel } from "@modrinth/utils";
import { useVIntl, defineMessages } from "@vintl/vintl";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import { getArrayOrString } from "~/composables/query.js";
import { useTags } from "~/composables/tag.js";
import { useNativeRouter, useNativeRoute } from "~/composables/nuxt-bugfest.ts";

const props = defineProps<{
  dependencies: {
    projects: Project[];
  };
}>();

const router = useNativeRouter();
const route = useNativeRoute();
const tags = useTags();

const currentPage = ref(Number(route.query.page ?? 1));
const pageSize = ref(20);

const projectVersions = computed((): Version[] =>
  (props.dependencies.projects ?? []).map((project) => ({
    id: project.id,
    name: project.title,
    version_number: project.versions[0] ?? "1.0.0",
    game_versions: project.game_versions,
    loaders: project.loaders,
    version_type: "release" as VersionChannel,
    date_published: project.published,
    downloads: project.downloads,
    files: [
      {
        hashes: [{ sha512: "", sha1: "" }] as VersionFileHash[],
        url: project.source_url ?? "",
        filename: project.slug,
        primary: true,
        size: 0,
      },
    ],
    featured: false,
    status: "listed",
    project_id: project.id,
    author_id: project.team,
    dependencies: [],
  })),
);

const filteredVersions = computed(() => {
  const selectedGameVersions = getArrayOrString(route.query.g) ?? [];
  const selectedLoaders = getArrayOrString(route.query.l) ?? [];

  if (!props.dependencies.projects) return [];

  return props.dependencies.projects.filter(
    (project) =>
      (selectedGameVersions.length === 0 ||
        selectedGameVersions.some((gameVersion: string) =>
          project.game_versions?.includes(gameVersion),
        )) &&
      (selectedLoaders.length === 0 ||
        selectedLoaders.some((loader: string) => project.loaders?.includes(loader))),
  );
});

const sortedAndPaginatedMods = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return filteredVersions.value.slice(start, end);
});

const pageCount = computed(() => Math.ceil((filteredVersions.value?.length || 0) / pageSize.value));

interface QueryUpdate {
  page?: number | undefined;
  g?: string[] | undefined;
  l?: string[] | undefined;
}

function updateQuery(newQueries: QueryUpdate) {
  if (newQueries.page) {
    currentPage.value = Number(newQueries.page);
  } else if (newQueries.page === undefined) {
    currentPage.value = 1;
  }

  router.replace({
    query: {
      ...route.query,
      ...newQueries,
    },
  });
}

function switchPage(page: number) {
  currentPage.value = page;
  router.replace({
    query: {
      ...route.query,
      page: currentPage.value !== 1 ? currentPage.value : undefined,
    },
  });
  window.scrollTo({ top: 0, behavior: "smooth" });
}

const vintl = useVIntl();
const { formatMessage } = vintl;

const messages = defineMessages({
  noResults: {
    id: "pages.project.mods.no_results",
    defaultMessage: "No dependencies match the selected filters.",
  },
});
</script>

<style lang="scss" scoped>
.content {
  margin: 0 auto;
  max-width: var(--content-width-lg);
  width: 100%;
}

.px-0 {
  padding-left: 0;
  padding-right: 0;
}

.mb-3 {
  margin-bottom: 0.75rem;
}

.mb-2 {
  margin-bottom: 0.5rem;
}

.mt-auto {
  margin-top: auto;
}

.display-mode--list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-md);
}

.flex {
  display: flex;
}

.justify-end {
  justify-content: flex-end;
}

.pagination-after {
  margin-top: var(--spacing-card-sm);
}

.gap-2 {
  gap: 0.5rem;
}

.ml-auto {
  margin-left: auto;
}

.max-w-\[16rem\] {
  max-width: 16rem;
}

.max-w-\[9rem\] {
  max-width: 9rem;
}

.font-semibold {
  font-weight: 600;
}

.text-primary {
  color: var(--color-text);
}

.text-secondary {
  color: var(--color-text-muted);
}

.p-6 {
  padding: 1.5rem;
}

.flex-col {
  flex-direction: column;
}

.gap-6 {
  gap: 1.5rem;
}

.items-center {
  align-items: center;
}

.whitespace-nowrap {
  white-space: nowrap;
}

.min-w-\[12rem\] {
  min-width: 12rem;
}

.max-w-\[20rem\] {
  max-width: 20rem;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-card-lg);
  color: var(--color-text-muted);
  text-align: center;
}
</style>
