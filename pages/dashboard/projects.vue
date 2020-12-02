<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="sidebar-l">
        <div class="card page-nav">
          <nuxt-link :to="'/dashboard/projects'" class="tab last">
            <ModIcon />
            My mods
          </nuxt-link>
          <nuxt-link
            v-if="
              $auth.user.role === 'admin' || $auth.user.role === 'moderator'
            "
            :to="'/dashboard/moderation'"
            class="tab last"
          >
            <ModerationIcon />
            Moderation
          </nuxt-link>
        </div>
        <client-only>
          <EthicalAd type="image" />
        </client-only>
      </div>
      <div class="content">
        <div class="section-header columns">
          <h3 class="column-grow-1">My mods</h3>
          <nuxt-link class="brand-button column" to="/mod/create">
            Create a mod
          </nuxt-link>
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
        />
      </div>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import EthicalAd from '@/components/EthicalAd'
import ModCard from '@/components/ProjectCard'

import ModIcon from '~/assets/images/sidebar/mod.svg?inline'
import ModerationIcon from '~/assets/images/sidebar/admin.svg?inline'

export default {
  components: {
    EthicalAd,
    ModCard,
    ModIcon,
    ModerationIcon,
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
      `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/mods`,
      config
    )

    res = await axios.get(
      `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`,
      config
    )

    return {
      mods: res.data,
    }
  },
}
</script>

<style lang="scss" scoped>
.section-header {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-md);
  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }
}

.mod-name {
  font-weight: bold;
}
</style>
