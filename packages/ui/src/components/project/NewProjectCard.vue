<template>
  <SmartClickable
    :style="{ '--_accent-color': experimentalColors && project.color ? rgbToOklchHue(project.color) : 0 }"
    class="project-card pointer-events-none w-full rounded-xl p-4 bg-bg-raised xl:grid grid-cols-[auto_1fr] gap-3 relative overflow-clip"
    :class="{ 'experimental-colors': experimentalColors }"
  >
    <template #clickable>
      <AutoLink :to="link" class="rounded-xl no-click-animation" />
    </template>
    <div v-if="experimentalColors" class="project-card__bg absolute inset-0 rounded-xl z-[-1]"></div>
    <div class="hidden xl:flex">
      <Avatar :src="project.icon_url" size="96px" />
    </div>
    <div class="grid grid-rows-[1fr_auto] gap-2">
      <div class="grid grid-cols-[auto_1fr_auto] xl:grid-cols-[1fr_auto] gap-3">
        <div class="xl:hidden">
          <Avatar :src="project.icon_url" size="64px" />
        </div>
        <div class="flex flex-col gap-2">
          <div class="gap-2 overflow-x-clip text-ellipsis break-all line-clamp-1">
            <span
              class="smart-clickable:underline-on-hover text-lg font-extrabold text-contrast m-0 leading-none"
            >
              {{ project.title }}
            </span>
            <span v-if="creator" class="text-secondary">
              by
              <AutoLink
                :to="creatorLink"
                class="-outline-offset-1 text-inherit"
                :class="{
                  'smart-clickable:allow-pointer-events hover:text-primary hover:underline focus:underline':
                    !!creatorLink,
                }"
              >
                {{ creator }}
              </AutoLink>
            </span>
          </div>
          <div class="m-0 line-clamp-2 xl:line-clamp-2 overflow-hidden">
            {{ project.description }}
          </div>
        </div>
        <div v-if="!!$slots.actions" class="flex flex-col gap-2 items-end">
          <div class="smart-clickable:allow-pointer-events flex items-center gap-1">
            <slot name="actions" />
          </div>
          <div class="flex items-center gap-2 font-medium my-auto">
            <div
              v-tooltip="`${formattedDownloads} downloads`"
              class="smart-clickable:allow-pointer-events flex items-center gap-2 pr-3 border-0 border-r-[1px] border-solid border-button-border cursor-help"
            >
              <DownloadIcon class="shrink-0" />
              <span>
                {{ abbreviatedDownloads }}
              </span>
            </div>
            <div
              v-tooltip="`${formattedFollowers} followers`"
              class="smart-clickable:allow-pointer-events flex items-center gap-2 cursor-help"
            >
              <HeartIcon class="shrink-0" />
              <span>
                {{ abbreviatedFollowers }}
              </span>
            </div>
          </div>
        </div>
        <div v-else class="flex flex-col gap-2 items-end font-medium">
          <div
            v-tooltip="`${formattedDownloads}`"
            class="smart-clickable:allow-pointer-events flex items-center gap-1 cursor-help"
          >
            <DownloadIcon class="shrink-0" />
            <span class="text-secondary">
              <IntlFormatted :message-id="commonMessages.downloadsStat">
                <template #~count>
                  <span class="text-primary">
                    {{ abbreviatedDownloads }}
                  </span>
                </template>
              </IntlFormatted>
            </span>
          </div>
          <div
            v-tooltip="`${formattedFollowers}`"
            class="smart-clickable:allow-pointer-events flex items-center gap-1 cursor-help"
          >
            <HeartIcon class="shrink-0" />
            <span class="text-secondary">
              <IntlFormatted :message-id="commonMessages.followersStat">
                <template #~count>
                  <span class="text-primary">
                    {{ abbreviatedFollowers }}
                  </span>
                </template>
              </IntlFormatted>
            </span>
          </div>
        </div>
      </div>
      <div>
        <div
class="mt-auto grid items-center gap-2"
        :class="environment ? `grid-cols-[auto_auto_auto_1fr_auto]` : `grid-cols-[auto_1fr_auto]`"
        >
          <template v-if="environment">
            <TagItem>
              <component :is="environment.icon" />
              {{ formatMessage(environment.message) }}
            </TagItem>
            <div class="w-[1px] h-5 bg-button-border"></div>
          </template>
          <TagsIcon class="shrink-0" />
          <div class="flex flex-wrap h-6 overflow-hidden items-center gap-1 truncate w-fit">
            <TagItem v-for="tag in categories" :key="tag">
              {{ formatMessage(getCategoryMessage(tag)) }}
            </TagItem>
            <TagItem
              v-for="platform in platforms"
              :key="platform"
              :style="`--_color: var(--color-platform-${platform})`"
            >
              <component :is="getPlatformIcon(platform)" />
              {{ formatMessage(getPlatformMessage(platform)) }}
            </TagItem>
          </div>
          <div
            v-tooltip="
              formatMessage(commonMessages.dateAtTimeTooltip, {
                date: new Date(date),
                time: new Date(date),
              })
            "
            class="smart-clickable:allow-pointer-events ml-auto flex items-center gap-1 font-medium cursor-help"
          >
            <HistoryIcon class="shrink-0" />
            <span class="text-secondary">
              <IntlFormatted :message-id="dateType === 'updated' ? commonMessages.updatedDate : commonMessages.publishedDate">
                <template #~date>
                  <span class="text-primary">
                    {{ dayjs(date).fromNow() }}
                  </span>
                </template>
              </IntlFormatted>
            </span>
          </div>
        </div>
      </div>
    </div>
  </SmartClickable>
