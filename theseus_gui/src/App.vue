<script setup>
import { computed, ref, watch } from 'vue'
import { RouterView, RouterLink, useRouter, useRoute } from 'vue-router'
import {
  HomeIcon,
  SearchIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  Button,
  Notifications,
  XIcon,
} from 'omorphia'
import { useLoading, useTheming } from '@/store/state'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ModrinthLoadingIndicator from '@/components/modrinth-loading-indicator'
import { useNotifications } from '@/store/notifications.js'
import { warning_listener } from '@/helpers/events.js'
import { MinimizeIcon, MaximizeIcon } from '@/assets/icons'
import { type } from '@tauri-apps/api/os'
import { appWindow } from '@tauri-apps/api/window'
import { isDev } from '@/helpers/utils.js'
import mixpanel from 'mixpanel-browser'
import { saveWindowState, StateFlags } from 'tauri-plugin-window-state-api'
import OnboardingModal from '@/components/OnboardingModal.vue'
import { getVersion } from '@tauri-apps/api/app'
import { window as TauriWindow } from '@tauri-apps/api'
import { TauriEvent } from '@tauri-apps/api/event'
import { await_sync, check_safe_loading_bars_complete } from './helpers/state'
import { confirm } from '@tauri-apps/api/dialog'

const themeStore = useTheming()

const isLoading = ref(true)
defineExpose({
  initialize: async () => {
    isLoading.value = false
    const { theme, opt_out_analytics, collapsed_navigation, advanced_rendering, onboarded } =
      await get()
    const dev = await isDev()
    const version = await getVersion()

    themeStore.setThemeState(theme)
    themeStore.collapsedNavigation = collapsed_navigation
    themeStore.advancedRendering = advanced_rendering

    mixpanel.init('014c7d6a336d0efaefe3aca91063748d', { debug: dev, persistence: 'localStorage' })
    if (opt_out_analytics) {
      mixpanel.opt_out_tracking()
    }
    mixpanel.track('Launched', { version, dev, onboarded })

    if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

    if ((await type()) === 'Darwin') {
      document.getElementsByTagName('html')[0].classList.add('mac')
    } else {
      document.getElementsByTagName('html')[0].classList.add('windows')
    }

    await warning_listener((e) =>
      notificationsWrapper.value.addNotification({
        title: 'Warning',
        text: e.message,
        type: 'warn',
      })
    )
  },
})

const confirmClose = async () => {
  const confirmed = await confirm(
    'An action is currently in progress. Are you sure you want to exit?',
    {
      title: 'Modrinth',
      type: 'warning',
    }
  )
  return confirmed
}

const handleClose = async () => {
  const isSafe = await check_safe_loading_bars_complete()
  if (!isSafe) {
    const response = await confirmClose()
    if (!response) {
      return
    }
  }
  await await_sync()
  await TauriWindow.getCurrent().close()
}

TauriWindow.getCurrent().listen(TauriEvent.WINDOW_CLOSE_REQUESTED, async () => {
  await handleClose()
})

const router = useRouter()
router.afterEach((to, from, failure) => {
  if (mixpanel.__loaded) {
    mixpanel.track('PageView', { path: to.path, fromPath: from.path, failed: failure })
  }
})
const route = useRoute()
const isOnBrowse = computed(() => route.path.startsWith('/browse'))

const loading = useLoading()

const notifications = useNotifications()
const notificationsWrapper = ref(null)

watch(notificationsWrapper, () => {
  notifications.setNotifs(notificationsWrapper.value)
})

