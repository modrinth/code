<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="content">
        <h1>Notifications</h1>

        <div class="divider card">
          <button class="iconified-button" @click="clearNotifications">
            <ClearIcon />
            Clear all
          </button>
        </div>
        <div class="notifications">
          <div
            v-for="notification in $user.notifications"
            :key="notification.id"
            class="card notification"
          >
            <div class="icon">
              <UpdateIcon v-if="notification.type === 'project-update'" />
              <UsersIcon v-else-if="notification.type === 'team_invite'" />
            </div>
            <div class="text">
              <nuxt-link :to="notification.link" class="top">
                <h3>{{ notification.title }}</h3>
                <span>
                  Notified {{ $dayjs(notification.created).fromNow() }}</span
                >
              </nuxt-link>
              <p>{{ notification.text }}</p>
            </div>
            <div class="buttons">
              <button
                v-for="(action, actionIndex) in notification.actions"
                :key="actionIndex"
                class="iconified-button"
                @click="
                  performAction(notification, notificationIndex, actionIndex)
                "
              >
                {{ action.title }}
              </button>
              <button
                v-if="$user.notifications.length === 0"
                class="iconified-button"
                @click="performAction(notification, notificationIndex, null)"
              >
                Dismiss
              </button>
            </div>
          </div>
          <div v-if="$user.notifications.length === 0" class="error">
            <UpToDate class="icon"></UpToDate>
            <br />
            <span class="text">You are up-to-date!</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import ClearIcon from '~/assets/images/utils/trash.svg?inline'
import UpdateIcon from '~/assets/images/utils/updated.svg?inline'
import UsersIcon from '~/assets/images/utils/users.svg?inline'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'

export default {
  name: 'Notifications',
  components: {
    ClearIcon,
    UpdateIcon,
    UsersIcon,
    UpToDate,
  },
  async fetch() {
    await this.$store.dispatch('user/fetchNotifications')
  },
  head: {
    title: 'Notifications - Modrinth',
  },
  methods: {
    async clearNotifications() {
      try {
        const ids = this.$user.notifications.map((x) => x.id)

        await this.$axios.delete(
          `notifications?ids=${JSON.stringify(ids)}`,
          this.$auth.headers
        )

        ids.forEach((x) => this.$store.dispatch('user/deleteNotification', x))
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.response.data.description,
          type: 'error',
        })
      }
    },
    async performAction(notification, notificationIndex, actionIndex) {
      this.$nuxt.$loading.start()
      try {
        await this.$axios.delete(
          `notification/${notification.id}`,
          this.$auth.headers
        )

        await this.$store.dispatch('user/deleteNotification', notification.id)

        if (actionIndex !== null) {
          const config = {
            method:
              notification.actions[actionIndex].action_route[0].toLowerCase(),
            url: `${notification.actions[actionIndex].action_route[1]}`,
            headers: {
              Authorization: this.$auth.token,
            },
          }
          await this.$axios(config)
        }
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
}
</script>

<style lang="scss" scoped>
h1 {
  color: var(--color-text-dark);
  margin: 0 0 1rem 1.5rem;
}

.divider {
  button {
    margin-left: auto;
  }
}

.notifications {
  .notification {
    display: flex;
    max-height: 4rem;
    padding: var(--spacing-card-sm) var(--spacing-card-lg);

    .icon svg {
      height: calc(4rem - var(--spacing-card-sm));
      width: auto;
      margin-right: 1rem;
    }

    .text {
      max-height: calc(4rem - var(--spacing-card-sm));
      display: flex;
      flex-direction: column;
      justify-content: space-between;

      .top {
        display: flex;
        align-items: baseline;

        h3 {
          font-size: var(--font-size-lg);
          margin: 0 0.5rem 0 0;

          strong {
            color: var(--color-brand);
          }
        }
      }

      p {
        padding: 0;
        margin: 0;
      }
    }

    .buttons {
      margin-left: auto;
      text-align: right;

      button {
        margin-left: auto;
        margin-bottom: 0.25rem;
      }
    }
  }
}

@media screen and (min-width: 1024px) {
  .page-contents {
    max-width: calc(1280px - 20rem) !important;
  }
}
</style>
