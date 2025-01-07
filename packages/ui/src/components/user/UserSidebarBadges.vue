<template>
  <div v-if="badges.length > 0" class="flex flex-col gap-3">
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div class="grid grid-cols-4 gap-2 place-items-center">
      <UserBadge v-for="badge in badges" :key="badge" :badge="badge" />
    </div>
  </div>
</template>
<script setup lang="ts">
import { useVIntl, defineMessages } from '@vintl/vintl'
import { isPermission, type User, type UserBadge as UserBadgeType } from '@modrinth/utils'
import { computed } from 'vue'
import UserBadge from './UserBadge.vue'

const { formatMessage } = useVIntl()

const props = defineProps<{
  user: User
  downloadCount: number
}>()

const joinDate = computed(() => new Date(props.user.created));
const MODRINTH_BETA_END_DATE = new Date("2022-02-27T08:00:00.000Z");
const MODRINTH_ALPHA_END_DATE = new Date("2020-11-30T08:00:00.000Z");

const badges = computed(() => {
  const badges: UserBadgeType[] = [];

  // Account type badges
  if (props.user.role === "admin") {
    badges.push("staff");
  }

  if (props.user.role === "moderator") {
    badges.push("mod");
  }

  // Account age badges
  if (isPermission(props.user.badges, 1 << 4) || joinDate.value < MODRINTH_ALPHA_END_DATE) {
    badges.push("alpha-tester");
  } else if (isPermission(props.user.badges, 1 << 4) || joinDate.value < MODRINTH_BETA_END_DATE) {
    badges.push("beta-tester");
  }

  // Early adopter badges
  if (
    isPermission(props.user.badges, 1 << 1) ||
    isPermission(props.user.badges, 1 << 2) ||
    isPermission(props.user.badges, 1 << 3)
  ) {
    badges.push("early-adopter");
  }

  // Contributor badges
  if (isPermission(props.user.badges, 1 << 5)) {
    badges.push("contributor");
  }

  if (isPermission(props.user.badges, 1 << 6)) {
    badges.push("translator");
  }

  // Earnable badges
  if (props.downloadCount && props.downloadCount > 10000000) {
    badges.push("10m-club");
  }

  // Purchaseable badges
  if (isPermission(props.user.badges, 1 << 0)) {
    badges.push("plus");
  }

  return badges;
});

const messages = defineMessages({
  title: {
    id: 'user.badges.title',
    defaultMessage: 'Badges',
  },
})
</script>