document.querySelector('body').addEventListener('click', function (e) {
  let target = e.target
  while (target != null) {
    if (target.matches('a')) {
      if (
        target.href &&
        ['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
        !target.classList.contains('router-link-active') &&
        !target.href.startsWith('http://localhost') &&
        !target.href.startsWith('https://tauri.localhost')
      ) {
        window.__TAURI_INVOKE__('tauri', {
          __tauriModule: 'Shell',
          message: {
            cmd: 'open',
            path: target.href,
          },
        })
      }
      e.preventDefault()
      break
    }
    target = target.parentElement
  }
})

const accounts = ref(null)
</script>

<template>
  <SplashScreen v-if="isLoading" app-loading />
  <div v-else class="container">
    <suspense>
      <OnboardingModal ref="testModal" :accounts="accounts" />
    </suspense>
    <div class="nav-container">
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" mode="small" />
        </suspense>
        <div class="pages-list">
          <RouterLink to="/" class="btn icon-only collapsed-button">
            <HomeIcon />
            <span v-if="!themeStore.collapsedNavigation">Home</span>
          </RouterLink>
          <RouterLink
            to="/browse/modpack"
            class="btn"
            :class="{
              'icon-only': themeStore.collapsedNavigation,
              'collapsed-button': themeStore.collapsedNavigation,
              'expanded-button': !themeStore.collapsedNavigation,
              'router-link-active': isOnBrowse,
            }"
          >
            <SearchIcon />
            <span v-if="!themeStore.collapsedNavigation">Browse</span>
          </RouterLink>
          <RouterLink
            to="/library"
            class="btn"
            :class="{
              'icon-only': themeStore.collapsedNavigation,
              'collapsed-button': themeStore.collapsedNavigation,
              'expanded-button': !themeStore.collapsedNavigation,
            }"
          >
            <LibraryIcon />
            <span v-if="!themeStore.collapsedNavigation">Library</span>
          </RouterLink>
          <Suspense>
            <InstanceCreationModal ref="installationModal" />
          </Suspense>
        </div>
      </div>
      <div class="settings pages-list">
        <Button
          class="sleek-primary"
          :class="{
            'icon-only': themeStore.collapsedNavigation,
            'collapsed-button': themeStore.collapsedNavigation,
            'expanded-button': !themeStore.collapsedNavigation,
          }"
          @click="() => $refs.installationModal.show()"
        >
          <PlusIcon />
          <span v-if="!themeStore.collapsedNavigation" class="no-wrap">New instance</span>
        </Button>
        <RouterLink
          to="/settings"
          class="btn"
          :class="{
            'icon-only': themeStore.collapsedNavigation,
            'collapsed-button': themeStore.collapsedNavigation,
            'expanded-button': !themeStore.collapsedNavigation,
          }"
        >
          <SettingsIcon />
          <span v-if="!themeStore.collapsedNavigation">Settings</span>
        </RouterLink>
      </div>
    </div>
    <div class="view" :class="{ expanded: !themeStore.collapsedNavigation }">
      <div class="appbar-row">
        <div data-tauri-drag-region class="appbar">
          <section class="navigation-controls">
            <Breadcrumbs data-tauri-drag-region />
          </section>
          <section class="mod-stats">
            <Suspense>
              <RunningAppBar data-tauri-drag-region />
            </Suspense>
          </section>
        </div>
        <section class="window-controls">
          <Button class="titlebar-button" icon-only @click="() => appWindow.minimize()">
            <MinimizeIcon />
          </Button>
          <Button class="titlebar-button" icon-only @click="() => appWindow.toggleMaximize()">
            <MaximizeIcon />
          </Button>
          <Button
            class="titlebar-button close"
            icon-only
            @click="
              () => {
                saveWindowState(StateFlags.ALL)
                handleClose()
              }
            "
          >
            <XIcon />
          </Button>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <Notifications ref="notificationsWrapper" />
        <RouterView v-slot="{ Component }" class="main-view">
          <template v-if="Component">
            <Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
              <component :is="Component"></component>
            </Suspense>
          </template>
        </RouterView>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.sleek-primary {
  background-color: var(--color-brand-highlight);
  transition: all ease-in-out 0.1s;
}

.navigation-controls {
  flex-grow: 1;
  width: min-content;
}

.appbar-row {
  display: flex;
  flex-direction: row;
}

.window-controls {
  z-index: 20;
  display: none;
  flex-direction: row;
  align-items: center;

  .titlebar-button {
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all ease-in-out 0.1s;
    background-color: var(--color-raised-bg);
    color: var(--color-base);
    border-radius: 0;
    height: 3.25rem;

    &.close {
      &:hover,
      &:active {
        background-color: var(--color-red);
        color: var(--color-accent-contrast);
      }
    }

    &:hover,
    &:active {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
    }
  }
}

.container {
  --appbar-height: 3.25rem;
  --sidebar-width: 4.5rem;

  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    &.expanded {
      --sidebar-width: 13rem;
    }

    width: calc(100% - var(--sidebar-width));

    .appbar {
      display: flex;
      align-items: center;
      flex-grow: 1;
      background: var(--color-raised-bg);
      box-shadow: inset 0px -3px 0px black;
      text-align: center;
      padding: var(--gap-md);
      height: 3.25rem;
      gap: var(--gap-sm);
      //no select
      user-select: none;
      -webkit-user-select: none;
    }

    .router-view {
      width: 100%;
      height: calc(100% - 3.125rem);
      overflow: auto;
      overflow-x: hidden;
      background-color: var(--color-bg);
    }
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: var(--gap-md);

  &.expanded {
    --sidebar-width: 13rem;

    width: var(--sidebar-width);
    max-width: var(--sidebar-width);
    min-width: var(--sidebar-width);
  }
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  a {
    display: flex;
    align-items: center;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-base);
    box-shadow: none;

    &.router-link-active {
      color: var(--color-contrast);
      background: var(--color-button-bg);
      box-shadow: var(--shadow-floating);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }
  }

  &.primary {
    color: var(--color-accent-contrast);
    background-color: var(--color-brand);
  }
}

.collapsed-button {
  height: 3rem !important;
  width: 3rem !important;
  padding: 0.75rem;
  border-radius: var(--radius-md);
  box-shadow: none;

  svg {
    width: 1.5rem !important;
    height: 1.5rem !important;
    max-width: 1.5rem !important;
    max-height: 1.5rem !important;
  }
}

.expanded-button {
  width: 100%;
  padding: var(--gap-md) var(--gap-lg);
}

.instance-list {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 70%;
  margin: 0.4rem;

  p:nth-child(1) {
    font-size: 0.6rem;
  }

  & > p {
    color: var(--color-base);
    margin: 0.8rem 0;
    font-size: 0.7rem;
    line-height: 0.8125rem;
    font-weight: 500;
    text-transform: uppercase;
  }
}

.user-section {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 4.375rem;

  section {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    text-align: left;
    margin-left: 0.5rem;
  }

  .username {
    margin-bottom: 0.3rem;
    font-weight: 400;
    line-height: 1.25rem;
    color: var(--color-contrast);
  }

  a {
    font-weight: 400;
    color: var(--color-secondary);
  }
}

.nav-section {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 100%;
  gap: 1rem;
}
</style>
