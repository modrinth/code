<template>
  <NewModal ref="modalLicense" :header="project.license.name ? project.license.name : 'License'">
    <template #title>
      <Avatar :src="project.icon_url" :alt="project.title" class="icon" size="32px" no-shadow />
      <span class="text-lg font-extrabold text-contrast">
        {{ project.license.name ? project.license.name : "License" }}
      </span>
    </template>
    <div
      class="markdown-body"
      v-html="
        renderString(licenseText).isEmpty ? 'Loading license text...' : renderString(licenseText)
      "
    />
  </NewModal>
  <section class="normal-page__content">
    <div
      v-if="project.body"
      class="markdown-body card"
      v-html="renderHighlightedString(project.body || '')"
    />
  </section>
  <div class="normal-page__sidebar">
    <AdPlaceholder />
    <div v-if="versions.length > 0" class="card flex-card experimental-styles-within">
      <h2>{{ formatMessage(compatibilityMessages.title) }}</h2>
      <section>
        <h3>{{ formatMessage(compatibilityMessages.minecraftJava) }}</h3>
        <div class="tag-list">
          <div
            v-for="version in getVersionsToDisplay(project)"
            :key="`version-tag-${version}`"
            class="tag-list__item"
          >
            {{ version }}
          </div>
        </div>
      </section>
      <section v-if="project.project_type !== 'resourcepack'">
        <h3>{{ formatMessage(compatibilityMessages.platforms) }}</h3>
        <div class="tag-list">
          <div
            v-for="platform in project.loaders"
            :key="`platform-tag-${platform}`"
            :class="`tag-list__item`"
            :style="`--_color: var(--color-platform-${platform})`"
          >
            <svg v-html="tags.loaders.find((x) => x.name === platform).icon"></svg>
            {{ formatCategory(platform) }}
          </div>
        </div>
      </section>
      <section
        v-if="
          (project.actualProjectType === 'mod' || project.project_type === 'modpack') &&
          !(project.client_side === 'unsupported' && project.server_side === 'unsupported') &&
          !(project.client_side === 'unknown' && project.server_side === 'unknown')
        "
      >
        <h3>{{ formatMessage(compatibilityMessages.environments) }}</h3>
        <div class="status-list">
          <div
            v-if="
              (project.client_side === 'required' && project.server_side !== 'required') ||
              (project.client_side === 'optional' && project.server_side === 'optional')
            "
            class="status-list__item"
          >
            <ClientIcon aria-hidden="true" />
            Client only
          </div>
          <div
            v-if="
              (project.server_side === 'required' && project.client_side !== 'unsupported') ||
              (project.client_side === 'optional' && project.server_side === 'optional')
            "
            class="status-list__item"
          >
            <ServerIcon aria-hidden="true" />
            Server only
          </div>
          <div class="status-list__item">
            <UserIcon aria-hidden="true" />
            Singleplayer
          </div>
          <div
            v-if="
              project.client_side === 'optional' ||
              (project.client_side === 'required' && project.server_side === 'optional') ||
              project.server_side === 'optional' ||
              (project.server_side === 'required' && project.client_side === 'optional')
            "
            class="status-list__item"
          >
            <MonitorSmartphoneIcon aria-hidden="true" />
            Client and server <span class="text-sm">(optional)</span>
          </div>
        </div>
      </section>
    </div>
    <div
      v-if="
        project.issues_url ||
        project.source_url ||
        project.wiki_url ||
        project.discord_url ||
        project.donation_urls.length > 0
      "
      class="card flex-card experimental-styles-within"
    >
      <h2>{{ formatMessage(linksMessages.title) }}</h2>
      <div class="links-list">
        <a
          v-if="project.issues_url"
          :href="project.issues_url"
          :target="$external()"
          rel="noopener nofollow ugc"
        >
          <IssuesIcon aria-hidden="true" />
          {{ formatMessage(linksMessages.issues) }}
          <ExternalIcon aria-hidden="true" class="external-icon" />
        </a>
        <a
          v-if="project.source_url"
          :href="project.source_url"
          :target="$external()"
          rel="noopener nofollow ugc"
        >
          <CodeIcon aria-hidden="true" />
          {{ formatMessage(linksMessages.source) }}
          <ExternalIcon aria-hidden="true" class="external-icon" />
        </a>
        <a
          v-if="project.wiki_url"
          :href="project.wiki_url"
          :target="$external()"
          rel="noopener nofollow ugc"
        >
          <WikiIcon aria-hidden="true" />
          {{ formatMessage(linksMessages.wiki) }}
          <ExternalIcon aria-hidden="true" class="external-icon" />
        </a>
        <a
          v-if="project.discord_url"
          :href="project.discord_url"
          :target="$external()"
          rel="noopener nofollow ugc"
        >
          <DiscordIcon class="shrink" aria-hidden="true" />
          {{ formatMessage(linksMessages.discord) }}
          <ExternalIcon aria-hidden="true" class="external-icon" />
        </a>
        <hr
          v-if="
            (project.issues_url || project.source_url || project.wiki_url || project.discord_url) &&
            project.donation_urls.length > 0
          "
        />
        <a
          v-for="(donation, index) in project.donation_urls"
          :key="index"
          :href="donation.url"
          :target="$external()"
          rel="noopener nofollow ugc"
        >
          <BuyMeACoffeeIcon v-if="donation.id === 'bmac'" aria-hidden="true" />
          <PatreonIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
          <KoFiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
          <PayPalIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
          <OpenCollectiveIcon v-else-if="donation.id === 'open-collective'" aria-hidden="true" />
          <HeartIcon v-else-if="donation.id === 'github'" />
          <CurrencyIcon v-else />
          <span v-if="donation.id === 'bmac'">{{ formatMessage(linksMessages.donateBmac) }}</span>
          <span v-else-if="donation.id === 'patreon'">{{
            formatMessage(linksMessages.donatePatreon)
          }}</span>
          <span v-else-if="donation.id === 'paypal'">{{
            formatMessage(linksMessages.donatePayPal)
          }}</span>
          <span v-else-if="donation.id === 'ko-fi'">{{
            formatMessage(linksMessages.donateKoFi)
          }}</span>
          <span v-else-if="donation.id === 'github'">{{
            formatMessage(linksMessages.donateGithub)
          }}</span>
          <span v-else>{{ formatMessage(linksMessages.donateGeneric) }}</span>
          <ExternalIcon aria-hidden="true" class="external-icon" />
        </a>
      </div>
    </div>
    <div class="card flex-card experimental-styles-within">
      <h2>{{ formatMessage(creatorsMessages.title) }}</h2>
      <div class="details-list">
        <template v-if="organization">
          <nuxt-link
            class="details-list__item details-list__item--type-large"
            :to="`/organization/${organization.slug}`"
          >
            <Avatar :src="organization.icon_url" :alt="organization.name" size="32px" />
            <div class="rows">
              <span>
                {{ organization.name }}
              </span>
              <span class="details-list__item__text--style-secondary">Organization</span>
            </div>
          </nuxt-link>
          <hr v-if="members.length > 0" />
        </template>
        <nuxt-link
          v-for="member in members"
          :key="`member-${member.id}`"
          class="details-list__item details-list__item--type-large"
          :to="'/user/' + member.user.username"
        >
          <Avatar :src="member.avatar_url" :alt="member.name" size="32px" circle />
          <div class="rows">
            <span class="flex items-center gap-1">
              {{ member.name }}
              <CrownIcon
                v-if="member.is_owner"
                v-tooltip="formatMessage(creatorsMessages.owner)"
                class="text-brand-orange"
              />
            </span>
            <span class="details-list__item__text--style-secondary">{{ member.role }}</span>
          </div>
        </nuxt-link>
      </div>
    </div>
    <div class="card flex-card experimental-styles-within">
      <h2>{{ formatMessage(detailsMessages.title) }}</h2>
      <div class="details-list">
        <div class="details-list__item">
          <BookTextIcon aria-hidden="true" />
          <div>
            Licensed
            <a
              v-if="project.license.url"
              class="text-link hover:underline"
              :href="project.license.url"
              :target="$external()"
              rel="noopener nofollow ugc"
            >
              {{ licenseIdDisplay }}
              <ExternalIcon aria-hidden="true" class="external-icon ml-1 mt-[-1px] inline" />
            </a>
            <span
              v-else-if="
                project.license.id === 'LicenseRef-All-Rights-Reserved' ||
                !project.license.id.includes('LicenseRef')
              "
              class="text-link hover:underline"
              @click="(event) => getLicenseData(event)"
            >
              {{ licenseIdDisplay }}
            </span>
            <span v-else>{{ licenseIdDisplay }}</span>
          </div>
        </div>
        <div
          v-if="project.approved"
          v-tooltip="$dayjs(project.approved).format('MMMM D, YYYY [at] h:mm A')"
          class="details-list__item"
        >
          <CalendarIcon aria-hidden="true" />
          <div>
            {{ formatMessage(detailsMessages.published, { date: publishedDate }) }}
          </div>
        </div>
        <div
          v-else
          v-tooltip="$dayjs(project.published).format('MMMM D, YYYY [at] h:mm A')"
          class="details-list__item"
        >
          <CalendarIcon aria-hidden="true" />
          <div>
            {{ formatMessage(detailsMessages.created, { date: createdDate }) }}
          </div>
        </div>
        <div
          v-if="project.status === 'processing' && project.queued"
          v-tooltip="$dayjs(project.queued).format('MMMM D, YYYY [at] h:mm A')"
          class="details-list__item"
        >
          <ScaleIcon aria-hidden="true" />
          <div>
            {{ formatMessage(detailsMessages.submitted, { date: submittedDate }) }}
          </div>
        </div>
        <div
          v-if="versions.length > 0 && project.updated"
          v-tooltip="$dayjs(project.updated).format('MMMM D, YYYY [at] h:mm A')"
          class="details-list__item"
        >
          <VersionIcon aria-hidden="true" />
          <div>
            {{ formatMessage(detailsMessages.updated, { date: updatedDate }) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  CalendarIcon,
  IssuesIcon,
  WikiIcon,
  OpenCollectiveIcon,
  DiscordIcon,
  ScaleIcon,
  KoFiIcon,
  BookTextIcon,
  PayPalIcon,
  CrownIcon,
  BuyMeACoffeeIcon,
  CurrencyIcon,
  PatreonIcon,
  HeartIcon,
  VersionIcon,
  ExternalIcon,
  CodeIcon,
  UserIcon,
  ServerIcon,
  ClientIcon,
  MonitorSmartphoneIcon,
} from "@modrinth/assets";

import { NewModal, Avatar } from "@modrinth/ui";

import { formatCategory, renderString } from "@modrinth/utils";
import { renderHighlightedString } from "~/helpers/highlight.js";
import { getVersionsToDisplay } from "~/helpers/projects.js";
import AdPlaceholder from "~/components/ui/AdPlaceholder.vue";

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
  versions: {
    type: Array,
    default() {
      return {};
    },
  },
  members: {
    type: Array,
    default() {
      return {};
    },
  },
  organization: {
    type: Object,
    default() {
      return {};
    },
  },
});

