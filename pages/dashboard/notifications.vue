<template>
  <div>
    <div class="section-header columns">
      <h3 class="column-grow-1">My notifications</h3>
    </div>
    <div v-if="notifications.length !== 0">
      <div
        v-for="(notification, notificationIndex) in notifications"
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
            v-for="(action, actionIndex) in notification.actions"
            :key="actionIndex"
            @click="performAction(notification, notificationIndex, actionIndex)"
          >
            {{ action.title }}
          </button>
        </div>
        <div v-else class="actions">
          <button @click="performAction(notification, notificationIndex, null)">
            Dismiss
          </button>
        </div>
      </div>
    </div>
    <div v-else class="error">
      <UpToDate class="icon"></UpToDate>
      <br />
      <span class="text">You are up-to-date!</span>
    </div>
  </div>
</template>

<script>
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'

export default {
  components: {
    UpToDate,
  },
  async asyncData(data) {
    const notifications = (
      await data.$axios.get(
        `user/${data.$auth.user.id}/notifications`,
        data.$auth.headers
      )
    ).data.sort((a, b) => new Date(b.created) - new Date(a.created))

    return {
      notifications,
    }
  },
  methods: {
    async performAction(notification, notificationIndex, actionIndex) {
      this.$nuxt.$loading.start()

      try {
        if (actionIndex !== null) {
          const config = {
            method: notification.actions[
              actionIndex
            ].action_route[0].toLowerCase(),
            url: `${notification.actions[actionIndex].action_route[1]}`,
            headers: {
              Authorization: this.$auth.token,
            },
          }

          await this.$axios(config)
        }

        await this.$axios.delete(
          `notification/${notification.id}`,
          this.$auth.headers
        )

        this.notifications.splice(notificationIndex, 1)
        this.$store.dispatch('user/fetchNotifications', { force: true })
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }

      this.$nuxt.$loading.finish()
    },
  },
  head: {
    title: 'Notifications - Modrinth',
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

.error {
  display: flex;
  flex-direction: column;
  width: 100%;
  justify-content: center;
  align-items: center;

  .icon {
    width: 8rem;
    height: 8rem;
    margin: 1.5rem 0;
  }

  .text {
    margin-bottom: 2rem;
    font-size: 1.25rem;
  }

  .link {
    text-decoration: underline;
  }
}
</style>
