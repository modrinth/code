<template>
  <div v-if="user" class="experimental-styles-within">
    <ModalCreation ref="modal_creation" />
    <CollectionCreateModal ref="modal_collection_creation" />
    <div class="new-page sidebar" :class="{ 'alt-layout': cosmetics.leftContentLayout }">
      <div class="normal-page__header py-4">
        <ContentPageHeader>
          <template #icon>
            <Avatar :src="user.avatar_url" :alt="user.username" size="96px" circle />
          </template>
          <template #title>
            {{ user.username }}
          </template>
          <template #summary>
            {{
              user.bio
                ? user.bio
                : projects.length === 0
                  ? "A Modrinth user."
                  : "A Modrinth creator."
            }}
          </template>
          <template #stats>
            <div
              class="flex items-center gap-2 border-0 border-r border-solid border-button-bg pr-4 font-semibold"
            >
              <BoxIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(projects?.length || 0) }}
              projects
            </div>
            <div
              class="flex items-center gap-2 border-0 border-r border-solid border-button-bg pr-4 font-semibold"
            >
              <DownloadIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(sumDownloads) }}
              downloads
            </div>
            <div class="flex items-center gap-2 font-semibold">
              <CalendarIcon class="h-6 w-6 text-secondary" />
              Joined
              {{ formatRelativeTime(user.created) }}
            </div>
          </template>
          <template #actions>
            <ButtonStyled size="large">
              <NuxtLink v-if="auth.user && auth.user.id === user.id" to="/settings/profile">
                <EditIcon aria-hidden="true" />
                {{ formatMessage(commonMessages.editButton) }}
              </NuxtLink>
            </ButtonStyled>
            <ButtonStyled size="large" circular type="transparent">
              <OverflowMenu
                :options="[
                  {
                    id: 'manage-projects',
                    action: () => navigateTo('/dashboard/projects'),
                    hoverOnly: true,
                    shown: auth.user && auth.user.id === user.id,
                  },
                  { divider: true, shown: auth.user && auth.user.id === user.id },
                  {
                    id: 'report',
                    action: () => reportUser(user.id),
                    color: 'red',
                    hoverOnly: true,
                  },
                  { id: 'copy-id', action: () => copyId() },
                ]"
                aria-label="More options"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #manage-projects>
                  <BoxIcon aria-hidden="true" />
                  {{ formatMessage(messages.profileManageProjectsButton) }}
                </template>
                <template #report>
                  <ReportIcon aria-hidden="true" />
                  {{ formatMessage(commonMessages.reportButton) }}
                </template>
                <template #copy-id>
                  <ClipboardCopyIcon aria-hidden="true" />
                  {{ formatMessage(commonMessages.copyIdButton) }}
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </template>
        </ContentPageHeader>
      </div>
      <div class="normal-page__content">
        <div v-if="navLinks.length > 2" class="mb-4 max-w-full overflow-x-auto">
          <NavTabs :links="navLinks" />
        </div>
        <div v-if="projects.length > 0">
          <div
            v-if="route.params.projectType !== 'collections'"
            :class="'project-list display-mode--' + cosmetics.searchDisplayMode.user"
          >
            <ProjectCard
              v-for="project in (route.params.projectType !== undefined
                ? projects.filter(
                    (x) =>
                      x.project_type ===
                      route.params.projectType.substr(0, route.params.projectType.length - 1),
                  )
                : projects
              )
                .slice()
                .sort((a, b) => b.downloads - a.downloads)"
              :id="project.slug || project.id"
              :key="project.id"
              :name="project.title"
              :display="cosmetics.searchDisplayMode.user"
              :featured-image="project.gallery.find((element) => element.featured)?.url"
              :description="project.description"
              :created-at="project.published"
              :updated-at="project.updated"
              :downloads="project.downloads.toString()"
              :follows="project.followers.toString()"
              :icon-url="project.icon_url"
              :categories="project.categories"
              :client-side="project.client_side"
              :server-side="project.server_side"
              :status="
                auth.user && (auth.user.id === user.id || tags.staffRoles.includes(auth.user.role))
                  ? project.status
                  : null
              "
              :type="project.project_type"
              :color="project.color"
            />
          </div>
        </div>
        <div v-else-if="route.params.projectType !== 'collections'" class="error">
          <UpToDate class="icon" /><br />
          <span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
            <IntlFormatted :message-id="messages.profileNoProjectsAuthLabel">
              <template #create-link="{ children }">
                <a class="link" @click.prevent="$refs.modal_creation.show()">
                  <component :is="() => children" />
                </a>
              </template>
            </IntlFormatted>
          </span>
          <span v-else class="text">{{ formatMessage(messages.profileNoProjectsLabel) }}</span>
        </div>
        <div v-if="['collections'].includes(route.params.projectType)" class="collections-grid">
          <nuxt-link
            v-for="collection in collections"
            :key="collection.id"
            :to="`/collection/${collection.id}`"
            class="card collection-item"
          >
            <div class="collection">
              <Avatar :src="collection.icon_url" class="icon" />
              <div class="details">
                <h2 class="title">{{ collection.name }}</h2>
                <div class="stats">
                  <LibraryIcon aria-hidden="true" />
                  Collection
                </div>
              </div>
            </div>
            <div class="description">
              {{ collection.description }}
            </div>
            <div class="stat-bar">
              <div class="stats"><BoxIcon /> {{ collection.projects?.length || 0 }} projects</div>
              <div class="stats">
                <template v-if="collection.status === 'listed'">
                  <WorldIcon />
                  <span> Public </span>
                </template>
                <template v-else-if="collection.status === 'unlisted'">
                  <LinkIcon />
                  <span> Unlisted </span>
                </template>
                <template v-else-if="collection.status === 'private'">
                  <LockIcon />
                  <span> Private </span>
                </template>
                <template v-else-if="collection.status === 'rejected'">
                  <XIcon />
                  <span> Rejected </span>
                </template>
              </div>
            </div>
          </nuxt-link>
        </div>
        <div
          v-if="route.params.projectType === 'collections' && collections.length === 0"
          class="error"
        >
          <UpToDate class="icon" /><br />
          <span v-if="auth.user && auth.user.id === user.id" class="preserve-lines text">
            <IntlFormatted :message-id="messages.profileNoCollectionsAuthLabel">
              <template #create-link="{ children }">
                <a
                  class="link"
                  @click.prevent="(event) => $refs.modal_collection_creation.show(event)"
                >
                  <component :is="() => children" />
                </a>
              </template>
            </IntlFormatted>
          </span>
          <span v-else class="text">{{ formatMessage(messages.profileNoCollectionsLabel) }}</span>
        </div>
      </div>
      <div class="normal-page__sidebar">
        <div v-if="organizations.length > 0" class="card flex-card">
          <h2 class="text-lg text-contrast">{{ formatMessage(messages.profileOrganizations) }}</h2>
          <div class="flex flex-wrap gap-2">
            <nuxt-link
              v-for="org in organizations"
              :key="org.id"
              v-tooltip="org.name"
              class="organization"
              :to="`/organization/${org.slug}`"
            >
              <Avatar :src="org.icon_url" :alt="'Icon for ' + org.name" size="3rem" />
            </nuxt-link>
          </div>
        </div>
        <div v-if="badges.length > 0" class="card flex-card">
          <h2 class="text-lg text-contrast">{{ formatMessage(messages.profileBadges) }}</h2>
          <div class="flex flex-wrap gap-2">
            <div v-for="badge in badges" :key="badge">
              <StaffBadge v-if="badge === 'staff'" class="h-14 w-14" />
              <ModBadge v-else-if="badge === 'mod'" class="h-14 w-14" />
              <nuxt-link v-else-if="badge === 'plus'" to="/plus">
                <PlusBadge class="h-14 w-14" />
              </nuxt-link>
              <TenMClubBadge v-else-if="badge === '10m-club'" class="h-14 w-14" />
              <EarlyAdopterBadge v-else-if="badge === 'early-adopter'" class="h-14 w-14" />
            </div>
          </div>
        </div>
        <AdPlaceholder
          v-if="!auth.user || !isPermission(auth.user.badges, 1 << 0) || flags.showAdsWithPlus"
        />
      </div>
    </div>
  </div>
