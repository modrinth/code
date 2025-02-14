<template>
  <div class="experimental-styles-within">
    <!-- COLLECTION DELETION -->
    <ConfirmModal
      v-if="canEdit"
      ref="deleteModal"
      :title="formatMessage(messages.deleteModalTitle)"
      :description="formatMessage(messages.deleteModalDescription)"
      :has-to-type="false"
      :proceed-label="formatMessage(commonMessages.deleteLabel)"
      @proceed="deleteCollection()"
    />
    <!-- COLLECTION EDITING -->
    <NewModal
      v-if="canEdit"
      ref="editCollectionModal"
      :on-show="() => (isEditing = true)"
      :on-hide="() => (isEditing = false)"
      header="Editing a collection"
    >
      <div class="flex flex-col gap-3">
        <span class="text-lg font-semibold text-contrast"> Preview </span>
        <div
          class="flex flex-row items-center gap-3 overflow-x-hidden rounded-3xl bg-bg p-3 sm:overflow-x-auto"
        >
          <Avatar
            size="md"
            :src="deletedIcon ? null : previewImage ? previewImage : collection.icon_url"
          />
          <div class="short flex flex-col justify-between gap-2">
            <div class="flex w-48 flex-col gap-0 sm:w-96 sm:max-w-96">
              <span
                class="line-clamp-2 overflow-ellipsis text-wrap text-xl font-extrabold text-contrast sm:line-clamp-1"
                >{{ name != "" ? name : "Collection Name" }}</span
              >
              <span class="line-clamp-2 w-full overflow-ellipsis sm:line-clamp-1">{{
                summary != "" ? summary : "Collection summary"
              }}</span>
            </div>
            <PopoutMenu class="btn">
              <EditIcon aria-hidden="true" />
              {{ formatMessage(messages.editIconButton) }}
              <template #menu>
                <span class="icon-edit-menu h-fit">
                  <FileInput
                    id="project-icon"
                    :max-size="262144"
                    :show-icon="true"
                    accept="image/png,image/jpeg,image/gif,image/webp"
                    class="btn btn-transparent upload"
                    style="white-space: nowrap"
                    aria-label="Upload icon"
                    @change="showPreviewImage"
                  >
                    <UploadIcon aria-hidden="true" />
                  </FileInput>
                  <Button
                    v-if="!deletedIcon && (previewImage || collection.icon_url)"
                    style="white-space: nowrap"
                    transparent
                    @click="
                      () => {
                        deletedIcon = true;
                        previewImage = null;
                      }
                    "
                  >
                    <TrashIcon aria-hidden="true" />
                    {{ formatMessage(messages.deleteIconButton) }}
                  </Button>
                </span>
              </template>
            </PopoutMenu>
          </div>
        </div>
        <div class="flex flex-col gap-2">
          <label for="name">
            <span class="text-lg font-semibold text-contrast"> Name </span>
          </label>
          <input
            id="name"
            v-model="name"
            type="text"
            maxlength="64"
            :placeholder="`Enter collection name...`"
            autocomplete="off"
          />
        </div>
        <div class="flex flex-col gap-2">
          <span class="text-lg font-semibold text-contrast"> Visibility </span>
          <DropdownSelect
            id="visibility"
            v-model="visibility"
            :options="['listed', 'unlisted', 'private']"
            :disabled="visibility === 'rejected'"
            :multiple="false"
            :display-name="
              (s) => {
                if (s === 'listed') return formatMessage(commonMessages.publicLabel);
                return formatMessage(commonMessages[`${s}Label`]);
              }
            "
            :searchable="false"
          />
        </div>
        <div class="flex flex-col gap-2">
          <label for="summary" class="flex flex-col gap-1">
            <span class="text-lg font-semibold text-contrast"> Summary </span>
            <span>A sentence or two that describes your collection.</span>
          </label>
          <div class="textarea-wrapper">
            <textarea id="summary" v-model="summary" maxlength="256" />
          </div>
        </div>
        <div class="flex gap-2">
          <ButtonStyled color="brand">
            <button
              @click="
                saveChanges();
                editCollectionModal.hide();
              "
            >
              <SaveIcon aria-hidden="true" />
              Save
            </button>
          </ButtonStyled>
          <ButtonStyled>
            <button @click="editCollectionModal.hide()">
              <XIcon aria-hidden="true" />
              Cancel
            </button>
          </ButtonStyled>
        </div>
      </div>
    </NewModal>
    <ShareModal
      ref="shareModal"
      :share-title="`${collection.name} on Modrinth`"
      :share-text="
        canEdit
          ? `Check out my collection, ${collection.name} on Modrinth!\n\nhttps://modrinth.com/collection/${collection.id}`
          : `Check out the collection ${collection.name} on Modrinth!\n\nhttps://modrinth.com/collection/${collection.id}`
      "
      header="Sharing a collection"
    />
    <div class="new-page sidebar" :class="{ 'alt-layout': cosmetics.leftContentLayout }">
      <div class="normal-page__header py-4">
        <ContentPageHeader class="collectionInput">
          <template #icon>
            <Avatar :src="collection.icon_url" :alt="collection.name" size="96px" />
          </template>
          <template #title>
            {{ collection.name }}
          </template>
          <template #title-suffix>
            <div class="ml-1 flex items-center gap-2 font-semibold">
              <CollectionIcon /> Collection
            </div>
          </template>
          <template #summary>
            {{ collection.description }}
          </template>
          <template #stats>
            <div
              v-if="canEdit"
              class="flex cursor-help items-center border-0 border-r border-solid border-divider pr-4"
            >
              <SimpleBadge
                v-if="collection.status === 'listed'"
                v-tooltip="'This collection is visible to all users.'"
                :icon="WorldIcon"
                :formatted-name="formatMessage(commonMessages.publicLabel)"
              />
              <SimpleBadge
                v-else-if="collection.status === 'unlisted'"
                v-tooltip="'This collection is only visible to users with the link.'"
                :icon="LinkIcon"
                :formatted-name="formatMessage(commonMessages.unlistedLabel)"
              />
              <SimpleBadge
                v-else-if="collection.status === 'private'"
                v-tooltip="'This collection is only visible to you.'"
                :icon="LockIcon"
                :formatted-name="formatMessage(commonMessages.privateLabel)"
              />
              <SimpleBadge
                v-else-if="collection.status === 'rejected'"
                v-tooltip="'This collection has been rejected.'"
                :icon="XIcon"
                :formatted-name="formatMessage(commonMessages.rejectedLabel)"
              />
            </div>
            <div
              v-tooltip="
                `${$formatNumber(projects.length || 0, false)} project${(projects.length || 0) !== 1 ? 's' : ''}`
              "
              class="flex cursor-help items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
            >
              <BoxIcon class="h-6 w-6 text-secondary" />
              {{ formatCompactNumber(projects.length || 0) }}
            </div>
            <div
              v-tooltip="
                formatMessage(commonMessages.dateAtTimeTooltip, {
                  date: new Date(collection.created),
                  time: new Date(collection.created),
                })
              "
              class="flex cursor-help items-center gap-2 border-0 border-r border-solid border-divider pr-4 font-semibold"
            >
              <CalendarIcon class="h-6 w-6 text-secondary" />

              {{
                formatMessage(messages.createdAtLabel, {
                  ago: formatRelativeTime(collection.created),
                })
              }}
            </div>
            <div
              v-tooltip="
                formatMessage(commonMessages.dateAtTimeTooltip, {
                  date: new Date(collection.updated),
                  time: new Date(collection.updated),
                })
              "
              class="flex cursor-help items-center gap-2 font-semibold"
            >
              <UpdatedIcon class="h-6 w-6 text-secondary" />

              {{
                formatMessage(messages.updatedAtLabel, {
                  ago: formatRelativeTime(collection.updated),
                })
              }}
            </div>
          </template>
          <template #actions>
            <ButtonStyled v-if="canEdit" size="large">
              <button @click="$refs.editCollectionModal.show()">
                <EditIcon aria-hidden="true" />
                {{ formatMessage(commonMessages.editButton) }}
              </button>
            </ButtonStyled>
            <ButtonStyled circular size="large">
              <button @click="$refs.shareModal.show()">
                <ShareIcon />
              </button>
            </ButtonStyled>
            <ButtonStyled size="large" circular>
              <OverflowMenu
                :options="[
                  {
                    id: 'delete',
                    action: () => $refs.deleteModal.show(),
                    color: 'red',
                    hoverOnly: true,
                    shown: canEdit && isEditing === false,
                  },
                  { id: 'copy-id', action: () => copyId() },
                ]"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #delete>
                  <TrashIcon aria-hidden="true" />
                  {{ formatMessage(commonMessages.deleteLabel) }}
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
      <div class="normal-page__sidebar">
        <div class="card flex-card">
          <h2>{{ formatMessage(messages.curatedByLabel) }}</h2>

          <nuxt-link
            class="details-list__item details-list__item--type-large"
            :to="`/user/${creator.username}`"
          >
            <Avatar :src="creator.avatar_url" circle />
            <div class="rows">
              <span class="flex items-center gap-1 font-bold text-primary">
                {{ creator.username }}
              </span>
              <span class="text-sm font-medium text-secondary">
                {{ formatMessage(messages.ownerLabel) }}
              </span>
            </div>
          </nuxt-link>
        </div>
        <AdPlaceholder
          v-if="!auth.user || !isPermission(auth.user.badges, 1 << 0) || flags.showAdsWithPlus"
        />
      </div>
      <div class="normal-page__content">
        <div v-if="navLinks.length >= 2" class="mb-4 flex flex-row items-center gap-2">
          <div class="max-w-full overflow-x-auto">
            <NavTabs :links="navLinks" />
          </div>
          <ButtonStyled circular>
            <button
              v-tooltip="
                formatMessage(
                  commonMessages[`${cosmetics.searchDisplayMode.collection || 'list'}InputView`],
                )
              "
              :aria-label="
                formatMessage(
                  commonMessages[`${cosmetics.searchDisplayMode.collection || 'list'}InputView`],
                )
              "
              class="square-button"
              @click="cycleSearchDisplayMode()"
            >
              <GridIcon v-if="cosmetics.searchDisplayMode.collection === 'grid'" />
              <ImageIcon v-else-if="cosmetics.searchDisplayMode.collection === 'gallery'" />
              <ListIcon v-else />
            </button>
          </ButtonStyled>
        </div>

        <div
          v-if="projects && projects?.length > 0"
          :class="
            'project-list display-mode--' + (cosmetics.searchDisplayMode.collection || 'list')
          "
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
            :id="project.id"
            :key="project.id"
            :type="project.project_type"
            :categories="project.categories"
            :created-at="project.published"
            :updated-at="project.updated"
            :description="project.description"
            :downloads="project.downloads ? project.downloads.toString() : '0'"
            :follows="project.followers ? project.followers.toString() : '0'"
            :featured-image="project.gallery.find((element) => element.featured)?.url"
            :icon-url="project.icon_url"
            :name="project.title"
            :client-side="project.client_side"
            :server-side="project.server_side"
            :color="project.color"
            :show-updated-date="!canEdit && collection.id !== 'following'"
            :show-created-date="!canEdit && collection.id !== 'following'"
          >
            <button
              v-if="canEdit"
              class="iconified-button remove-btn"
              @click="
                () => {
                  removeProjects = [project];
                  saveChanges();
                }
              "
            >
              <TrashIcon aria-hidden="true" />
              {{ formatMessage(messages.removeProjectButton) }}
            </button>
            <button
              v-if="collection.id === 'following'"
              class="iconified-button"
              @click="unfollowProject(project)"
            >
              <TrashIcon aria-hidden="true" />
              {{ formatMessage(messages.unfollowProjectButton) }}
            </button>
          </ProjectCard>
        </div>
        <div v-else class="error">
          <UpToDate class="icon" /><br />
          <span v-if="auth.user && auth.user.id === creator.id" class="preserve-lines text">
            <IntlFormatted :message-id="messages.noProjectsAuthLabel">
              <template #create-link="{ children }">
                <a class="link" @click.prevent="$router.push('/mods')">
                  <component :is="() => children" />
                </a>
              </template>
            </IntlFormatted>
          </span>
          <span v-else class="text">{{ formatMessage(messages.noProjectsLabel) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import {
  CalendarIcon,
  EditIcon,
  XIcon,
  SaveIcon,
  UploadIcon,
  TrashIcon,
  LinkIcon,
  LockIcon,
  GridIcon,
  ImageIcon,
  ListIcon,
  UpdatedIcon,
  BoxIcon,
  CollectionIcon,
  MoreVerticalIcon,
  ClipboardCopyIcon,
  ShareIcon,
} from "@modrinth/assets";
import {
  PopoutMenu,
  FileInput,
  DropdownSelect,
  Avatar,
  Button,
  commonMessages,
  ConfirmModal,
  ShareModal,
} from "@modrinth/ui";

import { isAdmin } from "@modrinth/utils";
import WorldIcon from "assets/images/utils/world.svg";
import UpToDate from "assets/images/illustrations/up_to_date.svg";
import ContentPageHeader from "@modrinth/ui/src/components/base/ContentPageHeader.vue";
import SimpleBadge from "@modrinth/ui/src/components/base/SimpleBadge.vue";
import ButtonStyled from "@modrinth/ui/src/components/base/ButtonStyled.vue";
import OverflowMenu from "@modrinth/ui/src/components/base/OverflowMenu.vue";
import NewModal from "@modrinth/ui/src/components/modal/NewModal.vue";
import { addNotification } from "~/composables/notifs.js";
import ProjectCard from "~/components/ui/ProjectCard.vue";
import AdPlaceholder from "~/components/ui/AdPlaceholder.vue";
import NavTabs from "~/components/ui/NavTabs.vue";

const vintl = useVIntl();
const { formatMessage } = vintl;
const formatRelativeTime = useRelativeTime();
const formatCompactNumber = useCompactNumber();
const editCollectionModal = ref();

const messages = defineMessages({
  collectionDescription: {
    id: "collection.description",
    defaultMessage: "{description} - View the collection {name} by {username} on Modrinth",
  },
  collectionLabel: {
    id: "collection.label.collection",
    defaultMessage: "Collection",
  },
  collectionTitle: {
    id: "collection.title",
    defaultMessage: "{name} - Collection",
  },
  editIconButton: {
    id: "collection.button.edit-icon",
    defaultMessage: "Edit icon",
  },
  deleteIconButton: {
    id: "collection.button.delete-icon",
    defaultMessage: "Delete icon",
  },
  createdAtLabel: {
    id: "collection.label.created-at",
    defaultMessage: "Created {ago}",
  },
  collectionNotFoundError: {
    id: "collection.error.not-found",
    defaultMessage: "Collection not found",
  },
  curatedByLabel: {
    id: "collection.label.curated-by",
    defaultMessage: "Curated by",
  },
  deleteModalDescription: {
    id: "collection.delete-modal.description",
    defaultMessage: "This will remove this collection forever. This action cannot be undone.",
  },
  deleteModalTitle: {
    id: "collection.delete-modal.title",
    defaultMessage: "Are you sure you want to delete this collection?",
  },
  followingCollectionDescription: {
    id: "collection.description.following",
    defaultMessage: "Auto-generated collection of all the projects you're following.",
  },
  noProjectsLabel: {
    id: "collection.label.no-projects",
    defaultMessage: "This collection has no projects!",
  },
  noProjectsAuthLabel: {
    id: "collection.label.no-projects-auth",
    defaultMessage:
      "You don't have any projects.\nWould you like to <create-link>add one</create-link>?",
  },
  ownerLabel: {
    id: "collection.label.owner",
    defaultMessage: "Owner",
  },
  projectsCountLabel: {
    id: "collection.label.projects-count",
    defaultMessage:
      "{count, plural, one {<stat>{count}</stat> project} other {<stat>{count}</stat> projects}}",
  },
  removeProjectButton: {
    id: "collection.button.remove-project",
    defaultMessage: "Remove project",
  },
  unfollowProjectButton: {
    id: "collection.button.unfollow-project",
    defaultMessage: "Unfollow project",
  },
  updatedAtLabel: {
    id: "collection.label.updated-at",
    defaultMessage: "Updated {ago}",
  },
  uploadIconButton: {
    id: "collection.button.upload-icon",
    defaultMessage: "Upload icon",
  },
});

const data = useNuxtApp();
const route = useNativeRoute();
const auth = await useAuth();
const cosmetics = useCosmetics();
const tags = useTags();
const flags = useFeatureFlags();

const isEditing = ref(false);

function cycleSearchDisplayMode() {
  cosmetics.value.searchDisplayMode.collection = data.$cycleValue(
    cosmetics.value.searchDisplayMode.collection,
    tags.value.projectViewModes,
  );
}

let collection, refreshCollection, creator, projects, refreshProjects;

try {
  if (route.params.id === "following") {
    collection = ref({
      id: "following",
      icon_url: "https://cdn.modrinth.com/follow-collection.png",
      name: formatMessage(commonMessages.followedProjectsLabel),
      description: formatMessage(messages.followingCollectionDescription),
      status: "private",
      user: auth.value.user.id,
      created: auth.value.user.created,
      updated: auth.value.user.created,
    });
    [{ data: projects, refresh: refreshProjects }] = await Promise.all([
      useAsyncData(
        `user/${auth.value.user.id}/follows`,
        () => useBaseFetch(`user/${auth.value.user.id}/follows`),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders);
            }

            return projects;
          },
        },
      ),
    ]);
    creator = ref(auth.value.user);
    refreshCollection = async () => {};
  } else {
    const val = await useAsyncData(`collection/${route.params.id}`, () =>
      useBaseFetch(`collection/${route.params.id}`, { apiVersion: 3 }),
    );
    collection = val.data;
    refreshCollection = val.refresh;
    [{ data: creator }, { data: projects, refresh: refreshProjects }] = await Promise.all([
      await useAsyncData(`user/${collection.value.user}`, () =>
        useBaseFetch(`user/${collection.value.user}`),
      ),
      await useAsyncData(
        `projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}]`,
        () =>
          useBaseFetch(
            `projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}`,
          ),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders);
            }

            return projects;
          },
        },
      ),
    ]);
  }
} catch (err) {
  console.error(err);
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.collectionNotFoundError),
  });
}

