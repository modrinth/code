<template>
  <section class="experimental-styles-within overflow-visible">
    <div
      v-if="currentMember && isPermission(currentMember?.permissions, 1 << 0)"
      class="card flex items-center gap-4"
    >
      <FileInput
        :max-size="524288000"
        :accept="acceptFileFromProjectType(project.project_type)"
        prompt="Upload a version"
        class="btn btn-primary"
        aria-label="Upload a version"
        @change="handleFiles"
      >
        <UploadIcon aria-hidden="true" />
      </FileInput>
      <span class="flex items-center gap-2">
        <InfoIcon aria-hidden="true" /> Click to choose a file or drag one onto this page
      </span>
      <DropArea :accept="acceptFileFromProjectType(project.project_type)" @change="handleFiles" />
    </div>
    <div class="mb-3 flex flex-wrap gap-2">
      <VersionFilterControl
        ref="versionFilters"
        :versions="props.versions"
        @switch-page="switchPage"
      />
      <Pagination
        :page="currentPage"
        class="ml-auto mt-auto"
        :count="Math.ceil(filteredVersions.length / 20)"
        :link-function="(page) => `?page=${currentPage}`"
        @switch-page="switchPage"
      />
    </div>
    <div
      v-if="versions.length > 0"
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
      <template
        v-for="(version, index) in filteredVersions.slice((currentPage - 1) * 20, currentPage * 20)"
        :key="index"
      >
        <div
          :class="`versions-grid-row h-px w-full bg-button-bg ${index === 0 ? `max-sm:!hidden` : ``}`"
        ></div>
        <div class="versions-grid-row group relative">
          <nuxt-link
            class="absolute inset-[calc(-1rem-2px)_-2rem] before:absolute before:inset-0 before:transition-all before:content-[''] hover:before:backdrop-brightness-110"
            :to="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/version/${encodeURI(version.displayUrlEnding)}`"
          ></nuxt-link>
          <div class="flex flex-col justify-center gap-2 sm:contents">
            <div class="flex flex-row items-center gap-2 sm:contents">
              <div class="self-center">
                <div class="relative z-[1] cursor-pointer">
                  <VersionChannelIndicator
                    v-tooltip="`Toggle filter for ${version.version_type}`"
                    :channel="version.version_type"
                    @click="versionFilters.toggleFilter('channel', version.version_type)"
                  />
                </div>
              </div>
              <div
                class="pointer-events-none relative z-[1] flex flex-col justify-center group-hover:underline"
              >
                <div class="font-bold text-contrast">{{ version.version_number }}</div>
                <div class="text-xs font-medium">{{ version.name }}</div>
              </div>
            </div>
            <div class="flex flex-col justify-center gap-2 sm:contents">
              <div class="flex flex-row flex-wrap items-center gap-1 xl:contents">
                <div class="flex items-center">
                  <div class="tag-list">
                    <div
                      v-for="gameVersion in formatVersionsForDisplay(version.game_versions)"
                      :key="`version-tag-${gameVersion}`"
                      v-tooltip="`Toggle filter for ${gameVersion}`"
                      class="tag-list__item z-[1] cursor-pointer hover:underline"
                      @click="versionFilters.toggleFilters('gameVersion', version.game_versions)"
                    >
                      {{ gameVersion }}
                    </div>
                  </div>
                </div>
                <div class="flex items-center">
                  <div class="tag-list">
                    <div
                      v-for="platform in version.loaders"
                      :key="`platform-tag-${platform}`"
                      v-tooltip="`Toggle filter for ${platform}`"
                      :class="`tag-list__item z-[1] cursor-pointer hover:underline`"
                      :style="`--_color: var(--color-platform-${platform})`"
                      @click="versionFilters.toggleFilter('platform', platform)"
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
                  v-tooltip="
                    formatMessage(commonMessages.dateAtTimeTooltip, {
                      date: new Date(version.date_published),
                      time: new Date(version.date_published),
                    })
                  "
                  class="z-[1] flex cursor-help items-center gap-1 text-nowrap font-medium xl:self-center"
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
              <a
                v-tooltip="`Download`"
                :href="getPrimaryFile(version).url"
                class="z-[1] group-hover:!bg-brand group-hover:!text-brand-inverted"
                aria-label="Download"
                @click="emits('onDownload')"
              >
                <DownloadIcon aria-hidden="true" />
              </a>
            </ButtonStyled>
            <ButtonStyled circular type="transparent">
              <OverflowMenu
                class="group-hover:!bg-button-bg"
                :options="[
                  {
                    id: 'download',
                    color: 'primary',
                    hoverFilled: true,
                    link: getPrimaryFile(version).url,
                    action: () => {
                      emits('onDownload');
                    },
                  },
                  {
                    id: 'new-tab',
                    action: () => {},
                    link: `/${project.project_type}/${
                      project.slug ? project.slug : project.id
                    }/version/${encodeURI(version.displayUrlEnding)}`,
                    external: true,
                  },
                  {
                    id: 'copy-link',
                    action: () =>
                      copyToClipboard(
                        `https://modrinth.com/${project.project_type}/${
                          project.slug ? project.slug : project.id
                        }/version/${encodeURI(version.displayUrlEnding)}`,
                      ),
                  },
                  {
                    id: 'share',
                    action: () => {},
                    shown: false,
                  },
                  {
                    id: 'report',
                    color: 'red',
                    hoverFilled: true,
                    action: () => reportVersion(version.id),
                  },
                  { divider: true, shown: currentMember },
                  {
                    id: 'edit',
                    link: `/${project.project_type}/${
                      project.slug ? project.slug : project.id
                    }/version/${encodeURI(version.displayUrlEnding)}/edit`,
                    shown: currentMember,
                  },
                  {
                    id: 'delete',
                    color: 'red',
                    hoverFilled: true,
                    action: () => {},
                    shown: currentMember && false,
                  },
                ]"
                aria-label="More options"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #download>
                  <DownloadIcon aria-hidden="true" />
                  Download
                </template>
                <template #new-tab>
                  <ExternalIcon aria-hidden="true" />
                  Open in new tab
                </template>
                <template #copy-link>
                  <LinkIcon aria-hidden="true" />
                  Copy link
                </template>
                <template #share>
                  <ShareIcon aria-hidden="true" />
                  Share
                </template>
                <template #report>
                  <ReportIcon aria-hidden="true" />
                  Report
                </template>
                <template #edit>
                  <EditIcon aria-hidden="true" />
                  Edit
                </template>
                <template #delete>
                  <TrashIcon aria-hidden="true" />
                  Delete
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </div>
          <div
            v-if="flags.showVersionFilesInTable"
            class="tag-list pointer-events-none relative z-[1] col-span-full"
          >
            <div
              v-for="(file, fileIdx) in version.files"
              :key="`platform-tag-${fileIdx}`"
              :class="`flex items-center gap-1 text-wrap rounded-full bg-button-bg px-2 py-0.5 text-xs font-medium ${file.primary || fileIdx === 0 ? 'bg-brand-highlight text-contrast' : 'text-primary'}`"
            >
              <StarIcon v-if="file.primary || fileIdx === 0" class="shrink-0" />
              {{ file.filename }} - {{ formatBytes(file.size) }}
            </div>
          </div>
        </div>
      </template>
    </div>
    <div class="my-3 flex justify-end">
      <Pagination
        :page="currentPage"
        :count="Math.ceil(filteredVersions.length / 20)"
        :link-function="(page) => `?page=${currentPage}`"
        @switch-page="switchPage"
      />
    </div>
  </section>
