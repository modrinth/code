<template>
  <div
    v-if="organization"
    class="experimental-styles-within new-page sidebar"
    :class="{ 'alt-layout': cosmetics.leftContentLayout || routeHasSettings }"
  >
    <ModalCreation ref="modal_creation" :organization-id="organization.id" />
    <template v-if="routeHasSettings">
      <div class="normal-page__sidebar">
        <div class="universal-card">
          <Breadcrumbs
            current-title="Settings"
            :link-stack="[
              { href: `/dashboard/organizations`, label: 'Organizations' },
              {
                href: `/organization/${organization.slug}`,
                label: organization.name,
                allowTrimming: true,
              },
            ]"
          />
          <div class="page-header__settings">
            <Avatar size="sm" :src="organization.icon_url" />
            <div class="title-section">
              <h2 class="settings-title">
                <nuxt-link :to="`/organization/${organization.slug}/settings`">
                  {{ organization.name }}
                </nuxt-link>
              </h2>
              <span>
                {{ $formatNumber(acceptedMembers?.length || 0) }}
                member<template v-if="acceptedMembers?.length !== 1">s</template>
              </span>
            </div>
          </div>

          <h2>Organization settings</h2>

          <NavStack>
            <NavStackItem :link="`/organization/${organization.slug}/settings`" label="Overview">
              <SettingsIcon />
            </NavStackItem>
            <NavStackItem
              :link="`/organization/${organization.slug}/settings/members`"
              label="Members"
            >
              <UsersIcon />
            </NavStackItem>
            <NavStackItem
              :link="`/organization/${organization.slug}/settings/projects`"
              label="Projects"
            >
              <BoxIcon />
            </NavStackItem>
            <NavStackItem
              :link="`/organization/${organization.slug}/settings/analytics`"
              label="Analytics"
            >
              <ChartIcon />
            </NavStackItem>
          </NavStack>
        </div>
      </div>
      <div class="normal-page__content">
        <NuxtPage />
      </div>
    </template>
    <template v-else>
      <div class="normal-page__header py-4">
        <ContentPageHeader>
          <template #icon>
            <Avatar :src="organization.icon_url" :alt="organization.name" size="96px" />
          </template>
          <template #title>
            {{ organization.name }}
          </template>
          <template #title-suffix>
            <div class="ml-1 flex items-center gap-2 font-semibold">
              <OrganizationIcon />
              Organization
            </div>
          </template>
          <template #summary>
            {{ organization.description }}
          </template>
          <template #stats>
            <div
              class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
            >
              <UsersIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(acceptedMembers?.length || 0) }}
              members
            </div>
            <div
              class="flex items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
            >
              <BoxIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(projects?.length || 0) }}
              projects
            </div>
            <div
              v-tooltip="sumDownloads.toLocaleString()"
              class="flex items-center gap-2 font-semibold"
            >
              <DownloadIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(sumDownloads) }}
              downloads
            </div>
          </template>
          <template #actions>
            <ButtonStyled v-if="auth.user && currentMember" size="large">
              <NuxtLink :to="`/organization/${organization.slug}/settings`">
                <SettingsIcon aria-hidden="true" />
                Manage
              </NuxtLink>
            </ButtonStyled>
            <ButtonStyled size="large" circular type="transparent">
              <OverflowMenu
                :options="[
                  {
                    id: 'manage-projects',
                    action: () =>
                      navigateTo('/organization/' + organization.slug + '/settings/projects'),
                    hoverOnly: true,
                    shown: auth.user && currentMember,
                  },
                  { divider: true, shown: auth.user && currentMember },
                  { id: 'copy-id', action: () => copyId() },
                  { id: 'copy-permalink', action: () => copyPermalink() },
                ]"
                aria-label="More options"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #manage-projects>
                  <BoxIcon aria-hidden="true" />
                  Manage projects
                </template>
                <template #copy-id>
                  <ClipboardCopyIcon aria-hidden="true" />
                  {{ formatMessage(commonMessages.copyIdButton) }}
                </template>
                <template #copy-permalink>
                  <ClipboardCopyIcon aria-hidden="true" />
                  {{ formatMessage(commonMessages.copyPermalinkButton) }}
                </template>
              </OverflowMenu>
            </ButtonStyled>
          </template>
        </ContentPageHeader>
      </div>
      <div class="normal-page__sidebar">
        <AdPlaceholder v-if="!auth.user" />

        <div class="card flex-card">
          <h2>Members</h2>
          <div class="details-list">
            <template v-for="member in acceptedMembers" :key="member.user.id">
              <nuxt-link
                class="details-list__item details-list__item--type-large"
                :to="`/user/${member.user.username}`"
              >
                <Avatar :src="member.user.avatar_url" circle />
                <div class="rows">
                  <span class="flex items-center gap-1">
                    {{ member.user.username }}
                    <CrownIcon
                      v-if="member.is_owner"
                      v-tooltip="'Organization owner'"
                      class="text-brand-orange"
                    />
                  </span>
                  <span class="details-list__item__text--style-secondary">
                    {{ member.role ? member.role : "Member" }}
                  </span>
                </div>
              </nuxt-link>
            </template>
          </div>
        </div>
      </div>
      <div class="normal-page__content">
        <div v-if="isInvited" class="universal-card information invited">
          <h2>Invitation to join {{ organization.name }}</h2>
          <p>You have been invited to join {{ organization.name }}.</p>
          <div class="input-group">
            <button class="iconified-button brand-button" @click="onAcceptInvite">
              <CheckIcon />
              Accept
            </button>
            <button class="iconified-button danger-button" @click="onDeclineInvite">
              <XIcon />
              Decline
            </button>
          </div>
        </div>
        <div v-if="navLinks.length > 2" class="mb-4 max-w-full overflow-x-auto">
          <NavTabs :links="navLinks" />
        </div>
        <template v-if="projects?.length > 0">
          <div class="project-list display-mode--list">
            <ProjectCard
              v-for="project in (route.params.projectType !== undefined
                ? projects.filter((x) =>
                    x.project_types.includes(
                      route.params.projectType.substr(0, route.params.projectType.length - 1),
                    ),
                  )
                : projects
              )
                .slice()
                .sort((a, b) => b.downloads - a.downloads)"
              :id="project.slug || project.id"
              :key="project.id"
              :name="project.name"
              :display="cosmetics.searchDisplayMode.user"
              :featured-image="project.gallery.find((element) => element.featured)?.url"
              project-type-url="project"
              :description="project.summary"
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
              :type="project.project_types[0] ?? 'project'"
              :color="project.color"
            />
          </div>
        </template>

        <div v-else-if="true" class="error">
          <UpToDate class="icon" />
          <br />
          <span class="preserve-lines text">
            This organization doesn't have any projects yet.
            <template v-if="isPermission(currentMember?.organization_permissions, 1 << 4)">
              Would you like to
              <a class="link" @click="$refs.modal_creation.show()">create one</a>?
            </template>
          </span>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup>
