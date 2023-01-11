<template>
  <div>
    <ModalModeration
      ref="modal"
      :project="currentProject"
      :status="currentStatus"
      :on-close="onModalClose"
    />
    <div class="normal-page">
      <div class="normal-page__sidebar">
        <aside class="universal-card">
          <h1>Moderation</h1>
          <NavStack>
            <NavStackItem link="" label="All"> </NavStackItem>
            <NavStackItem
              v-for="type in moderationTypes"
              :key="type"
              :link="'?type=' + type"
              :label="$formatProjectType(type) + 's'"
            >
            </NavStackItem>
          </NavStack>
        </aside>
      </div>
      <div class="normal-page__content">
        <div class="project-list display-mode--list">
          <ProjectCard
            v-for="project in $route.query.type !== undefined
              ? projects.filter((x) => x.project_type === $route.query.type)
              : projects"
            :id="project.slug || project.id"
            :key="project.id"
            :name="project.title"
            :description="project.description"
            :created-at="project.published"
            :updated-at="project.updated"
            :icon-url="project.icon_url"
            :categories="project.categories"
            :client-side="project.client_side"
            :server-side="project.server_side"
            :type="project.project_type"
            :color="project.color"
            :moderation="true"
          >
            <button
              class="iconified-button"
              @click="
                setProjectStatus(
                  project,
                  project.requested_status
                    ? project.requested_status
                    : 'approved'
                )
              "
            >
              <CheckIcon />
              Approve
            </button>
            <button
              class="iconified-button"
              @click="setProjectStatus(project, 'withheld')"
            >
              <UnlistIcon />
              Withhold
            </button>
            <button
              class="iconified-button"
              @click="setProjectStatus(project, 'rejected')"
            >
              <CrossIcon />
              Reject
            </button>
          </ProjectCard>
        </div>
        <div
          v-if="
            $route.query.type === 'report' || $route.query.type === undefined
          "
          class="reports"
        >
          <div
            v-for="(item, index) in reports"
            :key="index"
            class="card report"
          >
            <div class="info">
              <div class="title">
                <h3>
                  {{ item.item_type }}
                  <nuxt-link :to="item.url">{{ item.item_id }}</nuxt-link>
                </h3>
                reported by
                <a :href="`/user/${item.reporter}`">{{ item.reporter }}</a>
              </div>
              <div
                v-highlightjs
                class="markdown-body"
                v-html="$xss($md.render(item.body))"
              />
              <Badge :type="`Marked as ${item.report_type}`" color="orange" />
            </div>
            <div class="actions">
              <button class="iconified-button" @click="deleteReport(index)">
                <TrashIcon /> Delete report
              </button>
              <span
                v-tooltip="
                  $dayjs(item.created).format(
                    '[Created at] YYYY-MM-DD [at] HH:mm A'
                  )
                "
                class="stat"
              >
                <CalendarIcon />
                Created {{ $dayjs(item.created).fromNow() }}
              </span>
            </div>
          </div>
        </div>
        <div v-if="reports.length === 0 && projects.length === 0" class="error">
          <Security class="icon"></Security>
          <br />
          <span class="text">You are up-to-date!</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import ProjectCard from '~/components/ui/ProjectCard'
import Badge from '~/components/ui/Badge'

import CheckIcon from '~/assets/images/utils/check.svg?inline'
import UnlistIcon from '~/assets/images/utils/eye-off.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import Security from '~/assets/images/illustrations/security.svg?inline'
import NavStack from '~/components/ui/NavStack'
import NavStackItem from '~/components/ui/NavStackItem'
import ModalModeration from '~/components/ui/ModalModeration'

export default {
  name: 'Moderation',
  components: {
    ModalModeration,
    NavStack,
    NavStackItem,
    ProjectCard,
    CheckIcon,
    CrossIcon,
    UnlistIcon,
    Badge,
    Security,
    TrashIcon,
    CalendarIcon,
  },
  async asyncData(data) {
    const [projects, reports] = (
      await Promise.all([
        data.$axios.get(`moderation/projects`, data.$defaultHeaders()),
        data.$axios.get(`report`, data.$defaultHeaders()),
      ])
    ).map((it) => it.data)

    const newReports = await Promise.all(
      reports.map(async (report) => {
        try {
          report.item_id = report.item_id?.replace
            ? report.item_id.replace(/"/g, '')
            : report.item_id
          let url = ''

          if (report.item_type === 'user') {
            const user = (
              await data.$axios.get(
                `user/${report.item_id}`,
                data.$defaultHeaders()
              )
            ).data
            url = `/user/${user.username}`
            report.item_id = user.username
          } else if (report.item_type === 'project') {
            const project = (
              await data.$axios.get(
                `project/${report.item_id}`,
                data.$defaultHeaders()
              )
            ).data
            report.item_id = project.slug || report.item_id
            url = `/${project.project_type}/${report.item_id}`
          } else if (report.item_type === 'version') {
            const version = (
              await data.$axios.get(
                `version/${report.item_id}`,
                data.$defaultHeaders()
              )
            ).data
            const project = (
              await data.$axios.get(
                `project/${version.project_id}`,
                data.$defaultHeaders()
              )
            ).data
            report.item_id = version.version_number || report.item_id
            url = `/${project.project_type}/${
              project.slug || project.id
            }/version/${report.item_id}`
          }

          report.reporter = (
            await data.$axios.get(
              `user/${report.reporter}`,
              data.$defaultHeaders()
            )
          ).data.username

          return {
            ...report,
            moderation_type: 'report',
            url,
          }
        } catch (err) {
          return {
            ...report,
            url: 'error',
            moderation_type: 'report',
          }
        }
      })
    )

    return {
      projects,
      reports: newReports,
    }
  },
  data() {
    return {
      currentProject: null,
      currentStatus: null,
    }
  },
  head: {
    title: 'Moderation - Modrinth',
  },
  computed: {
    moderationTypes() {
      const obj = {}

      for (const project of this.projects) {
        obj[project.project_type] = true
      }

      if (this.reports.length > 0) {
        obj.report = true
      }

      return Object.keys(obj)
    },
  },
  methods: {
    setProjectStatus(project, status) {
      this.currentProject = project
      this.currentStatus = status

      this.$refs.modal.show()
    },
    onModalClose() {
      this.projects.splice(
        this.projects.findIndex((x) => this.project.id === x.id),
        1
      )
      this.project = null
    },
    async deleteReport(index) {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.delete(
          `report/${this.reports[index].id}`,
          this.$defaultHeaders()
        )

        this.reports.splice(index, 1)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
  },
}
</script>

<style lang="scss" scoped>
h1 {
  color: var(--color-text-dark);
}

.report {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem;

  > div {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .info {
    display: flex;
    flex-wrap: wrap;

    .title {
      display: flex;
      align-items: baseline;
      gap: 0.25rem;
      flex-wrap: wrap;

      h3 {
        margin: 0;
        text-transform: capitalize;

        a {
          text-transform: none;
        }
      }

      a {
        text-decoration: underline;
      }
    }
  }

  .actions {
    min-width: fit-content;

    .iconified-button {
      margin-left: auto;
      width: fit-content;
    }

    .stat {
      margin-top: auto;
      display: flex;
      align-items: center;
      grid-gap: 0.5rem;

      svg {
        width: 1em;
      }
    }
  }
}

@media screen and (min-width: 1024px) {
  .page-contents {
    max-width: 800px;
  }
}
</style>
