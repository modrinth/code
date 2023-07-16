<template>
  <div v-if="user">
    <Head>
      <Title>{{ user.username + ' - Modrinth' }}</Title>
      <Meta name="og:title" :content="user.username" />
      <Meta name="description" :content="metaDescription" />
      <Meta name="og:type" content="website" />
      <Meta name="apple-mobile-web-app-title" :content="metaDescription" />
      <Meta name="og:description" :content="metaDescription" />
      <Meta
        name="og:image"
        :content="user.avatar_url ? user.avatar_url : 'https://cdn.modrinth.com/placeholder.png'"
      />
    </Head>
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
              accept="image/png,image/jpeg,image/gif,image/webp"
              class="choose-image iconified-button"
              prompt="Upload avatar"
              @change="showPreviewImage"
            >
              <UploadIcon />
            </FileInput>
            <button
              v-else-if="$auth.user && $auth.user.id === user.id"
              class="iconified-button"
              @click="isEditing = true"
            >
              <EditIcon />
              Edit
            </button>
            <button
              v-else-if="$auth.user"
              class="iconified-button"
              @click="$refs.modal_report.show()"
            >
              <ReportIcon aria-hidden="true" />
              Report
            </button>
            <a v-else class="iconified-button" :href="getAuthUrl()" rel="noopener nofollow">
              <ReportIcon aria-hidden="true" />
              Report
            </a>
          </div>
          <template v-if="isEditing">
            <div class="inputs universal-labels">
              <label for="user-username"><span class="label__title">Username</span></label>
              <input id="user-username" v-model="user.username" maxlength="39" type="text" />
              <label for="user-bio"><span class="label__title">Bio</span></label>
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
                    user = JSON.parse(JSON.stringify($auth.user))
                    previewImage = null
                    icon = null
                  }
                "
              >
                <CrossIcon /> Cancel
              </button>
              <button class="iconified-button brand-button" @click="saveChanges">
                <SaveIcon /> Save
              </button>
            </div>
          </template>
          <template v-else>
            <div class="sidebar__item">
              <Badge v-if="$tag.staffRoles.includes(user.role)" :type="user.role" />
              <Badge v-else-if="projects.length > 0" type="creator" />
            </div>
            <span v-if="user.bio" class="sidebar__item bio">{{ user.bio }}</span>
            <hr class="card-divider" />
            <div class="primary-stat">
              <DownloadIcon class="primary-stat__icon" aria-hidden="true" />
              <div class="primary-stat__text">
                <span class="primary-stat__counter">{{ sumDownloads }}</span>
                downloads
              </div>
            </div>
            <div class="primary-stat">
              <HeartIcon class="primary-stat__icon" aria-hidden="true" />
              <div class="primary-stat__text">
                <span class="primary-stat__counter">{{ sumFollows }}</span>
                followers of projects
              </div>
            </div>
            <div class="stats-block__item secondary-stat">
              <SunriseIcon class="secondary-stat__icon" aria-hidden="true" />
              <span
                v-tooltip="$dayjs(user.created).format('MMMM D, YYYY [at] h:mm A')"
                class="secondary-stat__text date"
              >
                Joined {{ fromNow(user.created) }}
              </span>
            </div>
            <hr class="card-divider" />
            <div class="stats-block__item secondary-stat">
              <UserIcon class="secondary-stat__icon" aria-hidden="true" />
              <span class="secondary-stat__text"> User ID: <CopyCode :text="user.id" /> </span>
            </div>
            <a
              v-if="githubUrl"
              :href="githubUrl"
              :target="$external()"
              rel="noopener noreferrer nofollow"
              class="sidebar__item github-button iconified-button"
            >
              <GitHubIcon aria-hidden="true" />
              View GitHub profile
            </a>
          </template>
        </div>
      </div>
      <div class="normal-page__content">
        <Promotion />
        <nav class="navigation-card">
          <NavRow
            :links="[
              {
                label: 'all',
                href: `/user/${user.username}`,
              },
              ...projectTypes.map((x) => {
                return {
                  label: $formatProjectType(x) + 's',
                  href: `/user/${user.username}/${x}s`,
                }
              }),
            ]"
          />
          <div class="input-group">
            <NuxtLink
              v-if="$auth.user && $auth.user.id === user.id"
              class="iconified-button"
              to="/dashboard/projects"
            >
              <SettingsIcon />
              Manage projects
            </NuxtLink>
            <button
              v-tooltip="$capitalizeString($cosmetics.searchDisplayMode.user) + ' view'"
              :aria-label="$capitalizeString($cosmetics.searchDisplayMode.user) + ' view'"
              class="square-button"
              @click="cycleSearchDisplayMode()"
            >
              <GridIcon v-if="$cosmetics.searchDisplayMode.user === 'grid'" />
              <ImageIcon v-else-if="$cosmetics.searchDisplayMode.user === 'gallery'" />
              <ListIcon v-else />
            </button>
          </div>
        </nav>
        <div
          v-if="projects.length > 0"
          class="project-list"
          :class="'display-mode--' + $cosmetics.searchDisplayMode.user"
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
            :display="$cosmetics.searchDisplayMode.user"
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
              $auth.user && ($auth.user.id === user.id || $tag.staffRoles.includes($auth.user.role))
                ? project.status
                : null
            "
            :type="project.project_type"
            :color="project.color"
          />
        </div>
        <div v-else class="error">
          <UpToDate class="icon" /><br />
          <span v-if="$auth.user && $auth.user.id === user.id" class="text">
            You don't have any projects.<br />
            Would you like to
            <a class="link" @click.prevent="$refs.modal_creation.show()"> create one</a>?
          </span>
          <span v-else class="text">This user has no projects!</span>
        </div>
      </div>
    </div>
  </div>