import {
  BoxIcon,
  MoreVerticalIcon,
  UsersIcon,
  SettingsIcon,
  ChartIcon,
  CheckIcon,
  XIcon,
  ClipboardCopyIcon,
  OrganizationIcon,
  DownloadIcon,
  CrownIcon,
} from "@modrinth/assets";
import {
  Avatar,
  ButtonStyled,
  Breadcrumbs,
  ContentPageHeader,
  OverflowMenu,
  commonMessages,
} from "@modrinth/ui";
import NavStack from "~/components/ui/NavStack.vue";
import NavStackItem from "~/components/ui/NavStackItem.vue";
import ModalCreation from "~/components/ui/ModalCreation.vue";
import UpToDate from "~/assets/images/illustrations/up_to_date.svg?component";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import AdPlaceholder from "~/components/ui/AdPlaceholder.vue";
import { acceptTeamInvite, removeTeamMember } from "~/helpers/teams.js";
import NavTabs from "~/components/ui/NavTabs.vue";

const vintl = useVIntl();
const { formatMessage } = vintl;

const formatCompactNumber = useCompactNumber(true);

const auth = await useAuth();
const user = await useUser();
const cosmetics = useCosmetics();
const route = useNativeRoute();
const tags = useTags();
const config = useRuntimeConfig();

