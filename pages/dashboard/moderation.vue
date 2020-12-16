<template>
  <DashboardPage>
    <div class="section-header">
      <h3 class="column-grow-1">Mods</h3>
    </div>
    <ModCard
      v-for="mod in mods"
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
      <button
        class="button column approve"
        @click="changeModStatus(mod.id, 'approved')"
      >
        Approve
      </button>
      <button
        class="button column reject"
        @click="changeModStatus(mod.id, 'rejected')"
      >
        Reject
      </button>
    </ModCard>
    <div class="section-header">
      <h3 class="column-grow-1">Versions</h3>
    </div>
  </DashboardPage>
</template>

<script>
import axios from 'axios'

import ModCard from '@/components/ProjectCard'
import DashboardPage from '@/components/DashboardPage'

export default {
  components: {
    DashboardPage,
    ModCard,
  },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    let res = await axios.get(
      `https://api.modrinth.com/api/v1/moderation/mods`,
      config
    )

    const mods = res.data

    res = await axios.get(
      `https://api.modrinth.com/api/v1/moderation/versions`,
      config
    )

    return {
      mods,
      versions: res.data,
    }
  },
  methods: {
    async changeModStatus(id, status) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local')
            ? this.$auth.getToken('local')
            : '',
        },
      }

      await axios.patch(
        `https://api.modrinth.com/api/v1/mod/${id}`,
        {
          status,
        },
        config
      )

      await this.$router.go(0)
    },
  },
}
</script>

<style lang="scss" scoped>
.button {
  margin: 0.25rem 0;
}
</style>
