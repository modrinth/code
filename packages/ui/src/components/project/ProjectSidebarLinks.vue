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
      class="flex flex-col gap-3 font-semibold [&>a]:flex [&>a]:gap-2 [&>a]:items-center [&>a]:w-fit [&>a]:text-primary [&>a]:leading-[1.2] [&>a:hover]:underline"
    >
      <a
        v-if="project.issues_url"
        :href="project.issues_url"
        :target="linkTarget"
        rel="noopener nofollow ugc"
      >
        <IssuesIcon aria-hidden="true" />
        {{ formatMessage(messages.issues) }}
        <ExternalIcon aria-hidden="true" class="external-icon" />
      </a>
      <a
        v-if="project.source_url"
        :href="project.source_url"
        :target="linkTarget"
        rel="noopener nofollow ugc"
      >
        <CodeIcon aria-hidden="true" />
        {{ formatMessage(messages.source) }}
        <ExternalIcon aria-hidden="true" class="external-icon" />
      </a>
      <a
        v-if="project.wiki_url"
        :href="project.wiki_url"
        :target="linkTarget"
        rel="noopener nofollow ugc"
      >
        <WikiIcon aria-hidden="true" />
        {{ formatMessage(messages.wiki) }}
        <ExternalIcon aria-hidden="true" class="external-icon" />
      </a>
      <a
        v-if="project.discord_url"
        :href="project.discord_url"
        :target="linkTarget"
        rel="noopener nofollow ugc"
      >
        <DiscordIcon class="shrink" aria-hidden="true" />
        {{ formatMessage(messages.discord) }}
        <ExternalIcon aria-hidden="true" class="external-icon" />
      </a>
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
