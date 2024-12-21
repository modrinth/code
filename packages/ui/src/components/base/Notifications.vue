<template>
  <div class="vue-notification-group" :class="{ 'has-sidebar': sidebar }">
    <transition-group name="notifs">
      <div
        v-for="(item, index) in notifications"
        :key="item.id"
        class="vue-notification-wrapper"
        @click="notifications.splice(index, 1)"
        @mouseenter="stopTimer(item)"
        @mouseleave="setNotificationTimer(item)"
      >
        <div class="vue-notification-template vue-notification" :class="{ [item.type]: true }">
          <div class="notification-title" v-html="item.title"></div>
          <div class="notification-content" v-html="item.text"></div>
        </div>
      </div>
    </transition-group>
  </div>
</template>
<script setup>
import { ref } from 'vue'

defineProps({
  sidebar: {
    type: Boolean,
    default: false,
  },
})

const notifications = ref([])

defineExpose({
  addNotification: (notification) => {
    const existingNotif = notifications.value.find(
      (x) =>
        x.text === notification.text &&
        x.title === notification.title &&
        x.type === notification.type,
    )
    if (existingNotif) {
      setNotificationTimer(existingNotif)

      return
    }

    notification.id = new Date()

    setNotificationTimer(notification)
    notifications.value.push(notification)
  },
})

function setNotificationTimer(notification) {
  if (!notification) return

  if (notification.timer) {
    clearTimeout(notification.timer)
  }

  notification.timer = setTimeout(() => {
    notifications.value.splice(notifications.value.indexOf(notification), 1)
  }, 30000)
}

function stopTimer(notif) {
  clearTimeout(notif.timer)
}
</script>
<style lang="scss" scoped>
.vue-notification {
  background: var(--color-blue) !important;
  color: var(--color-accent-contrast) !important;

  box-sizing: border-box;
  text-align: left;
  font-size: 12px;
  padding: 10px;
  margin: 0 5px 5px;

  &.success {
    background: var(--color-green) !important;
  }

  &.warn {
    background: var(--color-orange) !important;
  }

  &.error {
    background: var(--color-red) !important;
  }
}

.vue-notification-group {
  position: fixed;
  right: 25px;
  bottom: 25px;
  z-index: 99999999;
  width: 300px;

  &.has-sidebar {
    right: 325px;
  }

  .vue-notification-wrapper {
    width: 100%;
    overflow: hidden;
    margin-bottom: 10px;

    .vue-notification-template {
      border-radius: var(--radius-md);
      margin: 0;

      .notification-title {
        font-size: var(--font-size-lg);
        margin-right: auto;
        font-weight: 600;
      }

      .notification-content {
        margin-right: auto;
        font-size: var(--font-size-md);
      }
    }

    &:last-child {
      margin: 0;
    }
  }

  @media screen and (max-width: 750px) {
    bottom: calc(var(--size-mobile-navbar-height, 15px) + 10px) !important;

    &.browse-menu-open {
      bottom: calc(var(--size-mobile-navbar-height-expanded, 15px) + 10px) !important;
    }
  }
}

.notifs-enter-active,
.notifs-leave-active,
.notifs-move {
  transition: all 0.5s;

  @media (prefers-reduced-motion) {
    transition: none !important;
  }
}
.notifs-enter-from,
.notifs-leave-to {
  opacity: 0;
}
</style>
