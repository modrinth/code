<template>
  <div
    v-if="
      project.issues_url ||
      project.source_url ||
      project.wiki_url ||
      project.discord_url ||
      project.donation_urls.length > 0
    "
    class="flex flex-col gap-3"
  >
    <h2 class="text-lg m-0">{{ formatMessage(messages.title) }}</h2>
    <div
      class="flex flex-col gap-3 font-semibold"
    >
      <ProjectSidebarLinksLink
        :url="project.issues_url"
        :link-target="linkTarget"
      >
        <IssuesIcon aria-hidden="true" />
        {{ formatMessage(messages.issues) }}
      </ProjectSidebarLinksLink>
      <ProjectSidebarLinksLink
        :url="project.source_url"
        :link-target="linkTarget"
      >
        <CodeIcon aria-hidden="true" />
        {{ formatMessage(messages.source) }}
      </ProjectSidebarLinksLink>
      <ProjectSidebarLinksLink
        :url="project.wiki_url"
        :link-target="linkTarget"
      >
        <WikiIcon aria-hidden="true" />
        {{ formatMessage(messages.wiki) }}
      </ProjectSidebarLinksLink>
      <ProjectSidebarLinksLink
        :url="project.discord_url"
        :link-target="linkTarget"
      >
        <DiscordIcon aria-hidden="true" />
        {{ formatMessage(messages.discord) }}
      </ProjectSidebarLinksLink>
      <hr
        v-if="
          (project.issues_url || project.source_url || project.wiki_url || project.discord_url) &&
          project.donation_urls.length > 0
        "
        class="w-full border-button-border my-0.5"
      />
      <a
        v-for="(donation, index) in project.donation_urls"
        :key="index"
        :href="donation.url"
        :target="linkTarget"
        rel="noopener nofollow ugc"
      >
        <BuyMeACoffeeIcon v-if="donation.id === 'bmac'" aria-hidden="true" />
        <PatreonIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
        <KoFiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
        <PayPalIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
        <OpenCollectiveIcon v-else-if="donation.id === 'open-collective'" aria-hidden="true" />
        <HeartIcon v-else-if="donation.id === 'github'" />
        <CurrencyIcon v-else />
        <span v-if="donation.id === 'bmac'">{{ formatMessage(messages.donateBmac) }}</span>
        <span v-else-if="donation.id === 'patreon'">{{
          formatMessage(messages.donatePatreon)
        }}</span>
        <span v-else-if="donation.id === 'paypal'">{{ formatMessage(messages.donatePayPal) }}</span>
        <span v-else-if="donation.id === 'ko-fi'">{{ formatMessage(messages.donateKoFi) }}</span>
        <span v-else-if="donation.id === 'github'">{{ formatMessage(messages.donateGithub) }}</span>
        <span v-else>{{ formatMessage(messages.donateGeneric) }}</span>
        <ExternalIcon aria-hidden="true" class="external-icon" />
      </a>
    </div>
  </div>
</template>
<script setup lang="ts">
import {
  BuyMeACoffeeIcon,
  CodeIcon,
  CurrencyIcon,
  DiscordIcon,
  ExternalIcon,
  HeartIcon,
  IssuesIcon,
  KoFiIcon,
  OpenCollectiveIcon,
  PatreonIcon,
  PayPalIcon,
  WikiIcon,
} from '@modrinth/assets'
import { useVIntl, defineMessages } from '@vintl/vintl'
import ProjectSidebarLinksLink from './ProjectSidebarLinksLink.vue'

const { formatMessage } = useVIntl()

defineProps<{
  project: {
    issues_url: string
    source_url: string
    wiki_url: string
    discord_url: string
    donation_urls: {
      id: string
      url: string
    }[]
  }
  linkTarget: string
}>()

const messages = defineMessages({
  title: {
    id: 'project.about.links.title',
    defaultMessage: 'Links',
  },
  issues: {
    id: 'project.about.links.issues',
    defaultMessage: 'Report issues',
  },
  source: {
    id: 'project.about.links.source',
    defaultMessage: 'View source',
  },
  wiki: {
    id: 'project.about.links.wiki',
    defaultMessage: 'Visit wiki',
  },
  discord: {
    id: 'project.about.links.discord',
    defaultMessage: 'Join Discord server',
  },
  donateGeneric: {
    id: 'project.about.links.donate.generic',
    defaultMessage: 'Donate',
  },
  donateGitHub: {
    id: 'project.about.links.donate.github',
    defaultMessage: 'Sponsor on GitHub',
  },
  donateBmac: {
    id: 'project.about.links.donate.bmac',
    defaultMessage: 'Buy Me a Coffee',
  },
  donatePatreon: {
    id: 'project.about.links.donate.patreon',
    defaultMessage: 'Donate on Patreon',
  },
  donatePayPal: {
    id: 'project.about.links.donate.paypal',
    defaultMessage: 'Donate on PayPal',
  },
  donateKoFi: {
    id: 'project.about.links.donate.kofi',
    defaultMessage: 'Donate on Ko-fi',
  },
  donateGithub: {
    id: 'project.about.links.donate.github',
    defaultMessage: 'Sponsor on GitHub',
  },
})
</script>
