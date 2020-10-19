<template>
  <div>{{ JSON.stringify(mod) }}</div>
</template>

<script>
import axios from 'axios'

export default {
  auth: false,
  /*
  {                                                                                                                                                                                                                                                                                                                                                                           13:42:51
  id: 'kN7Mtmyo',
  team: 'eiP0Hzmw',
  title: 'Gravestones',
  description: 'A gravestones mod for fabric with tons ' +
    'of config options, an API, and more!',
  body_url: 'https://cdn.modrinth.com/file/modrinth/data/kN7Mtmyo/body.md',
  published: '2020-10-16T21:17:54.858156Z',
  updated: '2020-10-16T21:17:50.982804Z',
  status: 'processing',
  downloads: 0,
  categories: [
    'adventure',
    'utility',
    'library'
  ],
  versions: [
    'XUky61nw'
  ],
  icon_url: 'https://cdn.modrinth.com/file/modrinth/mods/icons/kN7Mtmyo/gravestones.png',
  issues_url: null,
  source_url: null,
  wiki_url: null
}

   */
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/mod/${data.params.id}`
    )
    const mod = res.data

    res = await axios.get(mod.body_url)

    console.log(res.data)

    return {
      mod,
      modBody: res.data,
    }
  },
}
</script>

<style lang="scss"></style>
