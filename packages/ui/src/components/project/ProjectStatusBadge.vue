<template>
  <Badge :icon="metadata.icon" :formatted-name="metadata.formattedName" />
</template>

<script setup lang="ts">
import {
  FileTextIcon,
  ArchiveIcon,
  UpdatedIcon,
  LockIcon,
  CalendarIcon,
  GlobeIcon,
  LinkIcon,
  UnknownIcon,
  XIcon,
} from '@modrinth/assets'
import { useVIntl, defineMessage, type MessageDescriptor } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'
import Badge from '../base/SimpleBadge.vue'
import type { ProjectStatus } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'

const props = defineProps<{
  status: ProjectStatus
}>()

const { formatMessage } = useVIntl()

const metadata = computed(() => ({
  icon: statusMetadata[props.status]?.icon ?? statusMetadata.unknown.icon,
  formattedName: formatMessage(statusMetadata[props.status]?.message ?? props.status),
}))

const statusMetadata: Record<ProjectStatus, { icon?: Component; message: MessageDescriptor }> = {
  approved: {
    icon: GlobeIcon,
    message: commonMessages.publicLabel,
  },
  unlisted: {
    icon: LinkIcon,
    message: commonMessages.unlistedLabel,
  },
  withheld: {
    icon: LinkIcon,
    message: defineMessage({
      id: 'project.visibility.unlisted-by-staff',
      defaultMessage: 'Unlisted by staff',
    }),
  },
  private: {
    icon: LockIcon,
    message: commonMessages.privateLabel,
  },
  scheduled: {
    icon: CalendarIcon,
    message: defineMessage({
      id: 'project.visibility.scheduled',
      defaultMessage: 'Scheduled',
    }),
  },
  draft: {
    icon: FileTextIcon,
    message: defineMessage({
      id: 'project.visibility.draft',
      defaultMessage: 'Draft',
    }),
  },
  archived: {
    icon: ArchiveIcon,
    message: defineMessage({
      id: 'project.visibility.archived',
      defaultMessage: 'Archived',
    }),
  },
  rejected: {
    icon: XIcon,
    message: commonMessages.rejectedLabel,
  },
  processing: {
    icon: UpdatedIcon,
    message: defineMessage({
      id: 'project.visibility.under-review',
      defaultMessage: 'Under review',
    }),
  },
  unknown: {
    icon: UnknownIcon,
    message: commonMessages.unknownLabel,
  },
}
</script>