</template>

<script setup>
import {
  ButtonStyled,
  OverflowMenu,
  Pagination,
  VersionChannelIndicator,
  FileInput,
} from "@modrinth/ui";
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
  UploadIcon,
  InfoIcon,
} from "@modrinth/assets";
import { formatBytes, formatCategory } from "@modrinth/utils";
import { formatVersionsForDisplay } from "~/helpers/projects.js";
import VersionFilterControl from "~/components/ui/VersionFilterControl.vue";
import DropArea from "~/components/ui/DropArea.vue";
import { acceptFileFromProjectType } from "~/helpers/fileUtils.js";

const formatCompactNumber = useCompactNumber();
const { formatMessage } = useVIntl();

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
  currentMember: {
    type: Object,
    default() {
      return null;
    },
  },
});

const tags = useTags();
const flags = useFeatureFlags();
const formatRelativeTime = useRelativeTime();

const emits = defineEmits(["onDownload"]);

const route = useNativeRoute();
const router = useNativeRouter();

const currentPage = ref(route.query.page ?? 1);

function switchPage(page) {
  currentPage.value = page;

  router.replace({
    query: {
      ...route.query,
      page: currentPage.value !== 1 ? currentPage.value : undefined,
    },
  });
}

function getPrimaryFile(version) {
  return version.files.find((x) => x.primary) || version.files[0];
}

const selectedGameVersions = computed(() => {
  return getArrayOrString(route.query.g) ?? [];
});

const selectedPlatforms = computed(() => {
  return getArrayOrString(route.query.l) ?? [];
});

const selectedVersionChannels = computed(() => {
  return getArrayOrString(route.query.c) ?? [];
});

const versionFilters = ref(null);
const filteredVersions = computed(() => {
  return props.versions.filter(
    (projectVersion) =>
      (selectedGameVersions.value.length === 0 ||
        selectedGameVersions.value.some((gameVersion) =>
          projectVersion.game_versions.includes(gameVersion),
        )) &&
      (selectedPlatforms.value.length === 0 ||
        selectedPlatforms.value.some((loader) => projectVersion.loaders.includes(loader))) &&
      (selectedVersionChannels.value.length === 0 ||
        selectedVersionChannels.value.includes(projectVersion.version_type)),
  );
});

async function handleFiles(files) {
  await router.push({
    name: "type-id-version-version",
    params: {
      type: props.project.project_type,
      id: props.project.slug ? props.project.slug : props.project.id,
      version: "create",
    },
    state: {
      newPrimaryFile: files[0],
    },
  });
}

async function copyToClipboard(text) {
  await navigator.clipboard.writeText(text);
}
</script>
<style scoped>
.versions-grid-row {
  @apply grid grid-cols-[1fr_min-content] gap-4 supports-[grid-template-columns:subgrid]:col-span-full supports-[grid-template-columns:subgrid]:!grid-cols-subgrid sm:grid-cols-[min-content_1fr_1fr_1fr_min-content] xl:grid-cols-[min-content_1fr_1fr_1fr_1fr_1fr_min-content];
}
</style>
