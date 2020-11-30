<template>
  <ModPage :mod="mod" :versions="versions" :members="members">
    <div v-compiled-markdown="body" class="markdown-body"></div>
  </ModPage>
</template>

<script>
import axios from 'axios'
import ModPage from '@/components/ModPage'

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

    const body = (await axios.get(mod.body_url)).data

    const versions = []

    for (const version of mod.versions) {
      try {
        res = await axios.get(
          `https://api.modrinth.com/api/v1/version/${version}`
        )
        versions.push(res.data)
      } catch {
        // eslint-disable-next-line no-console
        console.log('Some versions may be missing...')
      }
    }

    return {
      mod,
      body,
      versions,
      members,
    }
  },
  head() {
    return {
      title: this.mod.title + ' - Modrinth',
      meta: [
        {
          hid: 'og:type',
          name: 'og:type',
          content: 'website',
        },
        {
          hid: 'og:title',
          name: 'og:title',
          content: this.mod.title,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.mod.title,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.mod.description,
        },
        {
          hid: 'description',
          name: 'description',
          content:
            this.mod.description +
            ' View other minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform that is compatible with CurseForge too!',
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/mod/${this.mod.id}`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content: this.mod.icon_url
            ? this.mod.icon_url
            : 'https://cdn.modrinth.com/placeholder.png',
        },
      ],
    }
  },
}
</script>

<style lang="scss" scoped>
.markdown-body {
  padding: 1rem;
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
}
</style>