</template>
<script setup>
import {
  LibraryIcon,
  BoxIcon,
  LinkIcon,
  LockIcon,
  XIcon,
  CalendarIcon,
  DownloadIcon,
  ClipboardCopyIcon,
  MoreVerticalIcon,
} from "@modrinth/assets";
import { OverflowMenu, ButtonStyled, ContentPageHeader } from "@modrinth/ui";
import NavTabs from "~/components/ui/NavTabs.vue";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import { reportUser } from "~/utils/report-helpers.ts";

import StaffBadge from "~/assets/images/badges/staff.svg?component";
import ModBadge from "~/assets/images/badges/mod.svg?component";
import PlusBadge from "~/assets/images/badges/plus.svg?component";
import TenMClubBadge from "~/assets/images/badges/10m-club.svg?component";
import EarlyAdopterBadge from "~/assets/images/badges/early-adopter.svg?component";

import ReportIcon from "~/assets/images/utils/report.svg?component";
import UpToDate from "~/assets/images/illustrations/up_to_date.svg?component";
import EditIcon from "~/assets/images/utils/edit.svg?component";
import WorldIcon from "~/assets/images/utils/world.svg?component";
import ModalCreation from "~/components/ui/ModalCreation.vue";
import Avatar from "~/components/ui/Avatar.vue";
import CollectionCreateModal from "~/components/ui/CollectionCreateModal.vue";
import AdPlaceholder from "~/components/ui/AdPlaceholder.vue";

