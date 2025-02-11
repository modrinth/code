<template>
  <div>
    <div class="flex items-center gap-4">
      <div
        class="h-4 w-4 rounded-full border-2 border-solid border-button-border"
        :class="recent || first ? 'bg-brand' : 'bg-button-bg'"
      />
      <AutoLink
        :to="
          hasLink ? `/news/changelog/${entry.product}/${entry.version ?? entry.date.unix()}` : ''
        "
        class="flex items-center gap-2"
        :class="{ 'hover:underline': hasLink }"
      >
        <h2 class="flex items-center gap-2 m-0 text-xl font-extrabold text-contrast">
          <template v-if="showType">
            {{ formatMessage(messages[entry.product]) }}
            <div class="w-2 h-2 rounded-full bg-secondary" />
          </template>
          <span :class="{ 'text-primary font-bold': showType }">
            {{ entry.version ?? formattedDate }}
          </span>
        </h2>
        <div v-if="entry.version" v-tooltip="dateTooltip" :class="{ 'cursor-help': dateTooltip }">
          {{ formattedDate }}
        </div>
      </AutoLink>
    </div>
    <div class="ml-8 mt-3 rounded-2xl bg-bg-raised px-4 py-3">
      <div class="changelog-body" v-html="renderHighlightedString(entry.body)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { VersionEntry } from '@modrinth/utils/changelog'
import { renderHighlightedString } from '@modrinth/utils'
import dayjs from 'dayjs'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { computed, ref } from 'vue'
import AutoLink from '../base/AutoLink.vue'

const { formatMessage } = useVIntl()

const props = withDefaults(
  defineProps<{
    entry: VersionEntry
    showType?: boolean
    first?: boolean
    hasLink?: boolean
  }>(),
  {
    showType: false,
    first: false,
    hasLink: false,
  },
)

const currentDate = ref(dayjs())
const recent = computed(() => props.entry.date.isAfter(currentDate.value.subtract(1, 'week')))
const dateTooltip = computed(() => props.entry.date.format('MMMM D, YYYY [at] h:mm A'))
const formattedDate = computed(() =>
  props.entry.version ? props.entry.date.fromNow() : props.entry.date.format('MMMM D, YYYY'),
)

const messages = defineMessages({
  web: {
    id: 'changelog.product.web',
    defaultMessage: 'Website',
  },
  servers: {
    id: 'changelog.product.servers',
    defaultMessage: 'Servers',
  },
  app: {
    id: 'changelog.product.app',
    defaultMessage: 'App',
  },
  api: {
    id: 'changelog.product.api',
    defaultMessage: 'API',
  },
})
</script>
<style lang="scss" scoped>
:deep(.changelog-body) {
  line-height: 1.4;

  h1,
  h2,
  h3,
  h4,
  h5,
  h6 {
    margin: 0;
  }

  ul {
    padding-left: 1.25rem;
    margin: 0;
  }

  a {
    color: var(--color-link);

    &:hover,
    &:focus-visible {
      filter: brightness(1.2);
      text-decoration: underline;
    }
  }

  code {
    background-color: var(--color-bg);
    font-size: var(--font-size-sm);
    padding: 0.125rem 0.25rem;
    border-radius: 4px;
  }

  p {
    margin: 0;
  }

  * + p {
    margin-top: 0.5rem;
  }

  h3 + * {
    margin-top: 0.5rem;
  }

  * + h3 {
    margin-top: 0.75rem;
  }

  * + li {
    margin-top: 0.5rem;
  }

  li ul li {
    margin-top: 0.25rem;
  }

  img {
    max-width: 100%;
    border-radius: var(--radius-md);
  }
}
</style>
