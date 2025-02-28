<template>
  <span :class="'version-badges ' + color + ' type--' + type">
    <template v-if="color"> <span class="circle" /> {{ capitalizeString(type) }}</template>

    <!-- User roles -->
    <template v-else-if="type === 'admin'">
      <ModrinthIcon aria-hidden="true" /> {{ formatMessage(messages.modrinthTeamLabel) }}
    </template>
    <template v-else-if="type === 'moderator'">
      <ScaleIcon aria-hidden="true" /> {{ formatMessage(messages.moderatorLabel) }}
    </template>
    <template v-else-if="type === 'creator'">
      <BoxIcon aria-hidden="true" /> {{ formatMessage(messages.creatorLabel) }}
    </template>

    <!-- Project statuses -->
    <template v-else-if="type === 'approved'">
      <ListIcon aria-hidden="true" /> {{ formatMessage(messages.listedLabel) }}
    </template>
    <template v-else-if="type === 'approved-general'">
      <CheckIcon aria-hidden="true" /> {{ formatMessage(messages.approvedLabel) }}
    </template>
    <template v-else-if="type === 'unlisted'">
      <EyeOffIcon aria-hidden="true" /> {{ formatMessage(messages.unlistedLabel) }}
    </template>
    <template v-else-if="type === 'withheld'">
      <EyeOffIcon aria-hidden="true" /> {{ formatMessage(messages.withheldLabel) }}
    </template>
    <template v-else-if="type === 'private'">
      <LockIcon aria-hidden="true" /> {{ formatMessage(messages.privateLabel) }}
    </template>
    <template v-else-if="type === 'scheduled'">
      <CalendarIcon aria-hidden="true" /> {{ formatMessage(messages.scheduledLabel) }}
    </template>
    <template v-else-if="type === 'draft'">
      <FileTextIcon aria-hidden="true" /> {{ formatMessage(messages.draftLabel) }}
    </template>
    <template v-else-if="type === 'archived'">
      <ArchiveIcon aria-hidden="true" /> {{ formatMessage(messages.archivedLabel) }}
    </template>
    <template v-else-if="type === 'rejected'">
      <XIcon aria-hidden="true" /> {{ formatMessage(messages.rejectedLabel) }}
    </template>
    <template v-else-if="type === 'processing'">
      <UpdatedIcon aria-hidden="true" /> {{ formatMessage(messages.underReviewLabel) }}
    </template>

    <!-- Team members -->
    <template v-else-if="type === 'accepted'">
      <CheckIcon aria-hidden="true" /> {{ formatMessage(messages.acceptedLabel) }}
    </template>
    <template v-else-if="type === 'pending'">
      <UpdatedIcon aria-hidden="true" /> {{ formatMessage(messages.pendingLabel) }}
    </template>

    <!-- Transaction statuses (pending, processing reused) -->
    <template v-else-if="type === 'processed'">
      <CheckIcon aria-hidden="true" /> {{ formatMessage(messages.processedLabel) }}
    </template>
    <template v-else-if="type === 'failed'">
      <XIcon aria-hidden="true" /> {{ formatMessage(messages.failedLabel) }}
    </template>
    <template v-else-if="type === 'returned'">
      <XIcon aria-hidden="true" /> {{ formatMessage(messages.returnedLabel) }}
    </template>

    <!-- Report status -->
    <template v-else-if="type === 'closed'">
      <XIcon aria-hidden="true" /> {{ formatMessage(messages.closedLabel) }}
    </template>

    <!-- Other -->
    <template v-else> <span class="circle" /> {{ capitalizeString(type) }} </template>
  </span>
</template>

<script setup lang="ts">
import {
  ModrinthIcon,
  ScaleIcon,
  BoxIcon,
  ListIcon,
  EyeOffIcon,
  FileTextIcon,
  XIcon,
  ArchiveIcon,
  UpdatedIcon,
  CheckIcon,
  LockIcon,
  CalendarIcon,
} from '@modrinth/assets'
import { capitalizeString } from '@modrinth/utils'
import { useVIntl, defineMessages } from '@vintl/vintl'