const data = useNuxtApp();
const route = useNativeRoute();
const auth = await useAuth();
const cosmetics = useCosmetics();
const tags = useTags();
const flags = useFeatureFlags();

const vintl = useVIntl();
const { formatMessage } = vintl;

const formatCompactNumber = useCompactNumber();

const formatRelativeTime = useRelativeTime();

const messages = defineMessages({
  profileProjectsStats: {
    id: "profile.stats.projects",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> project} other {<stat>{count}</stat> projects}}",
  },
  profileDownloadsStats: {
    id: "profile.stats.downloads",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> project download} other {<stat>{count}</stat> project downloads}}",
  },
  profileProjectsFollowersStats: {
    id: "profile.stats.projects-followers",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> project follower} other {<stat>{count}</stat> project followers}}",
  },
  profileJoinedAt: {
    id: "profile.joined-at",
    defaultMessage: "Joined <date>{ago}</date>",
  },
  profileUserId: {
    id: "profile.user-id",
    defaultMessage: "User ID: {id}",
  },
  profileDetails: {
    id: "profile.label.details",
    defaultMessage: "Details",
  },
  profileOrganizations: {
    id: "profile.label.organizations",
    defaultMessage: "Organizations",
  },
  profileBadges: {
    id: "profile.label.badges",
    defaultMessage: "Badges",
  },
  profileManageProjectsButton: {
    id: "profile.button.manage-projects",
    defaultMessage: "Manage projects",
  },
  profileMetaDescription: {
    id: "profile.meta.description",
    defaultMessage: "Download {username}'s projects on Modrinth",
  },
  profileMetaDescriptionWithBio: {
    id: "profile.meta.description-with-bio",
    defaultMessage: "{bio} - Download {username}'s projects on Modrinth",
  },
  profileNoProjectsLabel: {
    id: "profile.label.no-projects",
    defaultMessage: "This user has no projects!",
  },
  profileNoProjectsAuthLabel: {
    id: "profile.label.no-projects-auth",
    defaultMessage:
      "You don't have any projects.\nWould you like to <create-link>create one</create-link>?",
  },
  profileNoCollectionsLabel: {
    id: "profile.label.no-collections",
    defaultMessage: "This user has no collections!",
  },
  profileNoCollectionsAuthLabel: {
    id: "profile.label.no-collections-auth",
    defaultMessage:
      "You don't have any collections.\nWould you like to <create-link>create one</create-link>?",
  },
  userNotFoundError: {
    id: "profile.error.not-found",
    defaultMessage: "User not found",
  },
});

let user, projects, organizations, collections;
try {
  [{ data: user }, { data: projects }, { data: organizations }, { data: collections }] =
    await Promise.all([
      useAsyncData(`user/${route.params.id}`, () => useBaseFetch(`user/${route.params.id}`)),
      useAsyncData(
        `user/${route.params.id}/projects`,
        () => useBaseFetch(`user/${route.params.id}/projects`),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders);
              project.project_type = data.$getProjectTypeForUrl(
                project.project_type,
                project.categories,
                tags.value,
              );
            }

            return projects;
          },
        },
      ),
      useAsyncData(`user/${route.params.id}/organizations`, () =>
        useBaseFetch(`user/${route.params.id}/organizations`, {
          apiVersion: 3,
        }),
      ),
      useAsyncData(`user/${route.params.id}/collections`, () =>
        useBaseFetch(`user/${route.params.id}/collections`, { apiVersion: 3 }),
      ),
    ]);
} catch {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

if (!user.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  });
}

