<template>
  <div v-if="user">
    <ModalCreation ref="modal_creation" />
    <ModalReport ref="modal_report" :item-id="user.id" item-type="user" />
    <div class="user-header-wrapper">
      <div class="user-header">
        <Avatar
          :src="previewImage ? previewImage : user.avatar_url"
          size="md"
          circle
          :alt="user.username"
        />
        <h1 class="username">
          {{ user.username }}
        </h1>
      </div>
    </div>
    <div class="normal-page">
      <div class="normal-page__sidebar">
        <div class="card sidebar">
          <h1 class="mobile-username">
            {{ user.username }}
          </h1>
          <div class="card__overlay">
            <FileInput
              v-if="isEditing"
              :max-size="262144"
              :show-icon="true"
              :prompt="formatMessage(messages.profileUploadAvatarInput)"
              accept="image/png,image/jpeg,image/gif,image/webp"
              class="choose-image iconified-button"
              @change="showPreviewImage"
            >
              <UploadIcon />
            </FileInput>
            <button
              v-else-if="auth.user && auth.user.id === user.id"
              class="iconified-button"
              @click="isEditing = true"
            >
              <EditIcon />
              {{ formatMessage(commonMessages.editButton) }}
            </button>
            <button
              v-else-if="auth.user"
              class="iconified-button"
              @click="$refs.modal_report.show()"
            >
              <ReportIcon aria-hidden="true" />
              {{ formatMessage(messages.profileReportButton) }}
            </button>
            <nuxt-link v-else class="iconified-button" to="/auth/sign-in">
              <ReportIcon aria-hidden="true" />
              {{ formatMessage(messages.profileReportButton) }}
            </nuxt-link>
          </div>
          <template v-if="isEditing">
            <div class="inputs universal-labels">
              <label for="user-username">
                <span class="label__title">
                  {{ formatMessage(messages.profileEditUsernameLabel) }}
                </span>
              </label>
              <input id="user-username" v-model="user.username" maxlength="39" type="text" />
              <label for="user-bio">
                <span class="label__title">
                  {{ formatMessage(messages.profileEditBioLabel) }}
                </span>
              </label>
              <div class="textarea-wrapper">
                <textarea id="user-bio" v-model="user.bio" maxlength="160" />
              </div>
            </div>
            <div class="button-group">
              <button
                class="iconified-button"
                @click="
                  () => {
                    isEditing = false
                    user = JSON.parse(JSON.stringify(auth.user))
                    previewImage = null
                    icon = null
                  }
                "
              >
                <CrossIcon /> {{ formatMessage(commonMessages.cancelButton) }}
              </button>
              <button class="iconified-button brand-button" @click="saveChanges">
                <SaveIcon /> {{ formatMessage(commonMessages.saveButton) }}
              </button>
            </div>
          </template>
          <template v-else>
            <div class="sidebar__item">
              <Badge v-if="tags.staffRoles.includes(user.role)" :type="user.role" />
              <Badge v-else-if="projects.length > 0" type="creator" />
            </div>
            <span v-if="user.bio" class="sidebar__item bio">{{ user.bio }}</span>
            <hr class="card-divider" />
            <div class="primary-stat">
              <DownloadIcon class="primary-stat__icon" aria-hidden="true" />
              <div class="primary-stat__text">
                <IntlFormatted
                  :message-id="messages.profileDownloadsStats"
                  :values="{ count: formatCompactNumber(sumDownloads) }"
                >
                  <template #stat="{ children }">
                    <span class="primary-stat__counter">
                      <component :is="() => normalizeChildren(children)" />
                    </span>
                  </template>
                </IntlFormatted>
              </div>
            </div>
            <div class="primary-stat">
              <HeartIcon class="primary-stat__icon" aria-hidden="true" />
              <div class="primary-stat__text">
                <IntlFormatted
                  :message-id="messages.profileProjectsFollowersStats"
                  :values="{ count: formatCompactNumber(sumFollows) }"
                >
                  <template #stat="{ children }">
                    <span class="primary-stat__counter">
                      <component :is="() => normalizeChildren(children)" />
                    </span>
                  </template>
                </IntlFormatted>
              </div>
            </div>
            <div class="stats-block__item secondary-stat">
              <SunriseIcon class="secondary-stat__icon" aria-hidden="true" />
              <span
                v-tooltip="
                  formatMessage(commonMessages.dateAtTimeTooltip, {
                    date: new Date(user.created),
                    time: new Date(user.created),
                  })
                "
                class="secondary-stat__text date"
              >
                {{
                  formatMessage(messages.profileJoinedAt, { ago: formatRelativeTime(user.created) })
                }}
              </span>
            </div>
            <hr class="card-divider" />
            <div class="stats-block__item secondary-stat">
              <UserIcon class="secondary-stat__icon" aria-hidden="true" />
              <span class="secondary-stat__text">
                <IntlFormatted :message-id="messages.profileUserId">
                  <template #~id>
                    <CopyCode :text="user.id" />
                  </template>
                </IntlFormatted>
              </span>
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
                href: `/user/${user.username}`,
              },
              ...projectTypes.map((x) => {
                return {
                  label: formatMessage(getProjectTypeMessage(x, true)),
                  href: `/user/${user.username}/${x}s`,
                }
              }),
            ]"
          />
          <div class="input-group">
            <NuxtLink
              v-if="auth.user && auth.user.id === user.id"
              class="iconified-button"
              to="/dashboard/projects"
            >
              <SettingsIcon />
              {{ formatMessage(messages.profileManageProjectsButton) }}
            </NuxtLink>
            <button
              v-tooltip="
                formatMessage(commonMessages[`${cosmetics.searchDisplayMode.user}InputView`])
              "
              :aria-label="
                formatMessage(commonMessages[`${cosmetics.searchDisplayMode.user}InputView`])
              "
              class="square-button"
              @click="cycleSearchDisplayMode()"
            >
              <GridIcon v-if="cosmetics.searchDisplayMode.user === 'grid'" />
              <ImageIcon v-else-if="cosmetics.searchDisplayMode.user === 'gallery'" />
              <ListIcon v-else />
            </button>
          </div>
        </nav>
        <div
          v-if="projects.length > 0"
          class="project-list"
          :class="'display-mode--' + cosmetics.searchDisplayMode.user"
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
            :id="project.slug || project.id"
            :key="project.id"
            :name="project.title"
            :display="cosmetics.searchDisplayMode.user"
            :featured-image="
              project.gallery
                .slice()
                .sort((a, b) => b.featured - a.featured)
                .map((x) => x.url)[0]
            "
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
        <div v-else class="error">
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
      </div>
    </div>
  </div>
