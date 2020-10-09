<template>
  <div></div>
</template>

<script>
import axios from 'axios'

export default {
  async asyncData(data) {
    const res = await axios.get(
      `https://api.modrinth.com/api/v1/mod/${data.params.id}`
    )

    return {
      mod: res.data,
    }
  },
}
</script>

<style lang="scss"></style>
