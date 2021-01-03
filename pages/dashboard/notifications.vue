<template>
  <DashboardPage>
    <div class="section-header columns">
      <h3 class="column-grow-1">My invites</h3>
    </div>
    <div v-for="invite in invites" :key="invite.team_id" class="invite columns">
      <div class="text">
        <p>
          Invite to join <strong>{{ invite.username }}'s</strong> team.
        </p>
      </div>
      <div class="actions">
        <button @click="declineInvite(invite.team_id)">Decline</button>
        <button @click="acceptInvite(invite.team_id)">Accept</button>
      </div>
    </div>
  </DashboardPage>
</template>

<script>
import axios from 'axios'
import DashboardPage from '@/components/DashboardPage'

export default {
  components: {
    DashboardPage,
  },
  async asyncData(data) {
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    const teams = (
      await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/teams`,
        config
      )
    ).data.filter((it) => !it.accepted)

    const members = (
      await Promise.all(
        teams.map((it) =>
          axios.get(
            `https://api.modrinth.com/api/v1/team/${it.team_id}/members`,
            config
          )
        )
      )
    ).map((it) => it.data)

    const invites = []

    for (const member of members) {
      const owner = member.find((it) => it.role === 'Owner')

      const ownerData = (
        await axios.get(
          `https://api.modrinth.com/api/v1/user/${owner.user_id}`,
          config
        )
      ).data

      invites.push({
        team_id: owner.team_id,
        username: ownerData.username,
      })
    }

    return {
      invites,
    }
  },
  methods: {
    async acceptInvite(teamId) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local'),
        },
      }

      this.$nuxt.$loading.start()

      try {
        await axios.post(
          `https://api.modrinth.com/api/v1/team/${teamId}/join`,
          config
        )
        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
    async declineInvite(teamId) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local'),
        },
      }

      this.$nuxt.$loading.start()

      try {
        await axios.delete(
          `https://api.modrinth.com/api/v1/team/${teamId}/members/${this.$auth.user.id}`,
          config
        )
        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
  },
}
</script>

<style lang="scss" scoped>
.invite {
  @extend %card;
  padding: var(--spacing-card-sm) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-sm);
  align-items: center;
  justify-content: space-between;

  p {
    margin: 0;
  }
}
</style>
