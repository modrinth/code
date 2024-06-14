<template>
  <div>
    <ModalConfirm
      v-if="auth.user && auth.user.id === creator.id"
      ref="deleteModal"
      :title="formatMessage(messages.deleteModalTitle)"
      :description="formatMessage(messages.deleteModalDescription)"
      :has-to-type="false"
      :proceed-label="formatMessage(commonMessages.deleteLabel)"
      @proceed="deleteCollection()"
    />
    <div class="normal-page">
      <div class="normal-page__sidebar">
        <div class="card">
          <div class="card__overlay input-group">
            <template v-if="canEdit && isEditing === false">
              <Button @click="isEditing = true">
                <EditIcon />
                {{ formatMessage(commonMessages.editButton) }}
              </Button>
              <Button id="delete-collection" @click="() => $refs.deleteModal.show()">
                <TrashIcon />
                {{ formatMessage(commonMessages.deleteLabel) }}
              </Button>
            </template>
            <template v-else-if="canEdit && isEditing === true">
              <PopoutMenu class="btn" position="bottom" direction="right">
                <EditIcon /> {{ formatMessage(messages.editIconButton) }}
                <template #menu>
                  <span class="icon-edit-menu">
                    <FileInput
                      id="project-icon"
                      :max-size="262144"
                      :show-icon="true"
                      accept="image/png,image/jpeg,image/gif,image/webp"
                      class="btn btn-transparent upload"
                      style="white-space: nowrap"
                      prompt=""
                      @change="showPreviewImage"
                    >
                      <UploadIcon />
                      {{ formatMessage(messages.uploadIconButton) }}
                    </FileInput>
                    <Button
                      v-if="!deletedIcon && (previewImage || collection.icon_url)"
                      style="white-space: nowrap"
                      transparent
                      @click="
                        () => {
                          deletedIcon = true
                          previewImage = null
                        }
                      "
                    >
                      <TrashIcon />
                      {{ formatMessage(messages.deleteIconButton) }}
                    </Button>
                  </span>
                </template>
              </PopoutMenu>
            </template>
          </div>
          <!-- Editing -->
          <template v-if="isEditing">
            <div class="inputs universal-labels">
              <div class="avatar-section">
                <Avatar
                  size="md"
                  :src="deletedIcon ? null : previewImage ? previewImage : collection.icon_url"
                />
              </div>
              <label for="collection-title">
                <span class="label__title"> {{ formatMessage(commonMessages.titleLabel) }} </span>
              </label>
              <input id="collection-title" v-model="name" maxlength="255" type="text" />
              <label for="collection-description">
                <span class="label__title">
                  {{ formatMessage(commonMessages.descriptionLabel) }}
                </span>
              </label>
              <div class="textarea-wrapper">
                <textarea id="collection-description" v-model="summary" maxlength="255" />
              </div>
              <label for="visibility">
                <span class="label__title">
                  {{ formatMessage(commonMessages.visibilityLabel) }}
                </span>
              </label>
              <DropdownSelect
                id="visibility"
                v-model="visibility"
                :options="['listed', 'unlisted', 'private']"
                :disabled="visibility === 'rejected'"
                :multiple="false"
                :display-name="
                  (s) => {
                    if (s === 'listed') return formatMessage(commonMessages.publicLabel)
                    return formatMessage(commonMessages[`${s}Label`])
                  }
                "
                :searchable="false"
              />
            </div>
            <div class="push-right input-group">
              <Button @click="isEditing = false">
                <XIcon />
                {{ formatMessage(commonMessages.cancelButton) }}
              </Button>
              <Button color="primary" @click="saveChanges()">
                <SaveIcon />
                {{ formatMessage(commonMessages.saveButton) }}
              </Button>
            </div>
          </template>
          <!-- Content -->
          <template v-if="!isEditing">
            <div class="page-header__icon">
              <Avatar size="md" :src="collection.icon_url" />
            </div>
            <div class="page-header__text">
              <h1 class="title">{{ collection.name }}</h1>

              <div>
                <span class="collection-label">
                  <BoxIcon /> {{ formatMessage(messages.collectionLabel) }}
                </span>
              </div>

              <div class="collection-info">
                <div class="metadata-item markdown-body collection-description">
                  <p>{{ collection.description }}</p>
                </div>

                <hr class="card-divider" />

                <div v-if="canEdit" class="primary-stat">
                  <template v-if="collection.status === 'listed'">
                    <WorldIcon class="primary-stat__icon" aria-hidden="true" />
                    <div class="primary-stat__text">
                      <strong> {{ formatMessage(commonMessages.publicLabel) }} </strong>
                    </div>
                  </template>
                  <template v-else-if="collection.status === 'unlisted'">
                    <LinkIcon class="primary-stat__icon" aria-hidden="true" />
                    <div class="primary-stat__text">
                      <strong> {{ formatMessage(commonMessages.unlistedLabel) }} </strong>
                    </div>
                  </template>
                  <template v-else-if="collection.status === 'private'">
                    <LockIcon class="primary-stat__icon" aria-hidden="true" />
                    <div class="primary-stat__text">
                      <strong> {{ formatMessage(commonMessages.privateLabel) }} </strong>
                    </div>
                  </template>
                  <template v-else-if="collection.status === 'rejected'">
                    <XIcon class="primary-stat__icon" aria-hidden="true" />
                    <div class="primary-stat__text">
                      <strong> {{ formatMessage(commonMessages.rejectedLabel) }} </strong>
                    </div>
                  </template>
                </div>
              </div>

              <div class="primary-stat">
                <LibraryIcon class="primary-stat__icon" aria-hidden="true" />
                <div v-if="projects" class="primary-stat__text">
                  <IntlFormatted
                    :message-id="messages.projectsCountLabel"
                    :values="{ count: formatCompactNumber(projects.length || 0) }"
                  >
                    <template #stat="{ children }">
                      <span class="primary-stat__counter">
                        <component :is="() => normalizeChildren(children)" />
                      </span>
                    </template>
                  </IntlFormatted>
                </div>
              </div>

              <div class="metadata-item">
                <div
                  v-tooltip="
                    formatMessage(commonMessages.dateAtTimeTooltip, {
                      date: new Date(collection.created),
                      time: new Date(collection.created),
                    })
                  "
                  class="date"
                >
                  <CalendarIcon />
                  <label>
                    {{
                      formatMessage(messages.createdAtLabel, {
                        ago: formatRelativeTime(collection.created),
                      })
                    }}
                  </label>
                </div>
              </div>

              <div v-if="collection.id !== 'following'" class="metadata-item">
                <div
                  v-tooltip="
                    formatMessage(commonMessages.dateAtTimeTooltip, {
                      date: new Date(collection.updated),
                      time: new Date(collection.updated),
                    })
                  "
                  class="date"
                >
                  <UpdatedIcon />
                  <label>
                    {{
                      formatMessage(messages.updatedAtLabel, {
                        ago: formatRelativeTime(collection.updated),
                      })
                    }}
                  </label>
                </div>
              </div>
            </div>

            <hr class="card-divider" />

            <div class="collection-info">
              <h2 class="card-header">{{ formatMessage(messages.curatedByLabel) }}</h2>
              <div class="metadata-item">
                <nuxt-link
                  class="team-member columns button-transparent"
                  :to="'/user/' + creator.username"
                >
                  <Avatar :src="creator.avatar_url" :alt="creator.username" size="sm" circle />

                  <div class="member-info">
                    <p class="name">{{ creator.username }}</p>
                    <p class="role">{{ formatMessage(messages.ownerLabel) }}</p>
                  </div>
                </nuxt-link>
              </div>
              <!-- <hr class="card-divider" />
            <div class="input-group">
              <Button @click="() => $refs.shareModal.show()">
                <ShareIcon />
                Share
              </Button>
            </div> -->
            </div>
          </template>
        </div>
      </div>
      <div class="normal-page__content">
        <Promotion />

        <nav class="navigation-card">
          <NavRow
            :links="[
              {
                label: formatMessage(commonMessages.allProjectType),
                href: `/collection/${collection.id}`,
              },
              ...projectTypes.map((x) => {
                return {
                  label: formatMessage(getProjectTypeMessage(x, true)),
                  href: `/collection/${collection.id}/${x}s`,
                }
              }),
            ]"
          />
          <button
            v-tooltip="
              formatMessage(
                commonMessages[`${cosmetics.searchDisplayMode.collection || 'list'}InputView`]
              )
            "
            :aria-label="
              formatMessage(
                commonMessages[`${cosmetics.searchDisplayMode.collection || 'list'}InputView`]
              )
            "
            class="square-button"
            @click="cycleSearchDisplayMode()"
          >
            <GridIcon v-if="cosmetics.searchDisplayMode.collection === 'grid'" />
            <ImageIcon v-else-if="cosmetics.searchDisplayMode.collection === 'gallery'" />
            <ListIcon v-else />
          </button>
        </nav>

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
                    route.params.projectType.substr(0, route.params.projectType.length - 1)
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
                  removeProjects = [project]
                  saveChanges()
                }
              "
            >
              <TrashIcon />
              {{ formatMessage(messages.removeProjectButton) }}
            </button>
            <button
              v-if="collection.id === 'following'"
              class="iconified-button"
              @click="unfollowProject(project)"
            >
              <TrashIcon />
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
  Avatar,
  Button,
  CalendarIcon,
  Promotion,
  EditIcon,
  XIcon,
  SaveIcon,
  UploadIcon,
  TrashIcon,
  PopoutMenu,
  FileInput,
  DropdownSelect,
  LinkIcon,
  LockIcon,
  GridIcon,
  ImageIcon,
  ListIcon,
  UpdatedIcon,
  LibraryIcon,
  BoxIcon,
} from 'omorphia'

