<template>
  <span :class="'version-badge ' + color + ' type--' + type">
    <template v-if="color"> <span class="circle" /> {{ capitalizeString(type) }}</template>

    <!-- User roles -->
    <template v-else-if="type === 'admin'"> <ModrinthIcon /> Modrinth Team</template>
    <template v-else-if="type === 'moderator'"> <ScaleIcon /> Moderator</template>
    <template v-else-if="type === 'creator'"><BoxIcon /> Creator</template>

    <!-- Project statuses -->
    <template v-else-if="type === 'approved'"><ListIcon /> Listed</template>
    <template v-else-if="type === 'approved-general'"><CheckIcon /> Approved</template>
    <template v-else-if="type === 'unlisted'"><EyeOffIcon /> Unlisted</template>
    <template v-else-if="type === 'withheld'"><EyeOffIcon /> Withheld</template>
    <template v-else-if="type === 'private'"><LockIcon /> Private</template>
    <template v-else-if="type === 'scheduled'"> <CalendarIcon /> Scheduled</template>
    <template v-else-if="type === 'draft'"><FileTextIcon /> Draft</template>
    <template v-else-if="type === 'archived'"> <ArchiveIcon /> Archived</template>
    <template v-else-if="type === 'rejected'"><XIcon /> Rejected</template>
    <template v-else-if="type === 'processing'"> <UpdatedIcon /> Under review</template>

    <!-- Team members -->
    <template v-else-if="type === 'accepted'"><CheckIcon /> Accepted</template>
    <template v-else-if="type === 'pending'"> <UpdatedIcon /> Pending</template>

    <!-- Transaction statuses (pending, processing reused) -->
    <template v-else-if="type === 'processed'"><CheckIcon /> Processed</template>
    <template v-else-if="type === 'failed'"><XIcon /> Failed</template>
    <template v-else-if="type === 'returned'"><XIcon /> Returned</template>

    <!-- Report status -->
    <template v-else-if="type === 'closed'"> <XIcon /> Closed</template>

    <!-- Other -->
    <template v-else> <span class="circle" /> {{ capitalizeString(type) }} </template>
  </span>
</template>

<script setup>
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
  capitalizeString,
} from '@'

defineProps({
  type: {
    type: String,
    required: true,
  },
  color: {
    type: String,
    default: '',
  },
})
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