const messages = defineMessages({
  acceptedLabel: {
    id: 'omorphia.component.badges.label.accepted',
    defaultMessage: 'Accepted',
  },
  approvedLabel: {
    id: 'omorphia.component.badges.label.approved',
    defaultMessage: 'Approved',
  },
  archivedLabel: {
    id: 'omorphia.component.badges.label.archived',
    defaultMessage: 'Archived',
  },
  closedLabel: {
    id: 'omorphia.component.badges.label.closed',
    defaultMessage: 'Closed',
  },
  creatorLabel: {
    id: 'omorphia.component.badges.label.creator',
    defaultMessage: 'Creator',
  },
  draftLabel: {
    id: 'omorphia.component.badges.label.draft',
    defaultMessage: 'Draft',
  },
  failedLabel: {
    id: 'omorphia.component.badges.label.failed',
    defaultMessage: 'Failed',
  },
  listedLabel: {
    id: 'omorphia.component.badges.label.listed',
    defaultMessage: 'Listed',
  },
  moderatorLabel: {
    id: 'omorphia.component.badges.label.moderator',
    defaultMessage: 'Moderator',
  },
  modrinthTeamLabel: {
    id: 'omorphia.component.badges.label.modrinth-team',
    defaultMessage: 'Modrinth Team',
  },
  pendingLabel: {
    id: 'omorphia.component.badges.label.pending',
    defaultMessage: 'Pending',
  },
  privateLabel: {
    id: 'omorphia.component.badges.label.private',
    defaultMessage: 'Private',
  },
  processedLabel: {
    id: 'omorphia.component.badges.label.processed',
    defaultMessage: 'Processed',
  },
  rejectedLabel: {
    id: 'omorphia.component.badges.label.rejected',
    defaultMessage: 'Rejected',
  },
  returnedLabel: {
    id: 'omorphia.component.badges.label.returned',
    defaultMessage: 'Returned',
  },
  scheduledLabel: {
    id: 'omorphia.component.badges.label.scheduled',
    defaultMessage: 'Scheduled',
  },
  underReviewLabel: {
    id: 'omorphia.component.badges.label.under-review',
    defaultMessage: 'Under review',
  },
  unlistedLabel: {
    id: 'omorphia.component.badges.label.unlisted',
    defaultMessage: 'Unlisted',
  },
  withheldLabel: {
    id: 'omorphia.component.badges.label.withheld',
    defaultMessage: 'Withheld',
  },
})
const { formatMessage } = useVIntl()

defineProps<{
  type: string
  color?: string
}>()
</script>
<style lang="scss" scoped>
.version-badge {
  display: flex;
  align-items: center;
  font-weight: bold;
  width: fit-content;
  --badge-color: var(--color-gray);
  color: var(--badge-color);

  .circle {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    display: inline-block;
    margin-right: 0.25rem;
    background-color: var(--badge-color);
  }

  svg {
    margin-right: 0.25rem;
  }

  &.type--closed,
  &.type--withheld,
  &.type--rejected,
  &.type--returned,
  &.type--failed,
  &.red {
    --badge-color: var(--color-red);
  }

  &.type--pending,
  &.type--moderator,
  &.type--processing,
  &.type--scheduled,
  &.orange {
    --badge-color: var(--color-orange);
  }

  &.type--accepted,
  &.type--admin,
  &.type--processed,
  &.type--approved-general,
  &.green {
    --badge-color: var(--color-green);
  }

  &.type--creator,
  &.type--approved,
  &.blue {
    --badge-color: var(--color-blue);
  }

  &.type--unlisted,
  &.purple {
    --badge-color: var(--color-purple);
  }

  &.type--private,
  &.gray {
    --badge-color: var(--color-gray);
  }

  &::first-letter {
    text-transform: capitalize;
  }
}
</style>