let orgId = useRouteId();

// hacky way to show the edit button on the corner of the card.
const routeHasSettings = computed(() => route.path.includes("settings"));

const [
  { data: organization, refresh: refreshOrganization },
  { data: projects, refresh: refreshProjects },
] = await Promise.all([
  useAsyncData(`organization/${orgId}`, () =>
    useBaseFetch(`organization/${orgId}`, { apiVersion: 3 }),
  ),
  useAsyncData(
    `organization/${orgId}/projects`,
    () => useBaseFetch(`organization/${orgId}/projects`, { apiVersion: 3 }),
    {
      transform: (projects) => {
        for (const project of projects) {
          project.categories = project.categories.concat(project.loaders);

          if (project.mrpack_loaders) {
            project.categories = project.categories.concat(project.mrpack_loaders);
          }

          const singleplayer = project.singleplayer && project.singleplayer[0];
          const clientAndServer = project.client_and_server && project.client_and_server[0];
          const clientOnly = project.client_only && project.client_only[0];
          const serverOnly = project.server_only && project.server_only[0];

          // quick and dirty hack to show envs as legacy
          if (singleplayer && clientAndServer && !clientOnly && !serverOnly) {
            project.client_side = "required";
            project.server_side = "required";
          } else if (singleplayer && clientAndServer && clientOnly && !serverOnly) {
            project.client_side = "required";
            project.server_side = "unsupported";
          } else if (singleplayer && clientAndServer && !clientOnly && serverOnly) {
            project.client_side = "unsupported";
            project.server_side = "required";
          } else if (singleplayer && clientAndServer && clientOnly && serverOnly) {
            project.client_side = "optional";
            project.server_side = "optional";
          }
        }

        return projects;
      },
    },
  ),
]);

const refresh = async () => {
  await Promise.all([refreshOrganization(), refreshProjects()]);
};

if (!organization.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: "Organization not found",
  });
}

// Filter accepted, sort by role, then by name and Owner role always goes first
const acceptedMembers = computed(() => {
  const acceptedMembers = organization.value.members?.filter((x) => x.accepted);
  const owner = acceptedMembers.find((x) => x.is_owner);
  const rest = acceptedMembers.filter((x) => !x.is_owner) || [];

  rest.sort((a, b) => {
    if (a.role === b.role) {
      return a.user.username.localeCompare(b.user.username);
    } else {
      return a.role.localeCompare(b.role);
    }
  });

  return [owner, ...rest];
});

const currentMember = computed(() => {
  if (auth.value.user && organization.value) {
    const member = organization.value.members.find((x) => x.user.id === auth.value.user.id);

    if (member) {
      return member;
    }

    if (tags.value.staffRoles.includes(auth.value.user.role)) {
      return {
        user: auth.value.user,
        role: auth.value.user.role,
        permissions: auth.value.user.role === "admin" ? 1023 : 12,
        accepted: true,
        payouts_split: 0,
        avatar_url: auth.value.user.avatar_url,
        name: auth.value.user.username,
      };
    }
  }

  return null;
});

const hasPermission = computed(() => {
  const EDIT_DETAILS = 1 << 2;
  return currentMember.value && (currentMember.value.permissions & EDIT_DETAILS) === EDIT_DETAILS;
});

const isInvited = computed(() => {
  return currentMember.value?.accepted === false;
});

