<template>
  <section class="normal-page__content experimental-styles-within overflow-visible">
    <div
      class="flex flex-col gap-4 rounded-2xl bg-bg-raised px-6 pb-8 pt-4 supports-[grid-template-columns:subgrid]:grid supports-[grid-template-columns:subgrid]:grid-cols-[1fr_min-content] sm:px-8 supports-[grid-template-columns:subgrid]:sm:grid-cols-[min-content_auto_auto_auto_min-content] supports-[grid-template-columns:subgrid]:xl:grid-cols-[min-content_auto_auto_auto_auto_auto_min-content]"
    >
      <div class="versions-grid-row">
        <div class="w-9 max-sm:hidden"></div>
        <div class="text-sm font-bold text-contrast max-sm:hidden">Name</div>
        <div
          class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
        >
          Game version
        </div>
        <div
          class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
        >
          Platforms
        </div>
        <div
          class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
        >
          Published
        </div>
        <div
          class="text-sm font-bold text-contrast max-sm:hidden sm:max-xl:collapse sm:max-xl:hidden"
        >
          Downloads
        </div>
        <div class="text-sm font-bold text-contrast max-sm:hidden xl:collapse xl:hidden">
          Compatibility
        </div>
        <div class="text-sm font-bold text-contrast max-sm:hidden xl:collapse xl:hidden">Stats</div>
        <div class="w-9 max-sm:hidden"></div>
      </div>
      <template v-for="(version, index) in versionsPage">
        <div
          :class="`versions-grid-row h-px w-full bg-button-bg ${index === 0 ? `max-sm:!hidden` : ``}`"
        ></div>
        <div class="versions-grid-row group relative">
          <nuxt-link
            class="absolute inset-[calc(-1rem-2px)_-2rem] before:absolute before:inset-0 before:transition-all before:content-[''] hover:before:backdrop-brightness-110"
            :to="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/version/${encodeURI(version.displayUrlEnding)}`"
          >
          </nuxt-link>
          <div class="flex flex-col justify-center gap-2 sm:contents">
            <div class="flex flex-row items-center gap-2 sm:contents">
              <div class="self-center">
                <div
                  :class="`pointer-events-none relative z-[1] flex h-9 w-9 items-center justify-center rounded-full text-sm font-bold group-hover:hidden ${version.version_type === 'release' ? 'bg-bg-green text-brand-green' : version.version_type === 'beta' ? 'bg-bg-orange text-brand-orange' : 'bg-bg-red text-brand-red'}`"
                >
                  {{ formatMessage(channelMessages[`${version.version_type}Symbol`]) }}
                </div>
                <div class="relative z-[1] hidden group-hover:!flex">
                  <ButtonStyled circular color="brand">
                    <button v-tooltip="`Download`">
                      <DownloadIcon />
                    </button>
                  </ButtonStyled>
                </div>
              </div>
              <div class="pointer-events-none relative z-[1] flex flex-col justify-center">
                <div class="font-bold text-contrast">{{ version.version_number }}</div>
                <div class="text-xs font-medium">{{ version.name }}</div>
              </div>
            </div>
            <div class="flex flex-col justify-center gap-2 sm:contents">
              <div class="flex flex-row flex-wrap items-center gap-1 xl:contents">
                <div class="flex items-center">
                  <div class="tag-list">
                    <div
                      v-for="version in formatVersionsForDisplay(version.game_versions)"
                      :key="`version-tag-${version}`"
                      v-tooltip="`Add filter for ${version}`"
                      class="tag-list__item z-[1] cursor-pointer hover:underline"
                    >
                      {{ version }}
                    </div>
                  </div>
                </div>
                <div class="flex items-center">
                  <div class="tag-list">
                    <div
                      v-for="platform in version.loaders"
                      :key="`platform-tag-${platform}`"
                      v-tooltip="`Add filter for ${platform}`"
                      :class="`tag-list__item z-[1] cursor-pointer hover:underline`"
                      :style="`--_color: var(--color-platform-${platform})`"
                    >
                      <svg v-html="tags.loaders.find((x) => x.name === platform).icon"></svg>
                      {{ formatCategory(platform) }}
                    </div>
                  </div>
                </div>
              </div>
              <div
                class="flex flex-col justify-center gap-1 max-sm:flex-row max-sm:justify-start max-sm:gap-3 xl:contents"
              >
                <div
                  class="pointer-events-none z-[1] flex items-center gap-1 text-nowrap font-medium xl:self-center"
                >
                  <CalendarIcon class="xl:hidden" />
                  {{ formatRelativeTime(version.date_published) }}
                </div>
                <div
                  class="pointer-events-none z-[1] flex items-center gap-1 font-medium xl:self-center"
                >
                  <DownloadIcon class="xl:hidden" />
                  {{ formatCompactNumber(version.downloads) }}
                </div>
              </div>
            </div>
          </div>
          <div class="flex items-start justify-end gap-1 sm:items-center">
            <ButtonStyled circular type="transparent">
              <OverflowMenu
                class="group-hover:!bg-button-bg"
                :options="[
                  {
                    id: 'download',
                    color: 'primary',
                    hoverFilled: true,
                    action: () => {},
                  },
                  {
                    id: 'new-tab',
                    action: () => {},
                  },
                  {
                    id: 'copy-link',
                    action: () => {},
                  },
                  {
                    id: 'share',
                    action: () => {},
                  },
                  {
                    id: 'report',
                    color: 'red',
                    hoverFilled: true,
                    action: () => {},
                  },
                  { divider: true },
                  {
                    id: 'edit',
                    action: () => {},
                  },
                  {
                    id: 'delete',
                    color: 'red',
                    hoverFilled: true,
                    action: () => {},
                  },
                ]"
              >
                <MoreVerticalIcon />
                <template #download>
                  <DownloadIcon />
                  Download
                </template>
                <template #new-tab>
                  <ExternalIcon />
                  Open in new tab
                </template>
                <template #copy-link>
                  <LinkIcon />
                  Copy link
                </template>
                <template #share>
                  <ShareIcon />
                  Share
                </template>
                <template #report>
                  <ReportIcon />
                  Report
                </template>
                <template #edit>
                  <EditIcon />
                  Edit
                </template>
                <template #delete>
                  <TrashIcon />
                  Delete
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </div>
          <div v-if="showFiles" class="tag-list pointer-events-none relative z-[1] col-span-full">
            <div
              v-for="(file, index) in version.files"
              :key="`platform-tag-${file}`"
              :class="`flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium ${file.primary || index === 0 ? 'bg-brand-highlight text-contrast' : 'text-primary'}`"
            >
              <StarIcon v-if="file.primary || index === 0" class="shrink-0" />
              {{ file.filename }} - {{ formatBytes(file.size) }}
            </div>
          </div>
        </div>
      </template>
    </div>
    <div class="my-3 flex justify-end">
      <Pagination
        :page="currentPage"
        :count="Math.ceil(versions.length / 20)"
        :link-function="(page) => `?page=${currentPage}`"
        @switch-page="switchPage"
      />
    </div>
  </section>
  <div class="normal-page__sidebar"></div>