if (user.value.username !== route.params.id) {
  await navigateTo(`/user/${user.value.username}`, { redirectCode: 301 });
}

const title = computed(() => `${user.value.username} - Modrinth`);
const description = computed(() =>
  user.value.bio
    ? formatMessage(messages.profileMetaDescriptionWithBio, {
        bio: user.value.bio,
        username: user.value.username,
      })
    : formatMessage(messages.profileMetaDescription, { username: user.value.username }),
);

useSeoMeta({
  title: () => title.value,
  description: () => description.value,
  ogTitle: () => title.value,
  ogDescription: () => description.value,
  ogImage: () => user.value.avatar_url ?? "https://cdn.modrinth.com/placeholder.png",
});

const projectTypes = computed(() => {
  const obj = {};

  if (collections.value.length > 0) {
    obj.collection = true;
  }

  for (const project of projects.value) {
    obj[project.project_type] = true;
  }

  delete obj.project;

  return Object.keys(obj);
});
const sumDownloads = computed(() => {
  let sum = 0;

  for (const project of projects.value) {
    sum += project.downloads;
  }

  return sum;
});

const badges = computed(() => {
  const badges = [];

  if (user.value.role === "admin") {
    badges.push("staff");
  }

  if (user.value.role === "moderator") {
    badges.push("mod");
  }

  if (isPermission(user.value.badges, 1 << 0)) {
    badges.push("plus");
  }

  if (sumDownloads.value > 10000000) {
    badges.push("10m-club");
  }

  if (
    isPermission(user.value.badges, 1 << 1) ||
    isPermission(user.value.badges, 1 << 2) ||
    isPermission(user.value.badges, 1 << 3)
  ) {
    badges.push("early-adopter");
  }

  if (isPermission(user.value.badges, 1 << 4)) {
    badges.push("alpha-tester");
  }

  if (isPermission(user.value.badges, 1 << 5)) {
    badges.push("contributor");
  }

  if (isPermission(user.value.badges, 1 << 6)) {
    badges.push("translator");
  }

  return badges;
});

async function copyId() {
  await navigator.clipboard.writeText(project.value.id);
}

const navLinks = computed(() => [
  {
    label: formatMessage(commonMessages.allProjectType),
    href: `/user/${user.value.username}`,
  },
  ...projectTypes.value
    .map((x) => {
      return {
        label: formatMessage(getProjectTypeMessage(x, true)),
        href: `/user/${user.value.username}/${x}s`,
      };
    })
    .slice()
    .sort((a, b) => a.label.localeCompare(b.label)),
]);
</script>
<script>
export default defineNuxtComponent({
  methods: {},
});
</script>

<style lang="scss" scoped>
.collections-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);

  @media screen and (max-width: 800px) {
    grid-template-columns: repeat(1, 1fr);
  }

  gap: var(--gap-lg);

  .collection-item {
    display: flex;
    flex-direction: column;
    gap: var(--gap-md);
  }

  .description {
    // Grow to take up remaining space
    flex-grow: 1;

    color: var(--color-text);
    font-size: 16px;
  }

  .stat-bar {
    display: flex;
    align-items: center;
    gap: var(--gap-md);
    margin-top: auto;
  }

  .stats {
    display: flex;
    align-items: center;
    gap: var(--gap-xs);

    svg {
      color: var(--color-secondary);
    }
  }

  .collection {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--gap-md);

    .icon {
      width: 100% !important;
      height: 6rem !important;
      max-width: unset !important;
      max-height: unset !important;
      aspect-ratio: 1 / 1;
      object-fit: cover;
    }

    .details {
      display: flex;
      flex-direction: column;
      gap: var(--gap-sm);

      .title {
        color: var(--color-contrast);
        font-weight: 600;
        font-size: var(--font-size-lg);
        margin: 0;
      }
    }
  }
}
</style>
