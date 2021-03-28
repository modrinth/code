<template>
  <DashboardPage>
    <div class="section-header">
      <h3 class="column-grow-1">Followed mods</h3>
    </div>
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
      :is-modrinth="true"
    >
      <div class="buttons">
        <button
          class="button column unfav-button iconified-button"
          @click="unfavMod(index)"
        >
          <FollowIcon />
          Unfollow
        </button>
      </div>
    </ModCard>
  </DashboardPage>
</template>

<script>
import axios from 'axios'

import ModCard from '@/components/ProjectCard'
import DashboardPage from '@/components/DashboardPage'
import FollowIcon from '~/assets/images/utils/heart.svg?inline'

export default {
  components: {
    DashboardPage,
    ModCard,
    FollowIcon,
  },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    const res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/follows`,
      config
    )

    const mods = (
      await axios.get(
        `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`
      )
    ).data

    return {
      mods,
    }
  },
  methods: {
    async unfavMod(index) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local')
            ? this.$auth.getToken('local')
            : '',
        },
      }

      await axios.delete(
        `https://api.modrinth.com/api/v1/mod/${this.mods[index].id}/follow`,
        config
      )

      this.mods.splice(index, 1)
    },
  },
}
</script>

<style lang="scss" scoped>
.button {
  margin: 0.25rem 2rem 0.25rem 0;
}

.buttons {
  flex-grow: 1;
  display: flex;
  flex-direction: row;
  justify-content: center;
}

.unfav-button {
  margin-left: auto;
  padding: 0.5rem;
}
</style>
