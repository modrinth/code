<template>
  <span
    :class="
      'badge flex items-center gap-1 font-semibold text-secondary ' + color + ' type--' + type
    "
  >
    <template v-if="color"> <span class="circle" /> {{ capitalizeString(type) }}</template>

    <!-- User roles -->
    <template v-else-if="type === 'admin'"> <ModrinthIcon /> Modrinth Team</template>
    <template v-else-if="type === 'moderator'"> <ModeratorIcon /> Moderator</template>
    <template v-else-if="type === 'creator'"><CreatorIcon /> Creator</template>
    <template v-else-if="type === 'plus'"><PlusIcon /> Modrinth Plus</template>

    <!-- Project statuses -->
    <template v-else-if="type === 'approved'"><GlobeIcon /> Public</template>
    <template v-else-if="type === 'approved-general'"><CheckIcon /> Approved</template>
    <template v-else-if="type === 'unlisted' || type === 'withheld'"
      ><LinkIcon /> Unlisted</template
    >
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
    <template v-else> <span class="circle" /> {{ capitalizeString(type) }} </template>
  </span>
</template>

<script setup>
import {
  GlobeIcon,
  LinkIcon,
  ModrinthIcon,
  PlusIcon,
  ScaleIcon as ModeratorIcon,
  BoxIcon as CreatorIcon,
  FileTextIcon as DraftIcon,
  XIcon as CrossIcon,
  ArchiveIcon,
  UpdatedIcon as ProcessingIcon,
  CheckIcon,
  LockIcon,
  CalendarIcon,
  XCircleIcon as CloseIcon,
} from "@modrinth/assets";
import { capitalizeString } from "@modrinth/utils";

defineProps({
  type: {
    type: String,
    required: true,
  },
  color: {
    type: String,
    default: "",
  },
});
</script>

<style lang="scss" scoped>
.badge {
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
  &.blue {
    --badge-color: var(--color-blue);
  }

  &.type--unlisted,
  &.type--plus,
  &.purple {
    --badge-color: var(--color-purple);
  }

  &.type--private,
  &.type--approved,
  &.gray {
    --badge-color: var(--color-secondary);
  }
}
</style>
