<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="content">
        <h1>Notifications</h1>

        <div class="divider card">
          <ThisOrThat
            v-model="selectedNotificationType"
            :items="notificationTypes"
          />
          <button class="iconified-button" @click="clearNotifications">
            <ClearIcon />
            Clear all
          </button>
        </div>
        <div class="notifications">
          <div
            v-for="notification in selectedNotificationType !== 'all'
              ? $user.notifications.filter(
                  (x) => x.type === NOTIFICATION_TYPES[selectedNotificationType]
                )
              : $user.notifications"
            :key="notification.id"
            class="card notification"
          >
            <div class="icon">
              <UpdateIcon v-if="notification.type === 'project_update'" />
              <UsersIcon v-else-if="notification.type === 'team_invite'" />
            </div>
            <div class="text">
              <nuxt-link :to="notification.link" class="top">
                <h3 v-html="$xss($md.render(notification.title))" />
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
                v-if="notification.actions.length === 0"
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
import ClearIcon from '~/assets/images/utils/clear.svg?inline'
import UpdateIcon from '~/assets/images/utils/updated.svg?inline'
import UsersIcon from '~/assets/images/utils/users.svg?inline'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'
import ThisOrThat from '~/components/ui/ThisOrThat'

const NOTIFICATION_TYPES = {
  'Team Invites': 'team_invite',
  'Project Updates': 'project_update',
}

export default {
  name: 'Notifications',
  components: {
    ThisOrThat,
    ClearIcon,
    UpdateIcon,
    UsersIcon,
    UpToDate,
  },
  data() {
    return {
      selectedNotificationType: 'all',
    }
  },
  async fetch() {
    await this.$store.dispatch('user/fetchNotifications')
  },
  head: {
    title: 'Notifications - Modrinth',
  },
  computed: {
    notificationTypes() {
      const obj = { all: true }

      for (const notification of this.$user.notifications.filter(
        (it) => it.type !== null
      )) {
        obj[
          Object.keys(NOTIFICATION_TYPES).find(
            (key) => NOTIFICATION_TYPES[key] === notification.type
          )
        ] = true
      }

      return Object.keys(obj)
    },
  },
  created() {
    this.NOTIFICATION_TYPES = NOTIFICATION_TYPES
  },
  methods: {
    async clearNotifications() {
      try {
        const ids = this.$user.notifications.map((x) => x.id)

        await this.$axios.delete(
          `notifications?ids=${JSON.stringify(ids)}`,
          this.$defaultHeaders()
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
          this.$defaultHeaders()
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
  align-items: center;
  display: flex;
  justify-content: space-between;
  flex-wrap: wrap;
  row-gap: 0.5rem;
}

.notifications {
  .notification {
    display: flex;
    flex-wrap: wrap;
    padding: var(--spacing-card-sm) var(--spacing-card-lg);

    .icon {
      display: flex;
      flex-direction: column;
      justify-content: center;

      svg {
        height: calc(3rem - var(--spacing-card-sm));
        width: auto;
        margin-right: 1rem;
      }
    }

    .text {
      display: flex;
      flex-direction: column;
      justify-content: space-between;

      .top {
        display: flex;
        align-items: baseline;
        flex-direction: column;

        h3 ::v-deep {
          font-size: var(--font-size-lg);
          margin: 0 0.5rem 0 0;

          p {
            margin: 0;

            strong {
              color: var(--color-brand);
            }
          }
        }
      }

      p {
        padding: 0;
        margin: 0;
      }
    }

    .buttons {
      display: flex;
      flex-direction: column;
      justify-content: center;
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

  .notifications {
    .notification {
      flex-wrap: nowrap;

      .text {
        flex-direction: column;

        .top {
          flex-direction: row;
        }
      }
    }
  }
}
</style>