import WorldIcon from 'assets/images/utils/world.svg'
import UpToDate from 'assets/images/illustrations/up_to_date.svg'
import { addNotification } from '~/composables/notifs.js'
import ModalConfirm from '~/components/ui/ModalConfirm.vue'
import NavRow from '~/components/ui/NavRow.vue'
import ProjectCard from '~/components/ui/ProjectCard.vue'

const vintl = useVIntl()
const { formatMessage } = vintl
const formatRelativeTime = useRelativeTime()
const formatCompactNumber = useCompactNumber()

const messages = defineMessages({
  collectionDescription: {
    id: 'collection.description',
    defaultMessage: '{description} - View the collection {name} by {username} on Modrinth',
  },
  collectionLabel: {
    id: 'collection.label.collection',
    defaultMessage: 'Collection',
  },
  collectionTitle: {
    id: 'collection.title',
    defaultMessage: '{name} - Collection',
  },
  editIconButton: {
    id: 'collection.button.edit-icon',
    defaultMessage: 'Edit icon',
  },
  deleteIconButton: {
    id: 'collection.button.delete-icon',
    defaultMessage: 'Delete icon',
  },
  createdAtLabel: {
    id: 'collection.label.created-at',
    defaultMessage: 'Created {ago}',
  },
  collectionNotFoundError: {
    id: 'collection.error.not-found',
    defaultMessage: 'Collection not found',
  },
  curatedByLabel: {
    id: 'collection.label.curated-by',
    defaultMessage: 'Curated by',
  },
  deleteModalDescription: {
    id: 'collection.delete-modal.description',
    defaultMessage: 'This will remove this collection forever. This action cannot be undone.',
  },
  deleteModalTitle: {
    id: 'collection.delete-modal.title',
    defaultMessage: 'Are you sure you want to delete this collection?',
  },
  followingCollectionDescription: {
    id: 'collection.description.following',
    defaultMessage: "Auto-generated collection of all the projects you're following.",
  },
  noProjectsLabel: {
    id: 'collection.label.no-projects',
    defaultMessage: 'This collection has no projects!',
  },
  noProjectsAuthLabel: {
    id: 'collection.label.no-projects-auth',
    defaultMessage:
      "You don't have any projects.\nWould you like to <create-link>add one</create-link>?",
  },
  ownerLabel: {
    id: 'collection.label.owner',
    defaultMessage: 'Owner',
  },
  projectsCountLabel: {
    id: 'collection.label.projects-count',
    defaultMessage:
      '{count, plural, one {<stat>{count}</stat> project} other {<stat>{count}</stat> projects}}',
  },
  removeProjectButton: {
    id: 'collection.button.remove-project',
    defaultMessage: 'Remove project',
  },
  unfollowProjectButton: {
    id: 'collection.button.unfollow-project',
    defaultMessage: 'Unfollow project',
  },
  updatedAtLabel: {
    id: 'collection.label.updated-at',
    defaultMessage: 'Updated {ago}',
  },
  uploadIconButton: {
    id: 'collection.button.upload-icon',
    defaultMessage: 'Upload icon',
  },
})

