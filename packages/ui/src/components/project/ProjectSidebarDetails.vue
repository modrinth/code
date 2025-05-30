<template>
  <div class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="flex flex-col gap-3 font-semibold [&>div]:flex [&>div]:gap-2 [&>div]:items-center">
      <div>
        <BookTextIcon aria-hidden="true" />
        <div>
          Licensed
          <a
            v-if="project.license.url"
            class="text-link hover:underline"
            :href="project.license.url"
            :target="linkTarget"
            rel="noopener nofollow ugc"
          >
            {{ licenseIdDisplay }}
            <ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
          </a>
          <span
            v-else-if="
              project.license.id === 'LicenseRef-All-Rights-Reserved' ||
              !project.license.id.includes('LicenseRef')
            "
          >
            {{ licenseIdDisplay }}
          </span>
          <span v-else>{{ licenseIdDisplay }}</span>
        </div>
      </div>
      <div
        v-if="project.approved"
        v-tooltip="dayjs(project.approved).format('MMMM D, YYYY [at] h:mm A')"
      >
        <CalendarIcon aria-hidden="true" />
        <div>
          {{ formatMessage(messages.published, { date: publishedDate }) }}
        </div>
      </div>
      <div v-else v-tooltip="dayjs(project.published).format('MMMM D, YYYY [at] h:mm A')">
        <CalendarIcon aria-hidden="true" />
        <div>
          {{ formatMessage(messages.created, { date: createdDate }) }}
        </div>
      </div>
      <div
        v-if="project.status === 'processing' && project.queued"
        v-tooltip="dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')"
      >
        <ScaleIcon aria-hidden="true" />
        <div>
          {{ formatMessage(messages.submitted, { date: submittedDate }) }}
        </div>
      </div>
      <div
        v-if="hasVersions && project.updated"
        v-tooltip="dayjs(project.updated).format('MMMM D, YYYY [at] h:mm A')"
      >
        <VersionIcon aria-hidden="true" />
        <div>
          {{ formatMessage(messages.updated, { date: updatedDate }) }}
        </div>
      </div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { BookTextIcon, CalendarIcon, ScaleIcon, VersionIcon, ExternalIcon } from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import { computed } from 'vue'
import dayjs from 'dayjs'
import { useRelativeTime } from '../../composables'

const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()

const props = defineProps<{
  project: {
    id: string
    published: string
    updated: string
    approved: string
    queued: string
    status: string
    license: {
      id: string
      url: string
    }
  }
  linkTarget: string
  hasVersions: boolean
}>()

const createdDate = computed(() =>
  props.project.published ? formatRelativeTime(props.project.published) : 'unknown',
)
const submittedDate = computed(() =>
  props.project.queued ? formatRelativeTime(props.project.queued) : 'unknown',
)
const publishedDate = computed(() =>
  props.project.approved ? formatRelativeTime(props.project.approved) : 'unknown',
)
const updatedDate = computed(() =>
  props.project.updated ? formatRelativeTime(props.project.updated) : 'unknown',
)

const licenseIdDisplay = computed(() => {
  const id = props.project.license.id

  if (id === 'LicenseRef-All-Rights-Reserved') {
    return 'ARR'
  } else if (id.includes('LicenseRef')) {
    return id.replaceAll('LicenseRef-', '').replaceAll('-', ' ')
  } else {
    return id
  }
})

const messages = defineMessages({
  title: {
    id: 'project.about.details.title',
    defaultMessage: 'Details',
  },
  licensed: {
    id: 'project.about.details.licensed',
    defaultMessage: 'Licensed {license}',
  },
  created: {
    id: 'project.about.details.created',
    defaultMessage: 'Created {date}',
  },
  submitted: {
    id: 'project.about.details.submitted',
    defaultMessage: 'Submitted {date}',
  },
  published: {
    id: 'project.about.details.published',
    defaultMessage: 'Published {date}',
  },
  updated: {
    id: 'project.about.details.updated',
    defaultMessage: 'Updated {date}',
  },
})
</script>