</template>
<script setup>
import ProjectCard from '~/components/ui/ProjectCard.vue'
import Badge from '~/components/ui/Badge.vue'
import Promotion from '~/components/ads/Promotion.vue'

import GitHubIcon from '~/assets/images/utils/github.svg'
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

let user, projects
try {
  ;[{ data: user }, { data: projects }] = await Promise.all([
    useAsyncData(`user/${route.params.id}`, () =>
      useBaseFetch(`user/${route.params.id}`, data.$defaultHeaders())
    ),
    useAsyncData(
      `user/${route.params.id}/projects`,
      () => useBaseFetch(`user/${route.params.id}/projects`, data.$defaultHeaders()),
      {
        transform: (projects) => {
          for (const project of projects) {
            project.categories = project.categories.concat(project.loaders)
            project.project_type = data.$getProjectTypeForUrl(
              project.project_type,
              project.categories
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
    message: 'User not found',
  })
}

if (!user.value) {
  throw createError({
    fatal: true,
    statusCode: 404,
    message: 'User not found',
  })
}

let githubUrl
try {
  const githubUser = await $fetch(`https://api.github.com/user/` + user.value.github_id)
  githubUrl = ref(githubUser.html_url)
} catch {}

if (user.value.username !== route.params.id) {
  await navigateTo(`/user/${user.value.username}`, { redirectCode: 301 })
}

const metaDescription = ref(
  user.value.bio
    ? `${user.value.bio} - Download ${user.value.username}'s projects on Modrinth`
    : `Download ${user.value.username}'s projects on Modrinth`
)

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

  return data.$formatNumber(sum)
})
const sumFollows = computed(() => {
  let sum = 0

  for (const project of projects.value) {
    sum += project.followers
  }

  return data.$formatNumber(sum)
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
        `user/${data.$auth.user.id}/icon?ext=${
          icon.value.type.split('/')[icon.value.type.split('/').length - 1]
        }`,
        {
          method: 'PATCH',
          body: icon.value,
          ...data.$defaultHeaders(),
        }
      )
    }

    const reqData = {
      email: user.value.email,
      bio: user.value.bio,
    }
    if (user.value.username !== data.$auth.user.username) {
      reqData.username = user.value.username
    }

    await useBaseFetch(`user/${data.$auth.user.id}`, {
      method: 'PATCH',
      body: reqData,
      ...data.$defaultHeaders(),
    })
    await useAuth(data.$auth.token)

    isEditing.value = false
  } catch (err) {
    console.error(err)
    data.$notify({
      group: 'main',
      title: 'An error occurred',
      text: err.data.description,
      type: 'error',
    })
  }
  stopLoading()
}

function cycleSearchDisplayMode() {
  data.$cosmetics.searchDisplayMode.user = data.$cycleValue(
    data.$cosmetics.searchDisplayMode.user,
    data.$tag.projectViewModes
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

.github-button {
  display: inline-flex;
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

.button-group:first-child {
  margin-left: auto;
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