const data = useNuxtApp()
const route = useNativeRoute()
const auth = await useAuth()
const cosmetics = useCosmetics()
const tags = useTags()

const isEditing = ref(false)

function cycleSearchDisplayMode() {
  cosmetics.value.searchDisplayMode.collection = data.$cycleValue(
    cosmetics.value.searchDisplayMode.collection,
    tags.value.projectViewModes
  )
  saveCosmetics()
}

let collection, refreshCollection, creator, projects, refreshProjects

try {
  if (route.params.id === 'following') {
    collection = ref({
      id: 'following',
      icon_url: 'https://cdn.modrinth.com/follow-collection.png',
      name: formatMessage(commonMessages.followedProjectsLabel),
      description: formatMessage(messages.followingCollectionDescription),
      status: 'private',
      user: auth.value.user.id,
      created: auth.value.user.created,
      updated: auth.value.user.created,
    })
    ;[{ data: projects, refresh: refreshProjects }] = await Promise.all([
      useAsyncData(
        `user/${auth.value.user.id}/follows`,
        () => useBaseFetch(`user/${auth.value.user.id}/follows`),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders)
            }

            return projects
          },
        }
      ),
    ])
    creator = ref(auth.value.user)
    refreshCollection = async () => {}
  } else {
    const val = await useAsyncData(`collection/${route.params.id}`, () =>
      useBaseFetch(`collection/${route.params.id}`, { apiVersion: 3 })
    )
    collection = val.data
    refreshCollection = val.refresh
    ;[{ data: creator }, { data: projects, refresh: refreshProjects }] = await Promise.all([
      await useAsyncData(`user/${collection.value.user}`, () =>
        useBaseFetch(`user/${collection.value.user}`)
      ),
      await useAsyncData(
        `projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}]`,
        () =>
          useBaseFetch(
            `projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}`
          ),
        {
          transform: (projects) => {
            for (const project of projects) {
              project.categories = project.categories.concat(project.loaders)
            }

            return projects
          },
        }
      ),
    ])
  }
} catch (err) {
  console.error(err)
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.collectionNotFoundError),
  })
}