if (!collection.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.collectionNotFoundError),
  });
}

const title = computed(() =>
  formatMessage(messages.collectionTitle, { name: collection.value.name }),
);

useSeoMeta({
  title,
  description: () =>
    formatMessage(messages.collectionDescription, {
      name: collection.value.name,
      description: collection.value.description,
      username: creator.value.username,
    }),
  ogTitle: title,
  ogDescription: collection.value.description,
  ogImage: collection.value.icon_url ?? "https://cdn.modrinth.com/placeholder.png",
  robots: collection.value.status === "listed" ? "all" : "noindex",
});

const canEdit = computed(
  () =>
    auth.value.user &&
    (auth.value.user.id === collection.value.user || isAdmin(auth.value.user)) &&
    collection.value.id !== "following",
);

const projectTypes = computed(() => {
  const projectSet = new Set(
    projects.value?.map((project) => project?.project_type).filter((x) => x !== undefined) || [],
  );
  projectSet.delete("project");
  return Array.from(projectSet);
});

const navLinks = computed(() => [
  {
    label: formatMessage(commonMessages.allProjectType),
    href: `/collection/${collection.value.id}`,
  },
  ...projectTypes.value
    .map((x) => {
      return {
        label: formatMessage(getProjectTypeMessage(x, true)),
        href: `/collection/${collection.value.id}/${x}s`,
      };
    })
    .slice()
    .sort((a, b) => a.label.localeCompare(b.label)),
]);

