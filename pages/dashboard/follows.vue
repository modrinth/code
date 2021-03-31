<template>
  <DashboardPage>
    <div class="section-header">
      <h3 class="column-grow-1">Followed mods</h3>
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
    </div>
    <div v-else class="error">
      <FollowIllustration class="icon"></FollowIllustration><br />
      <span class="text"
        >You don't have any followed mods. <br />
        Why don't you <nuxt-link to="/mods" class="link">search</nuxt-link> for
        new ones?</span
      >
    </div>
  </DashboardPage>
</template>

<script>
import axios from 'axios'

import DashboardPage from '@/components/wrapper/DashboardPage'
import ModCard from '~/components/ui/ProjectCard'
import FollowIcon from '~/assets/images/utils/heart.svg?inline'
import FollowIllustration from '~/assets/images/illustrations/follow_illustration.svg?inline'

export default {
  components: {
    DashboardPage,
    ModCard,
    FollowIcon,
    FollowIllustration,
  },
  async asyncData(data) {
    const res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/follows`,
      data.$auth.headers
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
    text-align: center;
  }

  .link {
    text-decoration: underline;
  }
}
</style>
