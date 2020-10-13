<template>
  <div>
    <img :src="user.avatar_url" />
    <div>
      <SearchResult
        v-for="(result, index) in mods"
        :id="result.mod_id"
        :key="result.mod_id"
        :name="result.title"
        :description="result.description"
        :latest-version="result.versions[0]"
        :created-at="result.published"
        :downloads="result.downloads.toString()"
        :icon-url="result.icon_url"
        :author-url="result.author_url"
        :page-url="result.page_url"
        :categories="result.categories"
        :is-ad="index === -1"
      />
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import SearchResult from '@/components/ModResult'

export default {
  auth: false,
  components: {
    SearchResult,
  },
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.params.id}`
    )
    const user = res.data

    let mods = []
    res = await axios.get(
      `https://api.modrinth.com/api/v1/user/${data.params.id}/mods`
    )
    if (res.data) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/mods?ids=${JSON.stringify(res.data)}`
      )
      mods = res.data
    }

    return {
      mods,
      user,
    }
  },
}
</script>

<style lang="scss"></style>