const icon = ref(null);
const deletedIcon = ref(false);
const previewImage = ref(null);

const name = ref(collection.value.name);
const summary = ref(collection.value.description);
const visibility = ref(collection.value.status);
const removeProjects = ref([]);

async function unfollowProject(project) {
  await userFollowProject(project);
  projects.value = projects.value.filter((x) => x.id !== project.id);
}

async function saveChanges() {
  startLoading();
  try {
    if (deletedIcon.value) {
      await useBaseFetch(`collection/${collection.value.id}/icon`, {
        method: "DELETE",
        apiVersion: 3,
      });
    } else if (icon.value) {
      const ext = icon.value?.type?.split("/").pop();
      if (!ext) throw new Error("Invalid file type");
      await useBaseFetch(`collection/${collection.value.id}/icon?ext=${ext}`, {
        method: "PATCH",
        body: icon.value,
        apiVersion: 3,
      });
    }

    const projectsToRemove = removeProjects.value?.map((p) => p.id) ?? [];
    const newProjects = projects.value
      .filter((p) => !projectsToRemove.includes(p.id))
      .map((p) => p.id);
    const newProjectIds = projectsToRemove.length > 0 ? newProjects : undefined;

    await useBaseFetch(`collection/${collection.value.id}`, {
      method: "PATCH",
      body: {
        name: name.value,
        description: summary.value,
        status: visibility.value,
        new_projects: newProjectIds,
      },
      apiVersion: 3,
    });

    await refreshCollection();
    await refreshProjects();

    name.value = collection.value.name;
    summary.value = collection.value.description;
    visibility.value = collection.value.status;
    removeProjects.value = [];

    isEditing.value = false;
  } catch (err) {
    addNotification({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err,
      type: "error",
    });
  }
  await initUserCollections();
  stopLoading();
}

