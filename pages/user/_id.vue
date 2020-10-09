<template>
  <div>
    <img :src="user.avatar_url" />
  </div>
</template>

<script>
import axios from 'axios'

export default {
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
        `https://api.modrinth.com/api/v1/mods/?ids=${JSON.stringify(res.data)}`
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
