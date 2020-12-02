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
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local'),
      },
    }

    const mod = (
      await axios.get(
        `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
        config
      )
    ).data

    const members = (
      await axios.get(
        `https://api.modrinth.com/api/v1/team/${mod.team}/members`,
        config
      )
    ).data
    for (let i = 0; i < members.length; i++) {
      members[i].avatar_url = (
        await axios.get(
          `https://api.modrinth.com/api/v1/user/${members[i].user_id}`,
          config
        )
      ).data.avatar_url
    }

    const versions = (
      await axios.get(
        `https://api.modrinth.com/api/v1/versions?ids=${JSON.stringify(
          mod.versions
        )}`,
        config
      )
    ).data.reverse()

    const body = (await axios.get(mod.body_url)).data

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