const tags = useTags();
const { formatMessage } = useVIntl();
const formatRelativeTime = useRelativeTime();

const compatibilityMessages = defineMessages({
  title: {
    id: "project.about.compatibility.title",
    defaultMessage: "Compatibility",
  },
  minecraftJava: {
    id: "project.about.compatibility.game.minecraftJava",
    defaultMessage: "Minecraft: Java Edition",
  },
  platforms: {
    id: "project.about.compatibility.platforms",
    defaultMessage: "Platforms",
  },
  environments: {
    id: "project.about.compatibility.environments",
    defaultMessage: "Supported environments",
  },
});
const linksMessages = defineMessages({
  title: {
    id: "project.about.links.title",
    defaultMessage: "Links",
  },
  issues: {
    id: "project.about.links.issues",
    defaultMessage: "Report issues",
  },
  source: {
    id: "project.about.links.source",
    defaultMessage: "View source",
  },
  wiki: {
    id: "project.about.links.wiki",
    defaultMessage: "Visit wiki",
  },
  discord: {
    id: "project.about.links.discord",
    defaultMessage: "Join Discord server",
  },
  donateGeneric: {
    id: "project.about.links.donate.generic",
    defaultMessage: "Donate",
  },
  donateGitHub: {
    id: "project.about.links.donate.github",
    defaultMessage: "Sponsor on GitHub",
  },
  donateBmac: {
    id: "project.about.links.donate.bmac",
    defaultMessage: "Buy Me a Coffee",
  },
  donatePatreon: {
    id: "project.about.links.donate.patreon",
    defaultMessage: "Donate on Patreon",
  },
  donatePayPal: {
    id: "project.about.links.donate.paypal",
    defaultMessage: "Donate on PayPal",
  },
  donateKoFi: {
    id: "project.about.links.donate.kofi",
    defaultMessage: "Donate on Ko-fi",
  },
  donateGithub: {
    id: "project.about.links.donate.github",
    defaultMessage: "Sponsor on GitHub",
  },
});
const creatorsMessages = defineMessages({
  title: {
    id: "project.about.creators.title",
    defaultMessage: "Creators",
  },
  owner: {
    id: "project.about.creators.owner",
    defaultMessage: "Project owner",
  },
});
const detailsMessages = defineMessages({
  title: {
    id: "project.about.details.title",
    defaultMessage: "Details",
  },
  licensed: {
    id: "project.about.details.licensed",
    defaultMessage: "Licensed {license}",
  },
  created: {
    id: "project.about.details.created",
    defaultMessage: "Created {date}",
  },
  submitted: {
    id: "project.about.details.submitted",
    defaultMessage: "Submitted {date}",
  },
  published: {
    id: "project.about.details.published",
    defaultMessage: "Published {date}",
  },
  updated: {
    id: "project.about.details.updated",
    defaultMessage: "Updated {date}",
  },
});

const modalLicense = ref(null);
const licenseText = ref("");

const createdDate = computed(() =>
  props.project.published ? formatRelativeTime(props.project.published) : "unknown",
);
const submittedDate = computed(() =>
  props.project.queued ? formatRelativeTime(props.project.queued) : "unknown",
);
const publishedDate = computed(() =>
  props.project.approved ? formatRelativeTime(props.project.approved) : "unknown",
);
const updatedDate = computed(() =>
  props.project.updated ? formatRelativeTime(props.project.updated) : "unknown",
);

const licenseIdDisplay = computed(() => {
  const id = props.project.license.id;

  if (id === "LicenseRef-All-Rights-Reserved") {
    return "ARR";
  } else if (id.includes("LicenseRef")) {
    return id.replaceAll("LicenseRef-", "").replaceAll("-", " ");
  } else {
    return id;
  }
});

async function getLicenseData(event) {
  modalLicense.value.show(event);

  try {
    const text = await useBaseFetch(`tag/license/${props.project.license.id}`);
    licenseText.value = text.body || "License text could not be retrieved.";
  } catch {
    licenseText.value = "License text could not be retrieved.";
  }
}
</script>