</template>

<script setup lang="ts">
import {
  ClientIcon,
  DownloadIcon,
  GlobeIcon,
  HeartIcon,
  HistoryIcon,
  MonitorSmartphoneIcon,
  ServerIcon,
  TagsIcon,
} from '@modrinth/assets'
import Avatar from '../base/Avatar.vue'
import {
  formatNumber,
  type Project,
  type SearchResult,
  rgbToOklchHue,
} from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { type Component, computed, type Ref } from 'vue'
import TagItem from '../base/TagItem.vue'
import type { Linkish } from '../../utils/link'
import AutoLink from '../base/AutoLink.vue'
import SmartClickable from '../base/SmartClickable.vue'
import { commonMessages } from '../../utils/common-messages'
import { defineMessage, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { IntlFormatted } from '@vintl/vintl/components'
import {
  getCategoryMessage,
  getPlatformIcon,
  getPlatformMessage,
  isPlatformTag
} from '../../utils/tags'

const { formatMessage } = useVIntl()

dayjs.extend(relativeTime)

const props = withDefaults(defineProps<{
  project: Project | SearchResult
  link: Linkish
  creatorLink?: Linkish
  experimentalColors?: boolean
  dateType?: 'updated' | 'published'
}>(), {
  creatorLink: undefined,
  experimentalColors: false,
  dateType: 'updated'
})

const environment: Ref<
  | {
      message: MessageDescriptor
      icon: Component
    }
  | undefined
> = computed(() => {
  const client = props.project.client_side
  const server = props.project.server_side

  if (client === server && client === 'required') {
    return {
      message: defineMessage({
        id: 'project.card.environment.client-and-server',
        defaultMessage: 'Client and server',
      }),
      icon: MonitorSmartphoneIcon as Component,
    }
  } else if (client === 'required' && server === 'unsupported') {
    return {
      message: defineMessage({
        id: 'project.card.environment.client',
        defaultMessage: 'Client',
      }),
      icon: ClientIcon as Component,
    }
  } else if (server === 'required' && client === 'unsupported') {
    return {
      message: defineMessage({
        id: 'project.card.environment.server',
        defaultMessage: 'Server',
      }),
      icon: ServerIcon as Component,
    }
  } else if (client === 'optional' && server === 'optional') {
    return {
      message: defineMessage({
        id: 'project.card.environment.client-or-server',
        defaultMessage: 'Client or server',
      }),
      icon: GlobeIcon as Component,
    }
  }
  return undefined
})

const creator: Ref<string | undefined> = computed(() =>
  isSearchResult(props.project) ? props.project.author : undefined,
)
const followers: Ref<number> = computed(() =>
  isSearchResult(props.project) ? props.project.follows : props.project.followers,
)
const date: Ref<string> = computed(() =>
  props.dateType === 'updated' ? (isSearchResult(props.project) ? props.project.date_modified : props.project.updated) :
    (isSearchResult(props.project) ? props.project.date_created : props.project.approved),
)

const tags: Ref<string[]> = computed(() =>
  isSearchResult(props.project)
    ? [...props.project.categories]
    : [...props.project.categories, ...props.project.additional_categories, ...props.project.loaders],
)
const categories: Ref<string[]> = computed(() => tags.value.filter((tag) => !isPlatformTag(tag)))
const platforms: Ref<string[]> = computed(() => tags.value.filter(isPlatformTag))

const formattedDownloads = computed(() => formatNumber(props.project.downloads, false))
const formattedFollowers = computed(() => formatNumber(followers.value, false))

const abbreviatedDownloads = computed(() => formatNumber(props.project.downloads))
const abbreviatedFollowers = computed(() => formatNumber(followers.value))

function isSearchResult(project: Project | SearchResult): project is SearchResult {
  return 'project_id' in project
}
</script>
<style lang="scss" scoped>

:deep(.experimental-colors.project-card) {
  --color-button-bg: rgba(255, 255, 255, 0.08);
}

.experimental-colors .project-card__bg {
  border: 1px solid var(--color-button-border);
  background-image: linear-gradient(
      to top right,
      oklch(30% 25% var(--_accent-color) / 15%) 50%,
      oklch(10% 20% var(--_accent-color) / 25%)
  );
  opacity: 1;
}
</style>
