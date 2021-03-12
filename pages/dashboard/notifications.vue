<template>
  <DashboardPage>
    <div class="section-header columns">
      <h3 class="column-grow-1">My notifications</h3>
    </div>
    <div
      v-for="notification in notifications"
      :key="notification.id"
      class="notification columns"
    >
      <div class="text">
        <nuxt-link :to="'/' + notification.link" class="top-wrapper">
          <h3 class="title">
            {{ notification.title }}
          </h3>
          <p
            v-tooltip="
              $dayjs(notification.created).format(
                '[Created at] YYYY-MM-DD [at] HH:mm A'
              )
            "
            class="date"
          >
            Notified {{ $dayjs(notification.created).fromNow() }}
          </p>
        </nuxt-link>
        <p class="description">
          {{ notification.text }}
        </p>
      </div>
      <div v-if="notification.actions.length > 0" class="actions">
        <button
          v-for="(action, index) in notification.actions"
          :key="index"
          @click="performAction(notification, index)"
        >
          {{ action.title }}
        </button>
      </div>
      <div v-else class="actions">
        <button @click="performAction(notification, null)">Dismiss</button>
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

    const notifications = (
      await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/notifications`,
        config
      )
    ).data

    return {
      notifications,
    }
  },
  methods: {
    async performAction(notification, index) {
      this.$nuxt.$loading.start()

      try {
        if (index) {
          const config = {
            method: notification.actions[index].action_route[0].toLowerCase(),
            url: `https://api.modrinth.com/api/v1/${notification.actions[index].action_route[1]}`,
            headers: {
              Authorization: this.$auth.getToken('local'),
            },
          }

          await axios(config)
        }

        const config = {
          headers: {
            Authorization: this.$auth.getToken('local')
              ? this.$auth.getToken('local')
              : '',
          },
        }

        await axios.delete(
          `https://api.modrinth.com/api/v1/notification/${notification.id}`,
          config
        )
        await this.$router.replace('/' + notification.link)
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
.notification {
  @extend %card;
  padding: var(--spacing-card-sm) var(--spacing-card-lg);
  margin-bottom: var(--spacing-card-sm);
  align-items: center;
  justify-content: space-between;

  .text {
    .top-wrapper {
      display: flex;
      flex-direction: row;
      align-items: baseline;

      .title {
        font-size: var(--font-size-lg);
        margin: 0 0.5rem 0 0;
      }
    }
  }

  p {
    margin: 0;
  }
}
</style>
