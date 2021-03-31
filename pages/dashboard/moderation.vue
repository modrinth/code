<template>
  <DashboardPage>
    <div class="section-header">
      <h3 class="column-grow-1">Mods</h3>
    </div>
    <div v-if="mods.length !== 0">
      <ModCard
        v-for="(mod, index) in mods"
        :id="mod.id"
        :key="mod.id"
        :author="mod.author"
        :name="mod.title"
        :description="mod.description"
        :latest-version="mod.latest_version"
        :created-at="mod.published"
        :updated-at="mod.updated"
        :downloads="mod.downloads.toString()"
        :icon-url="mod.icon_url"
        :author-url="mod.author_url"
        :page-url="mod.page_url"
        :categories="mod.categories"
        :edit-mode="true"
        :status="mod.status"
        :is-modrinth="true"
      >
        <div class="buttons">
          <button
            class="button column approve"
            @click="changeModStatus(mod.id, 'approved', index)"
          >
            Approve
          </button>
          <button
            class="button column reject"
            @click="changeModStatus(mod.id, 'rejected', index)"
          >
            Reject
          </button>
        </div>
      </ModCard>
    </div>
    <div v-else class="error">
      <Security class="icon"></Security>
      <br />
      <span class="text">You are up-to-date!</span>
    </div>
    <div class="section-header">
      <h3 class="column-grow-1">Reports</h3>
    </div>
    <div v-if="reports.length !== 0">
      <div v-for="(report, index) in reports" :key="report.id" class="report">
        <div class="header">
          <h5 class="title">
            Report for {{ report.item_type }}
            <nuxt-link
              :to="report.item_type + '/' + report.item_id.replace(/\W/g, '')"
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
          <button class="delete iconified-button" @click="deleteReport(index)">
            Delete
          </button>
        </div>
        <div
          v-compiled-markdown="report.body"
          v-highlightjs
          class="markdown-body"
        ></div>
      </div>
    </div>
    <div v-else class="error">
      <Security class="icon"></Security><br />
      <span class="text">You are up-to-date!</span>
    </div>
  </DashboardPage>
</template>

<script>
import axios from 'axios'

import DashboardPage from '@/components/wrapper/DashboardPage'
import ModCard from '~/components/ui/ProjectCard'
import Security from '~/assets/images/illustrations/security.svg?inline'

export default {
  components: {
    DashboardPage,
    ModCard,
    Security,
  },
  async asyncData(data) {
    const mods = (
      await axios.get(
        `https://api.modrinth.com/api/v1/moderation/mods`,
        data.$auth.headers
      )
    ).data

    const reports = (
      await axios.get(
        `https://api.modrinth.com/api/v1/report`,
        data.$auth.headers
      )
    ).data

    return {
      mods,
      reports,
    }
  },
  methods: {
    async changeModStatus(id, status, index) {
      await axios.patch(
        `https://api.modrinth.com/api/v1/mod/${id}`,
        {
          status,
        },
        this.$auth.headers
      )

      this.mods.splice(index, 1)
    },
    async deleteReport(index) {
      await axios.delete(
        `https://api.modrinth.com/api/v1/report/${this.reports[index].id}`,
        this.$auth.headers
      )

      this.reports.splice(index, 1)
    },
  },
}
</script>

<style lang="scss" scoped>
.button {
  margin: 0 5rem 0.5rem auto;
}

.buttons {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.report {
  @extend %card-spaced-b;
  padding: var(--spacing-card-sm) var(--spacing-card-lg);

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

.error {
  display: flex;
  flex-direction: column;
  width: 100%;
  justify-content: center;
  align-items: center;

  .icon {
    width: 8rem;
    height: 8rem;
    margin: 1.5rem 0;
  }

  .text {
    margin-bottom: 2rem;
    font-size: 1.25rem;
  }
}
</style>
