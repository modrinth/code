<template>
  <div class="page-container">
    <Popup v-if="currentProject" :show-popup="true">
      <div class="moderation-popup">
        <h2>Moderation form</h2>
        <p>
          Both of these fields are optional, but can be used to communicate
          problems with a project's team members. The body supports markdown
          formatting!
        </p>
        <div class="status">
          <span>New project status: </span>
          <Badge
            v-if="currentProject.newStatus === 'approved'"
            color="green"
            :type="currentProject.newStatus"
          />
          <Badge
            v-else-if="
              currentProject.newStatus === 'processing' ||
              currentProject.newStatus === 'unlisted' ||
              currentProject.newStatus === 'archived'
            "
            color="yellow"
            :type="currentProject.newStatus"
          />
          <Badge
            v-else-if="currentProject.newStatus === 'rejected'"
            color="red"
            :type="currentProject.newStatus"
          />
          <Badge v-else color="gray" :type="currentProject.newStatus" />
        </div>
        <input
          v-model="currentProject.moderation_message"
          type="text"
          placeholder="Enter the message..."
        />
        <h3>Body</h3>
        <ThisOrThat v-model="bodyViewMode" :items="['source', 'preview']" />
        <div v-if="bodyViewMode === 'source'" class="textarea-wrapper">
          <textarea
            id="body"
            v-model="currentProject.moderation_message_body"
          />
        </div>
        <div
          v-if="bodyViewMode === 'preview'"
          v-highlightjs
          class="markdown-body"
          v-html="$xss($md.render(currentProject.moderation_message_body))"
        ></div>
        <div class="buttons">
          <button class="iconified-button" @click="currentProject = null">
            <CrossIcon />
            Cancel
          </button>
          <button
            class="iconified-button brand-button-colors"
            @click="saveProject"
          >
            <CheckIcon />
            Confirm
          </button>
        </div>
      </div>
    </Popup>
    <div class="page-contents">
      <div class="content">
        <h1>Moderation</h1>
        <ThisOrThat
          v-model="selectedType"
          class="card"
          :items="moderationTypes"
        />
        <div class="projects">
          <ProjectCard
            v-for="project in selectedType !== 'all'
              ? projects.filter((x) => x.project_type === selectedType)
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
          >
            <button
              class="iconified-button"
              @click="setProjectStatus(project, 'approved')"
            >
              <CheckIcon />
              Approve
            </button>
            <button
              class="iconified-button"
              @click="setProjectStatus(project, 'unlisted')"
            >
              <UnlistIcon />
              Unlist
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
          v-if="selectedType === 'report' || selectedType === 'all'"
          class="reports"
        >
          <div
            v-for="(report, index) in reports"
            :key="report.id"
            class="report card"
          >
            <div class="header">
              <h5 class="title">
                Report for {{ report.item_type }}
                <nuxt-link
                  :to="
                    '/' +
                    report.item_type +
                    '/' +
                    report.item_id.replace(/\W/g, '')
                  "
                  >{{ report.item_id }}
                </nuxt-link>
              </h5>
              <p
                v-tooltip="
                  $dayjs(report.created).format(
                    '[Created at] YYYY-MM-DD [at] HH:mm A'
                  )
                "
                class="date"
              >
                Created {{ $dayjs(report.created).fromNow() }}
              </p>
              <button
                class="delete iconified-button"
                @click="deleteReport(index)"
              >
                Delete
              </button>
            </div>
            <div
              v-highlightjs
              class="markdown-body"
              v-html="$xss($md.render(report.body))"
            ></div>
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
import ThisOrThat from '~/components/ui/ThisOrThat'
import ProjectCard from '~/components/ui/ProjectCard'
import Popup from '~/components/ui/Popup'
import Badge from '~/components/ui/Badge'

import CheckIcon from '~/assets/images/utils/check.svg?inline'
import UnlistIcon from '~/assets/images/utils/eye-off.svg?inline'
import CrossIcon from '~/assets/images/utils/x.svg?inline'
import Security from '~/assets/images/illustrations/security.svg?inline'

export default {
  name: 'Moderation',
  components: {
    ThisOrThat,
    ProjectCard,
    CheckIcon,
    CrossIcon,
    UnlistIcon,
    Popup,
    Badge,
    Security,
  },
  async asyncData(data) {
    const [projects, reports] = (
      await Promise.all([
        data.$axios.get(`moderation/projects`, data.$defaultHeaders()),
        data.$axios.get(`report`, data.$defaultHeaders()),
      ])
    ).map((it) => it.data)

    return {
      projects,
      reports,
      selectedType: 'all',
    }
  },
  data() {
    return {
      bodyViewMode: 'source',
      currentProject: null,
    }
  },
  head: {
    title: 'Moderation - Modrinth',
  },
  computed: {
    moderationTypes() {
      const obj = { all: true }

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
      project.moderation_message = ''
      project.moderation_message_body = ''
      project.newStatus = status

      this.currentProject = project
    },
    async saveProject() {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `project/${this.currentProject.id}`,
          {
            moderation_message: this.currentProject.moderation_message
              ? this.currentProject.moderation_message
              : null,
            moderation_message_body: this.currentProject.moderation_message_body
              ? this.currentProject.moderation_message_body
              : null,
            status: this.currentProject.newStatus,
          },
          this.$defaultHeaders()
        )

        this.projects.splice(
          this.projects.findIndex((x) => this.currentProject.id === x.id),
          1
        )
        this.currentProject = null
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
.moderation-popup {
  width: auto;
  padding: var(--spacing-card-md) var(--spacing-card-lg);

  .status {
    display: flex;
    align-items: center;
    margin-bottom: 0.5rem;

    span {
      margin-right: 0.5rem;
    }
  }

  .buttons {
    display: flex;
    margin-top: 0.5rem;

    :first-child {
      margin-left: auto;
    }
  }
}

h1 {
  color: var(--color-text-dark);
  margin: 0 0 1rem 1.5rem;
}

.report {
  .header {
    display: flex;
    align-items: center;
    flex-direction: row;
    justify-content: center;
    .title {
      font-size: var(--font-size-lg);
      margin: 0 0.5rem 0 0;
    }
    .iconified-button {
      margin-left: auto;
    }
  }
}

@media screen and (min-width: 1024px) {
  .page-contents {
    max-width: calc(1280px - 20rem) !important;
  }
}
</style>
