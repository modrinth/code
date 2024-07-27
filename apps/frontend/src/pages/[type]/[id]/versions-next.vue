<template>
  <section class='normal-page__content experimental-styles-within overflow-visible'>
    <div class='flex gap-4 items-center pb-5'>
      <button class='btn' @click='page--'>Prev</button>
      <span>{{ page }}</span>
      <button class='btn' @click='page++'>Next</button>
    </div>
    <div
      class='grid grid-cols-[1fr_min-content] sm:grid-cols-[min-content_auto_auto_auto_min-content] xl:grid-cols-[min-content_auto_auto_auto_auto_auto_min-content] gap-4 bg-bg-raised px-6 sm:px-8 py-4 rounded-2xl'>
      <div class='max-sm:hidden'></div>
      <div class='max-sm:hidden text-contrast text-sm font-bold'>Name</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold sm:max-xl:collapse sm:max-xl:hidden'>Game version</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold sm:max-xl:collapse sm:max-xl:hidden'>Platforms</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold sm:max-xl:collapse sm:max-xl:hidden'>Published</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold sm:max-xl:collapse sm:max-xl:hidden'>Downloads</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold xl:collapse xl:hidden'>Compatibility</div>
      <div class='max-sm:hidden text-contrast text-sm font-bold xl:collapse xl:hidden'>Stats</div>
      <div class='max-sm:hidden'></div>
      <template v-for='(version, index) in versionsPage'>
        <div
          :class='`col-span-2 sm:col-span-5 xl:col-span-7 w-full h-px bg-button-bg ${index === 0 ? `max-sm:hidden` : ``}`'></div>
        <div class='flex flex-col gap-2 justify-center sm:contents'>
          <div class="flex flex-row gap-2 items-center sm:contents">
            <div class='self-center'>
              <div
                  :class="`flex text-sm font-bold justify-center items-center w-8 h-8 rounded-full ${version.version_type === 'release' ? 'bg-bg-green text-brand-green' : version.version_type === 'beta' ? 'bg-bg-orange text-brand-orange' : 'bg-bg-red text-brand-red'}`">
                {{ formatMessage(channelMessages[`${version.version_type}Symbol`]) }}
              </div>
            </div>
            <div class='flex flex-col justify-center'>
              <div class='text-contrast font-bold'>{{ version.version_number }}</div>
              <div class='text-xs font-medium'>{{ version.name }}</div>
            </div>
          </div>
          <div class='flex flex-col gap-2 justify-center sm:contents'>
            <div class='flex flex-row flex-wrap gap-1 items-center xl:contents'>
              <div class='items-center'>
                <div class='tag-list'>
                  <div
                      v-for='version in formatVersionsForDisplay(version.game_versions)'
                      :key='`version-tag-${version}`'
                      class='tag-list__item'
                  >
                    {{ version }}
                  </div>
                </div>
              </div>
              <div class='items-center'>
                <div class='tag-list'>
                  <div
                      v-for='platform in version.loaders'
                      :key='`platform-tag-${platform}`'
                      :class='`tag-list__item`'
                      :style='`--_color: var(--color-platform-${platform})`'
                  >
                    <svg v-html='tags.loaders.find((x) => x.name === platform).icon'></svg>
                    {{ formatCategory(platform) }}
                  </div>
                </div>
              </div>
            </div>
            <div class='flex flex-col gap-1 justify-center xl:contents max-sm:flex-row max-sm:justify-start max-sm:gap-3'>
              <div class='flex gap-1 items-center font-medium xl:self-center text-nowrap'>
                <CalendarIcon class='xl:hidden' />
                {{ formatRelativeTime(version.date_published) }}
              </div>
              <div class='flex gap-1 items-center font-medium xl:self-center'>
                <DownloadIcon class='xl:hidden' />
                {{ formatCompactNumber(version.downloads) }}
              </div>
            </div>
          </div>
        </div>
        <div class='flex items-start justify-end gap-1 sm:items-center'>
          <OverflowMenu
            class='btn icon-only btn-transparent'
            :options="[
              {
                'id': 'download',
                'color': 'primary',
                'hoverFilled': true,
                'action': () => {}
              },
              {
                'id': 'new-tab',
                'action': () => {}
              },
              {
                'id': 'copy-link',
                'action': () => {}
              },
              {
                'id': 'share',
                'action': () => {}
              },
              {
                'id': 'report',
                'color': 'red',
                'hoverFilled': true,
                'action': () => {}
              },
              { divider: true },
              {
                'id': 'edit',
                'action': () => {}
              },
              {
                'id': 'delete',
                'color': 'red',
                'hoverFilled': true,
                'action': () => {}
              },
            ]">
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
        </div>
      </template>
    </div>
  </section>
  <div class='normal-page__sidebar'>
  </div>

</template>

<script setup>
import { CalendarIcon, DownloadIcon, MoreVerticalIcon, TrashIcon, ExternalIcon, LinkIcon, ShareIcon, EditIcon, ReportIcon } from '@modrinth/assets'
import { formatVersionsForDisplay, getVersionsToDisplay } from '~/helpers/projects.js'
import { formatCategory } from '@modrinth/utils'
import { OverflowMenu } from '@modrinth/ui'

const formatCompactNumber = useCompactNumber()

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {}
    }
  },
  versions: {
    type: Array,
    default() {
      return []
    }
  }
})

const tags = useTags()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const page = ref(0)
const versionsPage = computed(() => props.versions.slice(page.value, page.value + 20))

const messages = defineMessages({
  versionName: {
    id: 'project.versions.channel.release.label',
    defaultMessage: 'Release'
  },
  releaseSymbol: {
    id: 'project.versions.channel.release.symbol',
    defaultMessage: 'R'
  },
  beta: {
    id: 'project.versions.channel.beta.label',
    defaultMessage: 'Beta'
  },
  betaSymbol: {
    id: 'project.versions.channel.beta.symbol',
    defaultMessage: 'B'
  },
  alpha: {
    id: 'project.versions.channel.alpha.label',
    defaultMessage: 'Alpha'
  },
  alphaSymbol: {
    id: 'project.versions.channel.alpha.symbol',
    defaultMessage: 'A'
  }
})

const channelMessages = defineMessages({
  release: {
    id: 'project.versions.channel.release.label',
    defaultMessage: 'Release'
  },
  releaseSymbol: {
    id: 'project.versions.channel.release.symbol',
    defaultMessage: 'R'
  },
  beta: {
    id: 'project.versions.channel.beta.label',
    defaultMessage: 'Beta'
  },
  betaSymbol: {
    id: 'project.versions.channel.beta.symbol',
    defaultMessage: 'B'
  },
  alpha: {
    id: 'project.versions.channel.alpha.label',
    defaultMessage: 'Alpha'
  },
  alphaSymbol: {
    id: 'project.versions.channel.alpha.symbol',
    defaultMessage: 'A'
  }
})

</script>
