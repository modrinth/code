<template>
  <ModPage
    :mod="mod"
    :featured-versions="featuredVersions"
    :members="members"
    :current-member="currentMember"
    :link-bar="[['Description', '']]"
  >
    <div
      v-compiled-markdown="mod.body"
      v-highlightjs
      class="markdown-body"
    ></div>
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
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    try {
      const mod = (
        await axios.get(
          `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
          config
        )
      ).data

      if (mod.body_url && !mod.body) {
        mod.body = (await axios.get(mod.body_url)).data
      }

      const [members, featuredVersions] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
          axios.get(
            `https://api.modrinth.com/api/v1/mod/${mod.id}/version?featured=true`
          ),
        ])
      ).map((it) => it.data)

      const users = (
        await axios.get(
          `https://api.modrinth.com/api/v1/users?ids=${JSON.stringify(
            members.map((it) => it.user_id)
          )}`,
          config
        )
      ).data

      users.forEach((it) => {
        const index = members.findIndex((x) => x.user_id === it.id)
        members[index].avatar_url = it.avatar_url
        members[index].name = it.username
      })

      const currentMember = data.$auth.loggedIn
        ? members.find((x) => x.user_id === data.$auth.user.id)
        : null

      return {
        mod,
        featuredVersions,
        members,
        currentMember,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Mod not found',
      })
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
