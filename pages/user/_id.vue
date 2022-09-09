<template>
  <div class="normal-page">
    <div class="normal-page__sidebar">
      <aside class="card sidebar">
        <img
          class="sidebar__item profile-picture"
          :src="user.avatar_url"
          :alt="user.username"
        />
        <h1 class="sidebar__item username">{{ user.username }}</h1>
        <div class="sidebar__item">
          <Badge v-if="user.role === 'admin'" type="admin" color="red" />
          <Badge
            v-else-if="user.role === 'moderator'"
            type="moderator"
            color="yellow"
          />
          <Badge v-else type="developer" color="green" />
        </div>
        <hr class="card-divider" />
        <h3 class="sidebar__item">About me</h3>
        <span v-if="user.bio" class="sidebar__item bio">{{ user.bio }}</span>
        <a
          :href="githubUrl"
          target="_blank"
          class="sidebar__item report-button iconified-button"
        >
          <GitHubIcon aria-hidden="true" />
          View GitHub profile
        </a>
        <div class="sidebar__item stats-block">
          <div class="stats-block__item secondary-stat">
            <SunriseIcon class="secondary-stat__icon" aria-hidden="true" />
            <span
              v-tooltip="
                $dayjs(user.created).format('MMMM D, YYYY [at] h:mm:ss A')
              "
              class="secondary-stat__text date"
            >
              Joined {{ $dayjs(user.created).fromNow() }}
            </span>
          </div>
          <div class="stats-block__item secondary-stat">
            <UserIcon class="secondary-stat__icon" aria-hidden="true" />
            <span class="secondary-stat__text">User ID: {{ user.id }}</span>
          </div>
        </div>
        <div class="sidebar__item stats-block">
          <div class="stats-block__item primary-stat">
            <DownloadIcon class="primary-stat__icon" aria-hidden="true" />
            <div class="primary-stat__text">
              <span class="primary-stat__counter">{{ sumDownloads() }}</span>
              <span class="primary-stat__label">downloads</span>
            </div>
          </div>
        </div>
        <template
          v-if="!$auth.user || ($auth.user && $auth.user.id !== user.id)"
        >
          <hr class="card-divider" />
          <nuxt-link
            :to="`/create/report?id=${user.id}&t=user`"
            class="sidebar__item report-button iconified-button"
          >
            <ReportIcon aria-hidden="true" />
            Report
          </nuxt-link>
        </template>
      </aside>
    </div>
    <div class="normal-page__content">
      <Advertisement />
      <nav class="card user-navigation">
        <ThisOrThat v-model="selectedProjectType" :items="projectTypes" />
        <nuxt-link
          v-if="$auth.user && $auth.user.id === user.id"
          to="/create/project"
          class="iconified-button brand-button-colors"
        >
          <PlusIcon />
          Create a project
        </nuxt-link>
      </nav>
      <div v-if="projects.length > 0">
        <ProjectCard
          v-for="project in selectedProjectType !== 'all'
            ? projects.filter(
                (x) =>
                  x.project_type === convertProjectType(selectedProjectType)
              )
            : projects"
          :id="project.slug || project.id"
          :key="project.id"
          :name="project.title"
          :description="project.description"
          :created-at="project.published"
          :updated-at="project.updated"
          :downloads="project.downloads.toString()"
          :follows="project.followers.toString()"
          :icon-url="project.icon_url"
          :categories="project.categories"
          :client-side="project.client_side"
          :server-side="project.server_side"
          :status="project.status"
          :type="project.project_type"
        >
          <nuxt-link
            v-if="$auth.user && $auth.user.id === user.id"
            class="iconified-button"
            :to="`/${project.project_type}/${
              project.slug ? project.slug : project.id
            }/settings`"
          >
            <SettingsIcon />
            Settings
          </nuxt-link>
        </ProjectCard>
      </div>
      <div v-else class="error">
        <UpToDate class="icon" /><br />
        <span v-if="$auth.user && $auth.user.id === user.id" class="text"
          >You don't have any projects.<br />
          Would you like to
          <nuxt-link class="link" to="/create/project">create one</nuxt-link
          >?</span
        >
        <span v-else class="text">This user has no projects!</span>
      </div>
    </div>
  </div>
</template>

<script>
import ProjectCard from '~/components/ui/ProjectCard'
import ThisOrThat from '~/components/ui/ThisOrThat'
import Badge from '~/components/ui/Badge'
import Advertisement from '~/components/ads/Advertisement'

import GitHubIcon from '~/assets/images/utils/github.svg?inline'
import ReportIcon from '~/assets/images/utils/report.svg?inline'
import SunriseIcon from '~/assets/images/utils/sunrise.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import SettingsIcon from '~/assets/images/utils/settings.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'
import UserIcon from '~/assets/images/utils/user.svg?inline'

export default {
  auth: false,
  components: {
    ProjectCard,
    SunriseIcon,
    DownloadIcon,
    GitHubIcon,
    ReportIcon,
    Badge,
    SettingsIcon,
    PlusIcon,
    ThisOrThat,
    UpToDate,
    UserIcon,
    Advertisement,
  },
  async asyncData(data) {
    try {
      const [user, projects] = (
        await Promise.all([
          data.$axios.get(`user/${data.params.id}`, data.$defaultHeaders()),
          data.$axios.get(
            `user/${data.params.id}/projects`,
            data.$defaultHeaders()
          ),
        ])
      ).map((it) => it.data)

      const [gitHubUser, versions] = (
        await Promise.all([
          data.$axios.get(`https://api.github.com/user/` + user.github_id),
          data.$axios.get(
            `versions?ids=${JSON.stringify(
              [].concat.apply(
                [],
                projects.map((x) => x.versions)
              )
            )}`
          ),
        ])
      ).map((it) => it.data)

      for (const version of versions) {
        const projectIndex = projects.findIndex(
          (x) => x.id === version.project_id
        )

        if (projects[projectIndex].loaders) {
          for (const loader of version.loaders) {
            if (!projects[projectIndex].loaders.includes(loader)) {
              projects[projectIndex].loaders.push(loader)
            }
          }
        } else {
          projects[projectIndex].loaders = version.loaders
        }
      }

      for (const project of projects) {
        project.categories = project.categories.concat(project.loaders)

        project.project_type = data.$getProjectTypeForUrl(
          project.project_type,
          project.categories
        )
      }

      return {
        selectedProjectType: 'all',
        user,
        projects,
        githubUrl: gitHubUser.html_url,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'User not found',
      })
    }
  },
  head() {
    return {
      title: this.user.username + ' - Modrinth',
      meta: [
        {
          hid: 'og:type',
          name: 'og:type',
          content: 'website',
        },
        {
          hid: 'og:title',
          name: 'og:title',
          content: this.user.username,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.user.username,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.user.bio,
        },
        {
          hid: 'description',
          name: 'description',
          content:
            this.user.bio +
            ' - View Minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform.',
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/user/${this.user.id}`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content:
            this.user.avatar_url || 'https://cdn.modrinth.com/placeholder.png',
        },
      ],
    }
  },
  computed: {
    projectTypes() {
      const obj = { all: true }

      for (const project of this.projects) {
        if (project.project_type === 'resourcepack') {
          obj['Resource Packs'] = true
        } else {
          obj[project.project_type + 's'] = true
        }
      }

      return Object.keys(obj)
    },
  },
  methods: {
    convertProjectType(name) {
      if (name === 'Resource Packs') {
        return 'resourcepack'
      } else {
        return name.slice(0, -1)
      }
    },
    sumDownloads() {
      let sum = 0

      for (const projects of this.projects) {
        sum += projects.downloads
      }

      return this.$formatNumber(sum)
    },
  },
}
</script>

<style scoped>
.user-navigation {
  align-items: center;
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  row-gap: 0.5rem;
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

.report-button {
  display: inline-flex;
}

.bio {
  display: block;
}

.stats-block__item {
  margin-bottom: 0.25rem;
}

.secondary-stat {
  align-items: center;
  color: var(--color-text-secondary);
  display: flex;
}

.secondary-stat__icon {
  height: 1rem;
  width: 1rem;
}

.secondary-stat__text {
  margin-left: 0.25rem;
}

.primary-stat {
  align-items: center;
  display: flex;
}

.primary-stat__icon {
  height: 1.25rem;
  width: 1.25rem;
}

.primary-stat__text {
  margin-left: 0.25rem;
}

.primary-stat__counter {
  font-size: var(--font-size-lg);
  font-weight: bold;
}

.date {
  cursor: default;
}
</style>
