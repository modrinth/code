<template>
  <div class="vue-notification-group experimental-styles-within">
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
                <button v-tooltip="`Dismiss`" @click="notifications.splice(index, 1)">
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
<script setup>
import { ButtonStyled } from "@modrinth/ui";
import {
  XCircleIcon,
  CheckCircleIcon,
  CheckIcon,
  InfoIcon,
  IssuesIcon,
  XIcon,
  CopyIcon,
} from "@modrinth/assets";
const notifications = useNotifications();

function stopTimer(notif) {
  clearTimeout(notif.timer);
}

const copied = ref({});

const createNotifText = (notif) => {
  let text = "";
  if (notif.title) {
    text += notif.title;
  }
  if (notif.text) {
    if (text.length > 0) {
      text += "\n";
    }
    text += notif.text;
  }
  if (notif.errorCode) {
    if (text.length > 0) {
      text += "\n";
    }
    text += notif.errorCode;
  }
  return text;
};

function copyToClipboard(notif) {
  const text = createNotifText(notif);

  copied.value[text] = true;
  navigator.clipboard.writeText(text);
  setTimeout(() => {
    delete copied.value[text];
  }, 2000);
}
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
