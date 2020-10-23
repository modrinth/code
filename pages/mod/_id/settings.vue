<template>
  <ModPage :mod="mod" :versions="versions" :members="members"></ModPage>
</template>

<script>
import ModPage from '@/components/ModPage'
import axios from 'axios'

export default {
  components: { ModPage },
  auth: false,
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/mod/${data.params.id}`
    )
    const mod = res.data

    res = await axios.get(
      `https://api.modrinth.com/api/v1/team/${mod.team}/members`
    )
    const members = res.data
    for (let i = 0; i < members.length; i++) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${members[i].user_id}`
      )
      members[i].avatar_url = res.data.avatar_url
    }

    res = await axios.get(mod.body_url)

    const versions = []

    for (const version of mod.versions) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/version/${version}`
      )

      versions.push(res.data)
    }

    return {
      mod,
      versions,
      members,
    }
  },
}
</script>

<style lang="scss" scoped></style>