</template>

<script setup>
import {
  StarIcon,
  CalendarIcon,
  DownloadIcon,
  MoreVerticalIcon,
  TrashIcon,
  ExternalIcon,
  LinkIcon,
  ShareIcon,
  EditIcon,
  ReportIcon,
} from "@modrinth/assets";
import { formatBytes, formatCategory } from "@modrinth/utils";
import { ButtonStyled, OverflowMenu, Pagination } from "@modrinth/ui";
import { formatVersionsForDisplay, getVersionsToDisplay } from "~/helpers/projects.js";

const formatCompactNumber = useCompactNumber();

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  versions: {
    type: Array,
    default() {
      return [];
    },
  },
});

const tags = useTags();
const { formatMessage } = useVIntl();
const formatRelativeTime = useRelativeTime();

const currentPage = ref(1);
const versionsPage = computed(() =>
  props.versions.slice((currentPage.value - 1) * 20, currentPage.value * 20),
);

const messages = defineMessages({
  versionName: {
    id: "project.versions.channel.release.label",
    defaultMessage: "Release",
  },
  releaseSymbol: {
    id: "project.versions.channel.release.symbol",
    defaultMessage: "R",
  },
  beta: {
    id: "project.versions.channel.beta.label",
    defaultMessage: "Beta",
  },
  betaSymbol: {
    id: "project.versions.channel.beta.symbol",
    defaultMessage: "B",
  },
  alpha: {
    id: "project.versions.channel.alpha.label",
    defaultMessage: "Alpha",
  },
  alphaSymbol: {
    id: "project.versions.channel.alpha.symbol",
    defaultMessage: "A",
  },
});

const channelMessages = defineMessages({
  release: {
    id: "project.versions.channel.release.label",
    defaultMessage: "Release",
  },
  releaseSymbol: {
    id: "project.versions.channel.release.symbol",
    defaultMessage: "R",
  },
  beta: {
    id: "project.versions.channel.beta.label",
    defaultMessage: "Beta",
  },
  betaSymbol: {
    id: "project.versions.channel.beta.symbol",
    defaultMessage: "B",
  },
  alpha: {
    id: "project.versions.channel.alpha.label",
    defaultMessage: "Alpha",
  },
  alphaSymbol: {
    id: "project.versions.channel.alpha.symbol",
    defaultMessage: "A",
  },
});

const showFiles = ref(false);

function switchPage(page) {
  currentPage.value = page;

  router.replace({
    query: {
      ...route.query,
      p: currentPage.value !== 1 ? currentPage.value : undefined,
    },
  });
}
</script>
<style scoped>
.versions-grid-row {
  @apply grid grid-cols-[1fr_min-content] gap-4 supports-[grid-template-columns:subgrid]:col-span-full supports-[grid-template-columns:subgrid]:!grid-cols-subgrid sm:grid-cols-[min-content_1fr_1fr_1fr_min-content] xl:grid-cols-[min-content_1fr_1fr_1fr_1fr_1fr_min-content];
}
</style>
