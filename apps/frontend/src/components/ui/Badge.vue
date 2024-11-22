<template>
  <span
    :class="
      'badge flex items-center gap-1 font-semibold text-secondary ' + color + ' type--' + type
    "
  >
    <template v-if="color"> <span class="circle" /> {{ $capitalizeString(type) }}</template>

    <!-- User roles -->
    <template v-else-if="type === 'admin'"> <ModrinthIcon /> BBSMC 团队</template>
    <template v-else-if="type === 'moderator'"> <ModeratorIcon /> 版主</template>
    <template v-else-if="type === 'creator'"><CreatorIcon /> 创建者</template>
    <template v-else-if="type === 'plus'"><PlusIcon /> BBSMC 会员</template>

    <!-- Project statuses -->
    <template v-else-if="type === 'approved'"><GlobeIcon /> 公开</template>
    <template v-else-if="type === 'approved-general'"><CheckIcon /> 已批准</template>
    <template v-else-if="type === 'unlisted' || type === 'withheld'"
      ><LinkIcon /> 未公开</template
    >
    <template v-else-if="type === 'private'"><LockIcon /> 私有</template>
    <template v-else-if="type === 'scheduled'"> <CalendarIcon /> 已安排</template>
    <template v-else-if="type === 'draft'"><DraftIcon /> 草稿</template>
    <template v-else-if="type === 'archived'"> <ArchiveIcon /> 已归档</template>
    <template v-else-if="type === 'rejected'"><CrossIcon /> 已拒绝</template>
    <template v-else-if="type === 'processing'"> <ProcessingIcon /> 审核中</template>

    <!-- Team members -->
    <template v-else-if="type === 'accepted'"><CheckIcon /> 已接收</template>
    <template v-else-if="type === 'pending'"> <ProcessingIcon /> 待处理 </template>

    <!-- Transaction statuses -->
    <template v-else-if="type === 'success'"><CheckIcon /> 完成</template>

    <!-- Report status -->
    <template v-else-if="type === 'closed'"> <CloseIcon /> 关闭</template>

    <!-- Other -->
    <template v-else> <span class="circle" /> {{ $capitalizeString(type) }} </template>
  </span>
</template>

<script setup>
import { GlobeIcon, LinkIcon } from "@modrinth/assets";

import ModrinthIcon from "~/assets/images/logo.svg?component";
import PlusIcon from "~/assets/images/utils/plus.svg?component";
import ModeratorIcon from "~/assets/images/sidebar/admin.svg?component";
import CreatorIcon from "~/assets/images/utils/box.svg?component";
import DraftIcon from "~/assets/images/utils/file-text.svg?component";
import CrossIcon from "~/assets/images/utils/x.svg?component";
import ArchiveIcon from "~/assets/images/utils/archive.svg?component";
import ProcessingIcon from "~/assets/images/utils/updated.svg?component";
import CheckIcon from "~/assets/images/utils/check.svg?component";
import LockIcon from "~/assets/images/utils/lock.svg?component";
import CalendarIcon from "~/assets/images/utils/calendar.svg?component";
import CloseIcon from "~/assets/images/utils/check-circle.svg?component";

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