async function deleteCollection() {
  startLoading();
  try {
    await useBaseFetch(`collection/${collection.value.id}`, {
      method: "DELETE",
      apiVersion: 3,
    });
    if (auth.value.user.id === collection.value.user) {
      await navigateTo("/dashboard/collections");
    } else {
      await navigateTo(`/user/${collection.value.user}/collections`);
    }
  } catch (err) {
    addNotification({
      group: "main",
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data.description,
      type: "error",
    });
  }
  await initUserCollections();
  stopLoading();
}

function showPreviewImage(files) {
  const reader = new FileReader();
  icon.value = files[0];
  deletedIcon.value = false;
  reader.readAsDataURL(icon.value);
  reader.onload = (event) => {
    previewImage.value = event.target.result;
  };
}

async function copyId() {
  await navigator.clipboard.writeText(collection.value.id);
}
</script>

<style scoped lang="scss">
.animated-dropdown {
  // Omorphia's dropdowns are harcoded in width, so we need to override that
  width: 100% !important;
}

.short > * {
  height: fit-content;
}

.inputs {
  margin-bottom: 1rem;

  input {
    margin-top: 0.5rem;
    width: 100%;
  }

  textarea {
    min-height: 10rem;
  }

  label {
    margin-bottom: 0;
  }
}

