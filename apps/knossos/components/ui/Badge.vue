<template>
  <span :class="'badge ' + color + ' type--' + type">
    <template v-if="color"> <span class="circle" /> {{ $capitalizeString(type) }}</template>

    <!-- User roles -->
    <template v-else-if="type === 'admin'"> <ModrinthIcon /> Modrinth Team</template>
    <template v-else-if="type === 'moderator'"> <ModeratorIcon /> Moderator</template>
    <template v-else-if="type === 'creator'"><CreatorIcon /> Creator</template>

    <!-- Project statuses -->
    <template v-else-if="type === 'approved'"><ListIcon /> Listed</template>
    <template v-else-if="type === 'approved-general'"><CheckIcon /> Approved</template>
    <template v-else-if="type === 'unlisted'"><EyeOffIcon /> Unlisted</template>
    <template v-else-if="type === 'withheld'"><EyeOffIcon /> Withheld</template>
    <template v-else-if="type === 'private'"><LockIcon /> Private</template>
    <template v-else-if="type === 'scheduled'"> <CalendarIcon /> Scheduled</template>
    <template v-else-if="type === 'draft'"><DraftIcon /> Draft</template>
    <template v-else-if="type === 'archived'"> <ArchiveIcon /> Archived</template>
    <template v-else-if="type === 'rejected'"><CrossIcon /> Rejected</template>
    <template v-else-if="type === 'processing'"> <ProcessingIcon /> Under review</template>

    <!-- Team members -->
    <template v-else-if="type === 'accepted'"><CheckIcon /> Accepted</template>
    <template v-else-if="type === 'pending'"> <ProcessingIcon /> Pending </template>

    <!-- Transaction statuses -->
    <template v-else-if="type === 'success'"><CheckIcon /> Success</template>

    <!-- Report status -->
    <template v-else-if="type === 'closed'"> <CloseIcon /> Closed</template>

    <!-- Other -->
    <template v-else> <span class="circle" /> {{ $capitalizeString(type) }} </template>
  </span>
</template>

<script setup>
import ModrinthIcon from '~/assets/images/logo.svg?component'
import ModeratorIcon from '~/assets/images/sidebar/admin.svg?component'
import CreatorIcon from '~/assets/images/utils/box.svg?component'
import ListIcon from '~/assets/images/utils/list.svg?component'
import EyeOffIcon from '~/assets/images/utils/eye-off.svg?component'
import DraftIcon from '~/assets/images/utils/file-text.svg?component'
import CrossIcon from '~/assets/images/utils/x.svg?component'
import ArchiveIcon from '~/assets/images/utils/archive.svg?component'
import ProcessingIcon from '~/assets/images/utils/updated.svg?component'
import CheckIcon from '~/assets/images/utils/check.svg?component'
import LockIcon from '~/assets/images/utils/lock.svg?component'
import CalendarIcon from '~/assets/images/utils/calendar.svg?component'
import CloseIcon from '~/assets/images/utils/check-circle.svg?component'

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
.badge {
  font-weight: bold;
  width: fit-content;
  --badge-color: var(--color-gray);
  color: var(--badge-color);
  white-space: nowrap;

  .circle {
    width: 0.5rem;
    height: 0.5rem;
    border-radius: 50%;
    display: inline-block;
    margin-right: 0.25rem;
    background-color: var(--badge-color);
  }

  svg {
    vertical-align: -15%;
    width: 1em;
    height: 1em;
  }

  &.type--closed,
  &.type--withheld,
  &.type--rejected,
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
  &.type--success,
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
}
</style>
