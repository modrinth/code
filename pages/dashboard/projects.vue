<template>
  <div>
    <div class="section-header columns">
      <h3 class="column-grow-1">My mods</h3>
      <nuxt-link class="brand-button column" to="/mod/create">
        Create a mod
      </nuxt-link>
    </div>
    <div v-if="mods.length !== 0">
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
          class="button column edit-button"
          :to="'/mod/' + mod.id + '/settings'"
        >
          Settings
        </nuxt-link>
      </ModCard>
    </div>
    <div v-else class="error">
      <UpToDate class="icon"></UpToDate><br />
      <span class="text"
        >You don't have any mods.<br />
        Would you like to
        <nuxt-link class="link" to="/mod/create">create one</nuxt-link>?</span
      >
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import ModCard from '~/components/ui/ProjectCard'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'

export default {
  components: {
    ModCard,
    UpToDate,
  },
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/mods`,
      data.$auth.headers
    )

    res = await axios.get(
      `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`,
      data.$auth.headers
    )

    return {
      mods: res.data,
    }
  },
  head: {
    title: 'My mods - Modrinth',
  },
}
</script>

<style lang="scss" scoped>
.mod-name {
  font-weight: bold;
}

.edit-button {
  align-self: flex-end;
  margin-right: 2rem;
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
    text-align: center;
    margin-bottom: 2rem;
    font-size: 1.25rem;
  }

  .link {
    text-decoration: underline;
  }
}

// .buttonse {
//   margin-left: 4.5rem;
//   padding: 0.5rem 2rem 0.5rem 2rem;
// }
</style>