</template>
<script setup>
import { Promotion } from 'omorphia'
import ProjectCard from '~/components/ui/ProjectCard.vue'
import Badge from '~/components/ui/Badge.vue'

import ReportIcon from '~/assets/images/utils/report.svg'
import SunriseIcon from '~/assets/images/utils/sunrise.svg'
import DownloadIcon from '~/assets/images/utils/download.svg'
import SettingsIcon from '~/assets/images/utils/settings.svg'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg'
import UserIcon from '~/assets/images/utils/user.svg'
import EditIcon from '~/assets/images/utils/edit.svg'
import HeartIcon from '~/assets/images/utils/heart.svg'
import CrossIcon from '~/assets/images/utils/x.svg'
import SaveIcon from '~/assets/images/utils/save.svg'
import GridIcon from '~/assets/images/utils/grid.svg'
import ListIcon from '~/assets/images/utils/list.svg'
import ImageIcon from '~/assets/images/utils/image.svg'
import UploadIcon from '~/assets/images/utils/upload.svg'
import FileInput from '~/components/ui/FileInput.vue'
import ModalReport from '~/components/ui/ModalReport.vue'
import ModalCreation from '~/components/ui/ModalCreation.vue'
import NavRow from '~/components/ui/NavRow.vue'
import CopyCode from '~/components/ui/CopyCode.vue'
import Avatar from '~/components/ui/Avatar.vue'

const data = useNuxtApp()
const route = useRoute()
const auth = await useAuth()
const cosmetics = useCosmetics()
const tags = useTags()

const vintl = useVIntl()
const { formatMessage } = vintl

const formatCompactNumber = useCompactNumber()

const formatRelativeTime = useRelativeTime()

const messages = defineMessages({
  profileDownloadsStats: {
    id: 'profile.stats.downloads',
    defaultMessage:
      '{count, plural, one {<stat>{count}</stat> download} other {<stat>{count}</stat> downloads}}',
  },
  profileProjectsFollowersStats: {
    id: 'profile.stats.projects-followers',
    defaultMessage:
      '{count, plural, one {<stat>{count}</stat> follower} other {<stat>{count}</stat> followers}} of projects',
  },
  profileJoinedAt: {
    id: 'profile.joined-at',
    defaultMessage: 'Joined {ago}',
  },
  profileUserId: {
    id: 'profile.user-id',
    defaultMessage: 'User ID: {id}',
  },
  profileManageProjectsButton: {
    id: 'profile.button.manage-projects',
    defaultMessage: 'Manage projects',
  },
  profileMetaDescription: {
    id: 'profile.meta.description',
    defaultMessage: "Download {username}'s projects on Modrinth",
  },
  profileMetaDescriptionWithBio: {
    id: 'profile.meta.description-with-bio',
    defaultMessage: "{bio} - Download {username}'s projects on Modrinth",
  },
  profileReportButton: {
    id: 'profile.button.report',
    defaultMessage: 'Report',
  },
  profileUploadAvatarInput: {
    id: 'profile.input.upload-avatar',
    defaultMessage: 'Upload avatar',
  },
  profileEditUsernameLabel: {
    id: 'profile.label.edit-username',
    defaultMessage: 'Username',
  },
  profileEditBioLabel: {
    id: 'profile.label.edit-bio',
    defaultMessage: 'Bio',
  },
  profileNoProjectsLabel: {
    id: 'profile.label.no-projects',
    defaultMessage: 'This user has no projects!',
  },
  profileNoProjectsAuthLabel: {
    id: 'profile.label.no-projects-auth',
    defaultMessage:
      "You don't have any projects.\nWould you like to <create-link>create one</create-link>?",
  },
  userNotFoundError: {
    id: 'profile.error.not-found',
    defaultMessage: 'User not found',
  },
})