const projectTypes = computed(() => {
  const obj = {};

  for (const project of projects.value) {
    obj[project.project_types[0] ?? "project"] = true;
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

const patchIcon = async (icon) => {
  const ext = icon.name.split(".").pop();
  await useBaseFetch(`organization/${organization.value.id}/icon`, {
    method: "PATCH",
    body: icon,
    query: { ext },
    apiVersion: 3,
  });
};

const deleteIcon = async () => {
  await useBaseFetch(`organization/${organization.value.id}/icon`, {
    method: "DELETE",
    apiVersion: 3,
  });
};

const patchOrganization = async (id, newData) => {
  await useBaseFetch(`organization/${id}`, {
    method: "PATCH",
    body: newData,
    apiVersion: 3,
  });

  if (newData.slug) {
    orgId = newData.slug;
  }
};

const onAcceptInvite = useClientTry(async () => {
  await acceptTeamInvite(organization.value.team_id);
  await refreshOrganization();
});

const onDeclineInvite = useClientTry(async () => {
  await removeTeamMember(organization.value.team_id, auth.value?.user.id);
  await refreshOrganization();
});

provide("organizationContext", {
  organization,
  projects,
  refresh,
  currentMember,
  hasPermission,
  patchIcon,
  deleteIcon,
  patchOrganization,
});

const title = `${organization.value.name} - Organization`;
const description = `${organization.value.description} - View the organization ${organization.value.name} on Modrinth`;

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: organization.value.description,
  ogImage: organization.value.icon_url ?? "https://cdn.modrinth.com/placeholder.png",
});

const navLinks = computed(() => [
  {
    label: formatMessage(commonMessages.allProjectType),
    href: `/organization/${organization.value.slug}`,
  },
  ...projectTypes.value
    .map((x) => {
      return {
        label: formatMessage(getProjectTypeMessage(x, true)),
        href: `/organization/${organization.value.slug}/${x}s`,
      };
    })
    .slice()
    .sort((a, b) => a.label.localeCompare(b.label)),
]);

async function copyId() {
  await navigator.clipboard.writeText(organization.value.id);
}

async function copyPermalink() {
  await navigator.clipboard.writeText(
    `${config.public.siteUrl}/organization/${organization.value.id}`,
  );
}
</script>

<style scoped lang="scss">
.page-header__settings {
  display: flex;
  flex-direction: row;
  gap: var(--gap-md);
  margin-bottom: var(--gap-md);

  .title-section {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: var(--gap-xs);
  }

  .settings-title {
    margin: 0 !important;
    font-size: var(--font-size-md);
  }
}

.page-header__icon {
  margin-block: 0 !important;
}

.universal-card {
  h1 {
    margin-bottom: var(--gap-md);
  }
}

.creator-list {
  display: flex;
  flex-direction: column;
  padding: var(--gap-xl);

  h3 {
    margin: 0 0 var(--gap-sm);
  }

  .creator {
    display: grid;
    gap: var(--gap-xs);
    background-color: var(--color-raised-bg);
    padding: var(--gap-sm);
    margin-left: -0.5rem;
    border-radius: var(--radius-lg);
    grid-template:
      "avatar name" auto
      "avatar role" auto
      / auto 1fr;

    p {
      margin: 0;
    }

    .name {
      grid-area: name;
      align-self: flex-end;
      margin-left: var(--gap-xs);
      font-weight: bold;

      display: flex;
      align-items: center;
      gap: 0.25rem;

      svg {
        color: var(--color-orange);
      }
    }

    .role {
      grid-area: role;
      align-self: flex-start;
      margin-left: var(--gap-xs);
    }

    .avatar {
      grid-area: avatar;
    }
  }
}

.secondary-stat {
  align-items: center;
  display: flex;
  margin-bottom: 0.8rem;
}

.secondary-stat__icon {
  height: 1rem;
  width: 1rem;
}

.secondary-stat__text {
  margin-left: 0.4rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.title {
  margin: var(--gap-md) 0 var(--spacing-card-xs) 0;
  font-size: var(--font-size-xl);
  color: var(--color-text-dark);
}

.organization-label {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.organization-description {
  margin-top: var(--spacing-card-sm);
  margin-bottom: 0;
}

.title-and-link {
  display: flex;
  justify-content: space-between;
  align-items: center;

  h3 {
    margin: 0;
  }

  a {
    display: flex;
    align-items: center;
    gap: var(--gap-xs);
    color: var(--color-blue);
  }
}

.project-overview {
  gap: var(--gap-md);
  padding: var(--gap-xl);

  .project-card {
    padding: 0;
    border-radius: 0;
    background-color: transparent;
    box-shadow: none;

    :deep(.title) {
      font-size: var(--font-size-nm) !important;
    }
  }
}

.popout-heading {
  padding: var(--gap-sm) var(--gap-md);
  margin: 0;
  font-size: var(--font-size-md);
  color: var(--color-text);
}

.popout-checkbox {
  padding: var(--gap-sm) var(--gap-md);
}
</style>