if (!collection.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.collectionNotFoundError),
  })
}

const title = computed(() =>
  formatMessage(messages.collectionTitle, { name: collection.value.name })
)

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
  ogImage: collection.value.icon_url ?? 'https://cdn.modrinth.com/placeholder.png',
  robots: collection.value.status === 'listed' ? 'all' : 'noindex',
})

const canEdit = computed(
  () =>
    auth.value.user &&
    auth.value.user.id === collection.value.user &&
    collection.value.id !== 'following'
)

const projectTypes = computed(() => {
  const projectSet = new Set(
    projects.value?.map((project) => project?.project_type).filter((x) => x !== undefined) || []
  )
  projectSet.delete('project')
  return Array.from(projectSet)
})

const icon = ref(null)
const deletedIcon = ref(false)
const previewImage = ref(null)

const name = ref(collection.value.name)
const summary = ref(collection.value.description)
const visibility = ref(collection.value.status)
const removeProjects = ref([])

async function unfollowProject(project) {
  await userUnfollowProject(project)
  projects.value = projects.value.filter((x) => x.id !== project.id)
}

async function saveChanges() {
  startLoading()
  try {
    if (deletedIcon.value) {
      await useBaseFetch(`collection/${collection.value.id}/icon`, {
        method: 'DELETE',
        apiVersion: 3,
      })
    } else if (icon.value) {
      const ext = icon.value?.type?.split('/').pop()
      if (!ext) throw new Error('Invalid file type')
      await useBaseFetch(`collection/${collection.value.id}/icon?ext=${ext}`, {
        method: 'PATCH',
        body: icon.value,
        apiVersion: 3,
      })
    }

    const projectsToRemove = removeProjects.value?.map((p) => p.id) ?? []
    const newProjects = projects.value
      .filter((p) => !projectsToRemove.includes(p.id))
      .map((p) => p.id)
    const newProjectIds = projectsToRemove.length > 0 ? newProjects : undefined

    await useBaseFetch(`collection/${collection.value.id}`, {
      method: 'PATCH',
      body: {
        name: name.value,
        description: summary.value,
        status: visibility.value,
        new_projects: newProjectIds,
      },
      apiVersion: 3,
    })

    await refreshCollection()
    await refreshProjects()

    name.value = collection.value.name
    summary.value = collection.value.description
    visibility.value = collection.value.status
    removeProjects.value = []

    isEditing.value = false
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err,
      type: 'error',
    })
  }
  await initUserCollections()
  stopLoading()
}

async function deleteCollection() {
  startLoading()
  try {
    await useBaseFetch(`collection/${collection.value.id}`, {
      method: 'DELETE',
      apiVersion: 3,
    })
    await navigateTo('/dashboard/collections')
  } catch (err) {
    addNotification({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data.description,
      type: 'error',
    })
  }
  await initUserCollections()
  stopLoading()
}

function showPreviewImage(files) {
  const reader = new FileReader()
  icon.value = files[0]
  deletedIcon.value = false
  reader.readAsDataURL(icon.value)
  reader.onload = (event) => {
    previewImage.value = event.target.result
  }
}
</script>

<style scoped lang="scss">
.animated-dropdown {
  // Omorphia's dropdowns are harcoded in width, so we need to override that
  width: 100% !important;
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