.team-member {
  align-items: center;
  padding: 0.25rem 0.5rem;

  .member-info {
    overflow: hidden;
    margin: auto 0 auto 0.75rem;

    .name {
      font-weight: bold;
    }

    p {
      font-size: var(--font-size-sm);
      margin: 0.2rem 0;
    }
  }
}

.remove-btn {
  margin-top: auto;
}

.card {
  padding: var(--spacing-card-lg);

  .page-header__icon {
    margin-block: 0;
  }

  .card__overlay {
    top: var(--spacing-card-lg);
    right: var(--spacing-card-lg);
  }
}

.collection-info {
  display: grid;
  grid-template-columns: 1fr;
}

.date {
  color: var(--color-text-secondary);
  font-size: var(--font-size-nm);
  display: flex;
  align-items: center;
  margin-bottom: 0.25rem;
  cursor: default;

  .label {
    margin-right: 0.25rem;
  }

  svg {
    height: 1rem;
    margin-right: 0.25rem;
  }
}

.card-header {
  font-size: 1.125rem;
  font-weight: bold;
  color: var(--color-heading);
  margin-bottom: 0.5rem;
  width: fit-content;
}

.title {
  margin: var(--gap-md) 0 var(--spacing-card-xs) 0;
  font-size: var(--font-size-xl);
  color: var(--color-text-dark);
}

.collection-label {
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.collection-description {
  margin-top: var(--spacing-card-sm);
  margin-bottom: 0;
}
</style>
