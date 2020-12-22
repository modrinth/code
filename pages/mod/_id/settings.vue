<template>
  <ModPage :mod="mod" :versions="versions" :members="members"> </ModPage>
</template>

<script>
import axios from 'axios'
import ModPage from '@/components/ModPage'

export default {
  components: { ModPage },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    const mod = (
      await axios.get(
        `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
        config
      )
    ).data

    const [members, allMembers, versions] = (
      await Promise.all([
        axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
        axios.get(
          `https://api.modrinth.com/api/v1/team/${mod.team}/members`,
          config
        ),
        axios.get(
          `https://api.modrinth.com/api/v1/versions?ids=${JSON.stringify(
            mod.versions
          )}`,
          config
        ),
      ])
    ).map((it) => it.data)

    const [users, allUsers] = (
      await Promise.all([
        axios.get(
          `https://api.modrinth.com/api/v1/users?ids=${JSON.stringify(
            members.map((it) => it.user_id)
          )}`,
          config
        ),
        axios.get(
          `https://api.modrinth.com/api/v1/users?ids=${JSON.stringify(
            allMembers.map((it) => it.user_id)
          )}`,
          config
        ),
      ])
    ).map((it) => it.data)

    users.forEach((it, index) => {
      members[index].avatar_url = it.avatar_url
      members[index].name = it.username
    })

    allUsers.forEach((it, index) => {
      allMembers[index].avatar_url = it.avatar_url
      allMembers[index].name = it.username
      allMembers[index].accepted = members.find(
        (it) => allMembers[index].user_id === it.user_id
      )
    })

    return {
      mod,
      versions: versions.reverse(),
      members,
      allMembers,
      allUsers,
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
