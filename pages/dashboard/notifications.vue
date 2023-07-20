<template>
  <div>
    <section class="universal-card">
      <Breadcrumbs
        v-if="history"
        current-title="History"
        :link-stack="[{ href: `/dashboard/notifications`, label: 'Notifications' }]"
      />
      <div class="header__row">
        <div class="header__title">
          <h2 v-if="history">Notification history</h2>
          <h2 v-else>Notifications</h2>
        </div>
        <template v-if="!history">
          <Button v-if="allNotifs && allNotifs.some((notif) => notif.read)" @click="updateRoute()">
            <HistoryIcon /> View history
          </Button>
          <Button v-if="notifications.length > 0" color="danger" @click="readAll()">
            <CheckCheckIcon /> Mark all as read
          </Button>
        </template>
      </div>
      <template v-if="notifications.length > 0">
        <Chips
          v-if="notifTypes.length > 1"
          v-model="selectedType"
          :items="notifTypes"
          :format-label="
            (x) => (x === 'all' ? 'All' : $formatProjectType(x).replace('_', ' ') + 's')
          "
          :capitalize="false"
        />
        <NotificationItem
          v-for="notification in notifications"
          :key="notification.id"
          v-model:notifications="allNotifs"
          class="universal-card recessed"
          :notification="notification"
          :auth="auth"
          raised
        />
      </template>
      <p v-else>You don't have any unread notifications.</p>
    </section>
  </div>
</template>
<script setup>
import { Button, HistoryIcon } from 'omorphia'
import { fetchNotifications, groupNotifications, markAsRead } from '~/helpers/notifications.js'
import NotificationItem from '~/components/ui/NotificationItem.vue'
import Chips from '~/components/ui/Chips.vue'
import CheckCheckIcon from '~/assets/images/utils/check-check.svg'
import Breadcrumbs from '~/components/ui/Breadcrumbs.vue'

useHead({
  title: 'Notifications - Modrinth',
})

const auth = await useAuth()

const route = useRoute()
const router = useRouter()

const history = computed(() => {
  return route.name === 'dashboard-notifications-history'
})

const allNotifs = ref(null)

const notifTypes = computed(() => {
  if (allNotifs.value === null) {
    return []
  }
  const types = [
    ...new Set(
      allNotifs.value
        .filter((notification) => {
          return history.value || !notification.read
        })
        .map((notif) => notif.type)
    ),
  ]
  return types.length > 1 ? ['all', ...types] : types
})

const notifications = computed(() => {
  if (allNotifs.value === null) {
    return []
  }
  const groupedNotifs = groupNotifications(allNotifs.value, history.value)
  return groupedNotifs.filter(
    (notif) => selectedType.value === 'all' || notif.type === selectedType.value
  )
})

const selectedType = ref('all')

await fetchNotifications().then((result) => {
  allNotifs.value = result
})

function updateRoute() {
  if (history.value) {
    router.push('/dashboard/notifications')
  } else {
    router.push('/dashboard/notifications/history')
  }
}

async function readAll() {
  const ids = notifications.value.flatMap((notification) => [
    notification.id,
    ...(notification.grouped_notifs ? notification.grouped_notifs.map((notif) => notif.id) : []),
  ])

  const updateNotifs = await markAsRead(ids)
  allNotifs.value = updateNotifs(allNotifs.value)
}
</script>
<style lang="scss" scoped>
.read-toggle-input {
  display: flex;
  align-items: center;
  gap: var(--spacing-card-md);

  .label__title {
    margin: 0;
  }
}

.header__title {
  h2 {
    margin: 0 auto 0 0;
  }
}
</style>
