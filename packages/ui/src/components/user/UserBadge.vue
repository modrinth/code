<template>
  <div class="w-full aspect-square">
    <Tooltip>
      <div class="rounded-xl transition-all" :class="{ 'hover:bg-button-bg': badgeData[badge].link }">
        <a v-if="badgeData[badge].link" :href="badgeData[badge].link" target="_blank">
          <component :is="badgeData[badge].icon" class="w-full h-full active:scale-[0.95] transition-transform cursor-pointer" />
        </a>
        <component :is="badgeData[badge].icon" v-else class="w-full h-full cursor-help" />
      </div>
      <template #popper>
        <div class="flex items-center gap-3 p-2 dark">
          <component :is="badgeData[badge].icon" class="h-14 w-14" />
          <div class="flex flex-col gap-1">
            <span>{{ formatMessage(badgeData[badge].title) }}</span>
            <p class="text-sm m-0 max-w-80 font-semibold" :class="badgeData[badge].description_color_class ?? 'text-secondary'">{{ formatMessage(badgeData[badge].description) }}</p>
          </div>
        </div>
      </template>
    </Tooltip>
  </div>
</template>
<script setup lang="ts">
import {
  StaffBadge,
  ModBadge,
  PlusBadge,
  TenMClubBadge,
  EarlyAdopterBadge,
  AlphaTesterBadge,
  BetaTesterBadge,
  TranslatorBadge,
  ContributorBadge
} from '@modrinth/assets'
import { useVIntl, defineMessage, type MessageDescriptor } from '@vintl/vintl'
import type { UserBadge } from '@modrinth/utils'
import { type Component, computed, type Ref } from 'vue'
import { Tooltip } from 'floating-vue'

const { formatMessage } = useVIntl()

defineProps<{
  badge: UserBadge
}>()

const badgeData: Ref<Record<UserBadge, {
  title: MessageDescriptor,
  description: MessageDescriptor,
  description_color_class?: string,
  icon: Component,
  link?: string,
}>> = computed(() => ({
  staff: {
    title: defineMessage({ id: 'user.badges.staff.title', defaultMessage: 'Modrinth Staff' }),
    description: defineMessage({ id: 'user.badges.staff.description', defaultMessage: 'This user is a member of the Modrinth Team' }),
    icon: StaffBadge,
  },
  mod: {
    title: defineMessage({ id: 'user.badges.mod.title', defaultMessage: 'Modrinth Content Moderator' }),
    description: defineMessage({ id: 'user.badges.mod.description', defaultMessage: `This user is a member of Modrinth's content moderation team`}),
    icon: ModBadge,
  },
  plus: {
    title: defineMessage({ id: 'user.badges.plus.title', defaultMessage: 'Modrinth+ Subscriber' }),
    description: defineMessage({ id: 'user.badges.plus.description', defaultMessage: 'Upgrade to Modrinth+ for ad-free browsing while still supporting creators and the platform' }),
    description_color_class: 'text-purple',
    icon: PlusBadge,
    link: 'https://modrinth.com/plus',
  },
  '10m-club': {
    title: defineMessage({ id: 'user.badges.10m-club.title', defaultMessage: '10,000,000 Downloads' }),
    description: defineMessage({ id: 'user.badges.10m-club.description', defaultMessage: 'This user is a creator with over 10 million downloads on their projects' }),
    icon: TenMClubBadge,
  },
  'early-adopter': {
    title: defineMessage({ id: 'user.badges.early-adopter.title', defaultMessage: 'Early Adopter' }),
    description: defineMessage({ id: 'user.badges.early-adopter.description', defaultMessage: 'This user participated in an Early Adopter program to help Modrinth launch a new product' }),
    icon: EarlyAdopterBadge,
  },
  'alpha-tester': {
    title: defineMessage({ id: 'user.badges.alpha-tester.title', defaultMessage: 'Since Alpha' }),
    description: defineMessage({ id: 'user.badges.alpha-tester.description', defaultMessage: 'This user has had a Modrinth account since Modrinth Alpha (Ended November 30th, 2020)' }),
    icon: AlphaTesterBadge,
  },
  'beta-tester': {
    title: defineMessage({ id: 'user.badges.beta-tester.title', defaultMessage: 'Since Beta' }),
    description: defineMessage({ id: 'user.badges.beta-tester.description', defaultMessage: 'This user has had a Modrinth account since Modrinth Beta (Ended February 27th, 2022)' }),
    icon: BetaTesterBadge,
  },
  contributor: {
    title: defineMessage({ id: 'user.badges.contributor.title', defaultMessage: 'Contributor' }),
    description: defineMessage({ id: 'user.badges.contributor.description', defaultMessage: `This user has meaningfully contributed to Modrinth's development` }),
    icon: ContributorBadge,
  },
  translator: {
    title: defineMessage({ id: 'user.badges.translator.title', defaultMessage: 'Translator' }),
    description: defineMessage({ id: 'user.badges.translator.description', defaultMessage: `This user has substantially contributed to Modrinth's translations into other languages` }),
    icon: TranslatorBadge,
  },
}));
</script>
