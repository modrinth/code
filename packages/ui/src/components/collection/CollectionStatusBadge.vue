<template>
  <Badge :icon="metadata.icon" :formatted-name="metadata.formattedName" />
</template>

<script setup lang="ts">
import {
  LockIcon,
  GlobeIcon,
  LinkIcon,
  UnknownIcon,
} from '@modrinth/assets'
import { useVIntl, type MessageDescriptor } from '@vintl/vintl'
import type { Component } from 'vue'
import { computed } from 'vue'
import Badge from '../base/SimpleBadge.vue'
import type { CollectionStatus } from '@modrinth/utils'
import { commonMessages } from '../../utils/common-messages'

const props = defineProps<{
  status: CollectionStatus
}>()

const { formatMessage } = useVIntl()

const metadata = computed(() => ({
  icon: statusMetadata[props.status]?.icon ?? statusMetadata.unknown.icon,
  formattedName: formatMessage(statusMetadata[props.status]?.message ?? props.status),
}))

const statusMetadata: Record<CollectionStatus, { icon?: Component; message: MessageDescriptor }> = {
  listed: {
    icon: GlobeIcon,
    message: commonMessages.publicLabel
  },
  unlisted: {
    icon: LinkIcon,
    message: commonMessages.unlistedLabel
  },
  private: {
    icon: LockIcon,
    message: commonMessages.privateLabel
  },
  unknown: {
    icon: UnknownIcon,
    message: commonMessages.unknownLabel
  },
}
</script>
