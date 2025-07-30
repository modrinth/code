<template>
  <div
    class="vue-notification-group experimental-styles-within"
    :class="{ 'intercom-present': isIntercomPresent }"
  >
    <transition-group name="notifs">
      <div
        v-for="(item, index) in notifications"
        :key="item.id"
        class="vue-notification-wrapper"
        @mouseenter="stopTimer(item)"
        @mouseleave="setNotificationTimer(item)"
      >
        <div class="flex w-full gap-2 overflow-hidden rounded-lg bg-bg-raised shadow-xl">
          <div
            class="w-2"
            :class="{
              'bg-red': item.type === 'error',
              'bg-orange': item.type === 'warning',
              'bg-green': item.type === 'success',
              'bg-blue': !item.type || !['error', 'warning', 'success'].includes(item.type),
            }"
          ></div>
          <div
            class="grid w-full grid-cols-[auto_1fr_auto] items-center gap-x-2 gap-y-1 py-2 pl-1 pr-3"
          >
            <div
              class="flex items-center"
              :class="{
                'text-red': item.type === 'error',
                'text-orange': item.type === 'warning',
                'text-green': item.type === 'success',
                'text-blue': !item.type || !['error', 'warning', 'success'].includes(item.type),
              }"
            >
              <IssuesIcon v-if="item.type === 'warning'" class="h-6 w-6" />
              <CheckCircleIcon v-else-if="item.type === 'success'" class="h-6 w-6" />
              <XCircleIcon v-else-if="item.type === 'error'" class="h-6 w-6" />
              <InfoIcon v-else class="h-6 w-6" />
            </div>
            <div class="m-0 text-wrap font-bold text-contrast" v-html="item.title"></div>
            <div class="flex items-center gap-1">
              <div v-if="item.count && item.count > 1" class="text-xs font-bold text-contrast">
                x{{ item.count }}
              </div>
              <ButtonStyled circular size="small">
                <button v-tooltip="'Copy to clipboard'" @click="copyToClipboard(item)">
                  <CheckIcon v-if="copied[createNotifText(item)]" />
                  <CopyIcon v-else />
                </button>
              </ButtonStyled>
              <ButtonStyled circular size="small">
                <button v-tooltip="`Dismiss`" @click="dismissNotification(index)">
                  <XIcon />
                </button>
              </ButtonStyled>
            </div>
            <div></div>
            <div class="col-span-2 text-sm text-primary" v-html="item.text"></div>
            <template v-if="item.errorCode">
              <div></div>
              <div
                class="m-0 text-wrap text-xs font-medium text-secondary"
                v-html="item.errorCode"
              ></div>
            </template>
          </div>
        </div>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { ButtonStyled, injectNotificationManager, type WebNotification } from '@modrinth/ui'
import {
  XCircleIcon,
  CheckCircleIcon,
  CheckIcon,
  InfoIcon,
  IssuesIcon,
  XIcon,
  CopyIcon,
} from '@modrinth/assets'

const notificationManager = injectNotificationManager()
const notifications = computed<WebNotification[]>(() => notificationManager.getNotifications())

const isIntercomPresent = ref<boolean>(false)
const copied = ref<Record<string, boolean>>({})

const stopTimer = (n: WebNotification) => notificationManager.stopNotificationTimer(n)
const setNotificationTimer = (n: WebNotification) => notificationManager.setNotificationTimer(n)
const dismissNotification = (n: number) => notificationManager.removeNotificationByIndex(n)

function createNotifText(notif: WebNotification): string {
  return [notif.title, notif.text, notif.errorCode].filter(Boolean).join('\n')
}

function checkIntercomPresence(): void {
  isIntercomPresent.value = !!document.querySelector('.intercom-lightweight-app')
}

function copyToClipboard(notif: WebNotification): void {
  const text = createNotifText(notif)

  copied.value[text] = true
  navigator.clipboard.writeText(text)

  setTimeout(() => {
    delete copied.value[text]
  }, 2000)
}

onMounted(() => {
  checkIntercomPresence()

  const observer = new MutationObserver(() => {
    checkIntercomPresence()
  })

  observer.observe(document.body, {
    childList: true,
    subtree: true,
  })

  onBeforeUnmount(() => {
    observer.disconnect()
  })
})
</script>

<style lang="scss" scoped>
.vue-notification-group {
  position: fixed;
  right: 1.5rem;
  bottom: 1.5rem;
  z-index: 200;
  width: 450px;

  @media screen and (max-width: 500px) {
    width: calc(100% - 0.75rem * 2);
    right: 0.75rem;
    bottom: 0.75rem;
  }

  &.intercom-present {
    bottom: 5rem;
  }

  .vue-notification-wrapper {
    width: 100%;
    overflow: hidden;
    margin-bottom: 10px;

    &:last-child {
      margin: 0;
    }
  }

  @media screen and (max-width: 750px) {
    transition: bottom 0.25s ease-in-out;
    bottom: calc(var(--size-mobile-navbar-height) + 10px) !important;

    &.browse-menu-open {
      bottom: calc(var(--size-mobile-navbar-height-expanded) + 10px) !important;
    }
  }
}

.notifs-enter-active,
.notifs-leave-active,
.notifs-move {
  transition: all 0.25s ease-in-out;
}
.notifs-enter-from,
.notifs-leave-to {
  opacity: 0;
}

.notifs-enter-from {
  transform: translateY(100%) scale(0.8);
}

.notifs-leave-to {
  transform: translateX(100%) scale(0.8);
}
</style>
