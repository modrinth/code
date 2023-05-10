<script setup>
import { useNotifications } from '@/store/notifications'

const notificationStore = useNotifications()

const stopTimer = (notif) => clearTimeout(notif.timer)
</script>

<template>
  <div class="vue-notification-group">
    <TransitionGroup name="notifs">
      <div
        v-for="item in notificationStore.notifications"
        :key="item.id"
        class="vue-notification-wrapper"
        @click="() => notificationStore.clearNotificationById(item.id)"
        @mouseenter="stopTimer(item)"
        @mouseleave="notificationStore.setNotificationTimer(item)"
      >
        <div class="vue-notification-template vue-notification" :class="{ [item.type]: true }">
          <div class="notification-title" v-html="item.title"></div>
          <div class="notification-content" v-html="item.text"></div>
        </div>
      </div>
    </TransitionGroup>
  </div>
</template>

<style lang="scss" scoped>
.vue-notification {
  background: var(--color-blue) !important;
  border-left: 5px solid var(--color-blue) !important;
  color: var(--color-contrast) !important;
  box-sizing: border-box;
  text-align: left;
  font-size: 12px;
  padding: 10px;
  margin: 0 5px 5px;
  cursor: pointer;

  &.success {
    background: var(--color-green) !important;
    border-left-color: var(--color-green) !important;
  }
  &.warn {
    background: var(--color-orange) !important;
    border-left-color: var(--color-orange) !important;
  }
  &.error {
    background: var(--color-red) !important;
    border-left-color: var(--color-red) !important;
  }
}
.vue-notification-group {
  position: fixed;
  right: 25px;
  bottom: 25px;
  z-index: 99999999;
  width: 300px;
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
        font-size: var(--font-size-xs);
      }
    }
    &:last-child {
      margin: 0;
    }
  }
}
.notifs-enter-active,
.notifs-leave-active,
.notifs-move {
  transition: all 0.5s;
}
.notifs-enter-from,
.notifs-leave-to {
  opacity: 0;
}
</style>
