<template>
  <div>
    <div class="section-header columns">
      <h3 class="column-grow-1">My notifications</h3>
    </div>
    <div v-if="notifications.length !== 0">
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
    </div>
    <div v-else class="error">
      <UpToDate class="icon"></UpToDate>
      <br />
      <span class="text">You are up-to-date!</span>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import UpToDate from '~/assets/images/illustrations/up_to_date.svg?inline'

export default {
  components: {
    UpToDate,
  },
  async asyncData(data) {
    const notifications = (
      await axios.get(
        `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/notifications`,
        data.$auth.headers
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
              Authorization: this.$auth.token,
            },
          }

          await axios(config)
        }

        await axios.delete(
          `https://api.modrinth.com/api/v1/notification/${notification.id}`,
          this.$auth.headers
        )

        this.notifications.splice(index, 1)
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
