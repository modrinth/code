<template>
  <div v-if="user" class="experimental-styles-within">
    <ModalCreation ref="modal_creation" />
    <CollectionCreateModal ref="modal_collection_creation" />
    <NewModal v-if="auth.user && isStaff(auth.user)" ref="userDetailsModal" header="User details">
      <div class="flex flex-col gap-3">
        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold text-primary">Email</span>
          <div>
            <span
              v-tooltip="user.email_verified ? 'Email verified' : 'Email not verified'"
              class="flex w-fit items-center gap-1"
            >
              <span>{{ user.email }}</span>
              <CheckIcon v-if="user.email_verified" class="h-4 w-4 text-brand" />
              <XIcon v-else class="h-4 w-4 text-red" />
            </span>
          </div>
        </div>

        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold text-primary"> Auth providers </span>
          <span>{{ user.auth_providers.join(", ") }}</span>
        </div>

        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold text-primary"> Payment methods</span>
          <span>
            <template v-if="user.payout_data?.paypal_address">
              Paypal ({{ user.payout_data.paypal_address }} - {{ user.payout_data.paypal_country }})
            </template>
            <template v-if="user.payout_data?.paypal_address && user.payout_data?.venmo_address">
              ,
            </template>
            <template v-if="user.payout_data?.venmo_address">
              Venmo ({{ user.payout_data.venmo_address }})
            </template>
          </span>
        </div>

        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold text-primary"> Has password </span>
          <span>
            {{ user.has_password ? "Yes" : "No" }}
          </span>
        </div>

        <div class="flex flex-col gap-1">
          <span class="text-lg font-bold text-primary"> Has TOTP </span>
          <span>
            {{ user.has_totp ? "Yes" : "No" }}
          </span>
        </div>
      </div>
    </NewModal>
    <div class="new-page sidebar" :class="{ 'alt-layout': cosmetics.leftContentLayout }">
      <div class="normal-page__header py-4">
        <UserHeader :user="user" :project-count="projects.length" :download-count="sumDownloads">
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
                    action: () => (auth.user ? reportUser(user.id) : navigateTo('/auth/sign-in')),
                    color: 'red',
                    hoverOnly: true,
                    shown: auth.user?.id !== user.id,
                  },
                  { id: 'copy-id', action: () => copyId() },
                  {
                    id: 'open-billing',
                    action: () => navigateTo(`/admin/billing/${user.id}`),
                    shown: auth.user && isStaff(auth.user),
                  },
                  {
                    id: 'open-info',
                    action: () => $refs.userDetailsModal.show(),
                    shown: auth.user && isStaff(auth.user),
                  },
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
                <template #open-billing>
                  <CurrencyIcon aria-hidden="true" />
                  {{ formatMessage(messages.billingButton) }}
                </template>
                <template #open-info>
                  <InfoIcon aria-hidden="true" />
                  {{ formatMessage(messages.infoButton) }}
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </template>
        </UserHeader>
      </div>
      <div class="normal-page__content">
        <ProjectsList
          v-if="flags.newProjectListUserPage"
          :projects="projects.filter((x) => x.status === 'approved' || x.status === 'archived')"
          :project-link="(project) => `/project/${project.id}`"
          :experimental-colors="flags.projectCardBackground"
        >
          <template #project-actions>
            <ButtonStyled color="brand">
              <button><DownloadIcon /> Install</button>
            </ButtonStyled>
            <ButtonStyled circular>
              <button v-tooltip="'Follow'">
                <HeartIcon />
              </button>
            </ButtonStyled>
            <ButtonStyled circular>
              <button v-tooltip="'Save'">
                <BookmarkIcon />
              </button>
            </ButtonStyled>
            <ButtonStyled circular type="transparent">
              <button>
                <MoreVerticalIcon />
              </button>
            </ButtonStyled>
          </template>
        </ProjectsList>
        <template v-else>
          <div v-if="navLinks.length >= 2" class="mb-4 max-w-full overflow-x-auto">
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
                  auth.user &&
                  (auth.user.id === user.id || tags.staffRoles.includes(auth.user.role))
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
              v-for="collection in collections.sort(
                (a, b) => new Date(b.created) - new Date(a.created),
              )"
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
                <div class="stats">
                  <BoxIcon />
                  {{
                    `${$formatNumber(collection.projects?.length || 0, false)} project${(collection.projects?.length || 0) !== 1 ? "s" : ""}`
                  }}
                </div>
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
        </template>
      </div>
      <div class="normal-page__sidebar">
        <UserSidebarOrganizations
          :organizations="organizations"
          :link="(org) => '/organization/' + org.slug"
          class="card flex-card experimental-styles-within"
        />
        <UserSidebarBadges
          :user="user"
          :download-count="sumDownloads"
          class="card flex-card experimental-styles-within"
        />
        <UserSidebarCollections
          :collections="collections.filter((x) => x.status === 'listed')"
          :link="(collection) => `/collection/${collection.id}`"
          class="card flex-card experimental-styles-within"
        />
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
  ClipboardCopyIcon,
  MoreVerticalIcon,
  DownloadIcon,
  BookmarkIcon,
  HeartIcon,
  CurrencyIcon,
  InfoIcon,
  CheckIcon,
} from "@modrinth/assets";
import {
  OverflowMenu,
  ButtonStyled,
  NewModal,
  UserHeader,
  commonMessages,
  UserSidebarBadges,
  UserSidebarOrganizations,
  ProjectsList,
  UserSidebarCollections,
} from "@modrinth/ui";
import { isStaff } from "~/helpers/users.js";
import NavTabs from "~/components/ui/NavTabs.vue";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import { reportUser } from "~/utils/report-helpers.ts";

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
  billingButton: {
    id: "profile.button.billing",
    defaultMessage: "Manage user billing",
  },
  infoButton: {
    id: "profile.button.info",
    defaultMessage: "View user details",
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

async function copyId() {
  await navigator.clipboard.writeText(user.value.id);
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

  gap: var(--gap-md);

  .collection-item {
    display: flex;
    flex-direction: column;
    gap: var(--gap-md);
    margin-bottom: 0px;
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
        font-weight: 700;
        font-size: var(--font-size-lg);
        margin: 0;
      }
    }
  }
}
</style>
