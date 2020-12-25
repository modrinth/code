<template>
  <DashboardPage>
    <div class="section-header columns">
      <h3 class="column-grow-1">My mods</h3>
      <nuxt-link class="brand-button column" to="/mod/create">
        Create a mod
      </nuxt-link>
    </div>
    <ModCard
      v-for="mod in mods"
      :id="mod.slug ? mod.slug : mod.id"
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
      <nuxt-link
        class="button buttonse column"
        :to="'/mod/' + mod.id + '/edit'"
      >
        Edit
      </nuxt-link>
    </ModCard>
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
.mod-name {
  font-weight: bold;
}
// .buttonse {
//   margin-left: 4.5rem;
//   padding: 0.5rem 2rem 0.5rem 2rem;
// }
</style>