let user, projects
try {
  ;[{ data: user }, { data: projects }] = await Promise.all([
    useAsyncData(`user/${route.params.id}`, () => useBaseFetch(`user/${route.params.id}`)),
    useAsyncData(
      `user/${route.params.id}/projects`,
      () => useBaseFetch(`user/${route.params.id}/projects`),
      {
        transform: (projects) => {
          for (const project of projects) {
            project.categories = project.categories.concat(project.loaders)
            project.project_type = data.$getProjectTypeForUrl(
              project.project_type,
              project.categories,
              tags.value
            )
          }

          return projects
        },
      }
    ),
  ])
} catch {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  })
}

if (!user.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: formatMessage(messages.userNotFoundError),
  })
}

if (user.value.username !== route.params.id) {
  await navigateTo(`/user/${user.value.username}`, { redirectCode: 301 })
}

const title = `${user.value.username} - Modrinth`
const description = ref(
  user.value.bio
    ? `${formatMessage(messages.profileMetaDescriptionWithBio, {
        bio: user.value.bio,
        username: user.value.username,
      })}`
    : `${formatMessage(messages.profileMetaDescription, { username: user.value.username })}`
)

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
  ogImage: user.value.avatar_url ?? 'https://cdn.modrinth.com/placeholder.png',
})

const projectTypes = computed(() => {
  const obj = {}

  for (const project of projects.value) {
    obj[project.project_type] = true
  }

  return Object.keys(obj)
})
const sumDownloads = computed(() => {
  let sum = 0

  for (const project of projects.value) {
    sum += project.downloads
  }

  return sum
})
const sumFollows = computed(() => {
  let sum = 0

  for (const project of projects.value) {
    sum += project.followers
  }

  return sum
})

const isEditing = ref(false)
const icon = shallowRef(null)
const previewImage = shallowRef(null)

function showPreviewImage(files) {
  const reader = new FileReader()
  icon.value = files[0]
  reader.readAsDataURL(icon.value)
  reader.onload = (event) => {
    previewImage.value = event.target.result
  }
}

async function saveChanges() {
  startLoading()
  try {
    if (icon.value) {
      await useBaseFetch(
        `user/${auth.value.user.id}/icon?ext=${
          icon.value.type.split('/')[icon.value.type.split('/').length - 1]
        }`,
        {
          method: 'PATCH',
          body: icon.value,
        }
      )
    }

    const reqData = {
      email: user.value.email,
      bio: user.value.bio,
    }
    if (user.value.username !== auth.value.user.username) {
      reqData.username = user.value.username
    }

    await useBaseFetch(`user/${auth.value.user.id}`, {
      method: 'PATCH',
      body: reqData,
    })
    await useAuth(auth.value.token)

    isEditing.value = false
  } catch (err) {
    console.error(err)
    data.$notify({
      group: 'main',
      title: formatMessage(commonMessages.errorNotificationTitle),
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

function cycleSearchDisplayMode() {
  cosmetics.value.searchDisplayMode.user = data.$cycleValue(
    cosmetics.value.searchDisplayMode.user,
    tags.value.projectViewModes
  )
  saveCosmetics()
}
</script>
<script>
export default defineNuxtComponent({
  methods: {},
})
</script>

<style lang="scss" scoped>
.user-header-wrapper {
  display: flex;
  margin: 0 auto -1.5rem;
  max-width: 80rem;

  .user-header {
    position: relative;
    z-index: 4;
    display: flex;
    width: 100%;
    padding: 0 1rem;
    gap: 1rem;
    align-items: center;

    .username {
      display: none;
      font-size: 2rem;
      margin-bottom: 2.5rem;
    }
  }
}

.mobile-username {
  margin: 0.25rem 0;
}

@media screen and (min-width: 501px) {
  .mobile-username {
    display: none;
  }

  .user-header-wrapper .user-header .username {
    display: block;
  }
}

.sidebar {
  padding-top: 2.5rem;
}

.sidebar__item:not(:last-child) {
  margin: 0 0 0.75rem 0;
}

.profile-picture {
  border-radius: var(--size-rounded-lg);
  height: 8rem;
  width: 8rem;
}

.username {
  font-size: var(--font-size-xl);
}

.bio {
  display: block;
  overflow-wrap: break-word;
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

.date {
  cursor: default;
}

.inputs {
  margin-bottom: 1rem;

  input {
    margin-top: 0.5rem;
    width: 100%;
  }

  label {
    margin-bottom: 0;
  }
}

.textarea-wrapper {
  height: 10rem;
}

@media (max-width: 400px) {
  .sidebar {
    padding-top: 3rem;
  }
}
</style>
