<script setup>
import { computed, ref, onMounted, watch, onUnmounted } from 'vue'
import { RouterView, useRouter, useRoute } from 'vue-router'
import {
  ArrowBigUpDashIcon,
  LogInIcon,
  HomeIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  XIcon,
  DownloadIcon,
  CompassIcon,
  MinimizeIcon,
  MaximizeIcon,
  RestoreIcon,
  LogOutIcon,
} from '@modrinth/assets'
import { Avatar, Button, ButtonStyled, Notifications, OverflowMenu } from '@modrinth/ui'
import { useLoading, useTheming } from '@/store/state'
import ModrinthAppLogo from '@/assets/modrinth_app.svg?component'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import ModrinthLoadingIndicator from '@/components/LoadingIndicatorBar.vue'
import { handleError, useNotifications } from '@/store/notifications.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { type } from '@tauri-apps/plugin-os'
import { isDev, getOS, restartApp } from '@/helpers/utils.js'
import { initAnalytics, debugAnalytics, optOutAnalytics, trackEvent } from '@/helpers/analytics'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { getVersion } from '@tauri-apps/api/app'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import { install_from_file } from './helpers/pack'
import { useError } from '@/store/error.js'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import { useInstall } from '@/store/install.js'
import { invoke } from '@tauri-apps/api/core'
import { get_opening_command, initialize_state } from '@/helpers/state'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { renderString } from '@modrinth/utils'
import { useFetch } from '@/helpers/fetch.js'
import { check } from '@tauri-apps/plugin-updater'
import NavButton from '@/components/ui/NavButton.vue'
import { get as getCreds, logout, login } from '@/helpers/mr_auth.js'
import { get_user } from '@/helpers/cache.js'
import AppSettingsModal from '@/components/ui/modal/AppSettingsModal.vue'
import dayjs from 'dayjs'
import PromotionWrapper from '@/components/ui/PromotionWrapper.vue'
import { hide_ads_window, show_ads_window } from '@/helpers/ads.js'
import FriendsList from '@/components/ui/friends/FriendsList.vue'
import { openUrl } from '@tauri-apps/plugin-opener'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'

const themeStore = useTheming()

const news = ref([
  {
    title: 'Introducing Modrinth Servers',
    summary: 'Host your next Minecraft server with Modrinth.',
    thumbnail:
      'https://media.beehiiv.com/cdn-cgi/image/format=auto,width=800,height=421,fit=scale-down,onerror=redirect/uploads/asset/file/eefddc59-b4c4-4e7d-92e8-c26bdef42984/Modrinth-Servers-Thumb.png',
    date: '2024-11-02T00:00:00Z',
    link: 'https://blog.modrinth.com/p/modrinth-servers-beta',
  },
  {
    title: 'Becoming Sustainable',
    summary: 'Announcing 5x creator revenue and updates to the monetization program.',
    thumbnail:
      'https://media.beehiiv.com/cdn-cgi/image/format=auto,width=800,height=421,fit=scale-down,onerror=redirect/uploads/asset/file/c99b9885-8248-4d7a-b19a-3ae2c902fdd5/revenue.png',
    date: '2024-09-13T00:00:00Z',
    link: 'https://blog.modrinth.com/p/creator-revenue-update',
  },
  {
    title: 'Modrinth+ and New Ads',
    summary:
      'Introducing a new advertising system, a subscription to remove ads, and a redesign of the website!\n',
    thumbnail:
      'https://media.beehiiv.com/cdn-cgi/image/fit=scale-down,format=auto,onerror=redirect,quality=80/uploads/asset/file/38ce85e4-5d93-43eb-b61b-b6296f6b9e66/things.png?t=1724260059',
    date: '2024-08-21T00:00:00Z',
    link: 'https://blog.modrinth.com/p/introducing-modrinth-refreshed-site-look-new-advertising-system',
  },
])

const urlModal = ref(null)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const os = ref('')

const stateInitialized = ref(false)

const criticalErrorMessage = ref()

const isMaximized = ref(false)

onMounted(async () => {
  await useCheckDisableMouseover()

  document.querySelector('body').addEventListener('click', handleClick)
  document.querySelector('body').addEventListener('auxclick', handleAuxClick)
})

onUnmounted(() => {
  document.querySelector('body').removeEventListener('click', handleClick)
  document.querySelector('body').removeEventListener('auxclick', handleAuxClick)
})

async function setupApp() {
  stateInitialized.value = true
  const {
    native_decorations,
    theme,
    telemetry,
    collapsed_navigation,
    advanced_rendering,
    onboarded,
    default_page,
  } = await get()

  if (default_page === 'Library') {
    await router.push('/library')
  }

  os.value = await getOS()
  const dev = await isDev()
  const version = await getVersion()
  showOnboarding.value = !onboarded

  nativeDecorations.value = native_decorations
  if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

  themeStore.setThemeState(theme)
  themeStore.collapsedNavigation = collapsed_navigation
  themeStore.advancedRendering = advanced_rendering

  isMaximized.value = await getCurrentWindow().isMaximized()

  await getCurrentWindow().onResized(async () => {
    isMaximized.value = await getCurrentWindow().isMaximized()
  })

  initAnalytics()
  if (!telemetry) {
    optOutAnalytics()
  }
  if (dev) debugAnalytics()
  trackEvent('Launched', { version, dev, onboarded })

  if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

  const osType = await type()
  if (osType === 'macos') {
    document.getElementsByTagName('html')[0].classList.add('mac')
  } else {
    document.getElementsByTagName('html')[0].classList.add('windows')
  }

  await warning_listener((e) =>
    notificationsWrapper.value.addNotification({
      title: 'Warning',
      text: e.message,
      type: 'warn',
    }),
  )

  useFetch(
    `https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
    'criticalAnnouncements',
    true,
  ).then((res) => {
    if (res && res.header && res.body) {
      criticalErrorMessage.value = res
    }
  })

  get_opening_command().then(handleCommand)
  checkUpdates()
  fetchCredentials()
}

const stateFailed = ref(false)
initialize_state()
  .then(() => {
    setupApp().catch((err) => {
      stateFailed.value = true
      console.error(err)
      error.showError(err, null, false, 'state_init')
    })
  })
  .catch((err) => {
    stateFailed.value = true
    console.error('Failed to initialize app', err)
    error.showError(err, null, false, 'state_init')
  })

const handleClose = async () => {
  await saveWindowState(StateFlags.ALL)
  await getCurrentWindow().close()
}

const router = useRouter()
router.afterEach((to, from, failure) => {
  trackEvent('PageView', { path: to.path, fromPath: from.path, failed: failure })
})
const route = useRoute()

const loading = useLoading()
loading.setEnabled(false)

const notifications = useNotifications()
const notificationsWrapper = ref()

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

const credentials = ref()

async function fetchCredentials() {
  const creds = await getCreds().catch(handleError)
  if (creds && creds.user_id) {
    creds.user = await get_user(creds.user_id).catch(handleError)
  }
  credentials.value = creds
}

async function signIn() {
  await login().catch(handleError)
  await fetchCredentials()
}

async function logOut() {
  await logout().catch(handleError)
  await fetchCredentials()
}

const MIDAS_BITFLAG = 1 << 0
const hasPlus = computed(
  () =>
    credentials.value &&
    credentials.value.user &&
    (credentials.value.user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG,
)

watch(
  hasPlus,
  () => {
    if (hasPlus.value) {
      hide_ads_window(true)
    } else {
      show_ads_window()
    }
  },
  { immediate: true },
)

onMounted(() => {
  invoke('show_window')

  notifications.setNotifs(notificationsWrapper.value)

  error.setErrorModal(errorModal.value)

  install.setIncompatibilityWarningModal(incompatibilityWarningModal)
  install.setInstallConfirmModal(installConfirmModal)
  install.setModInstallModal(modInstallModal)
})

const accounts = ref(null)

command_listener(handleCommand)
async function handleCommand(e) {
  if (!e) return

  if (e.event === 'RunMRPack') {
    // RunMRPack should directly install a local mrpack given a path
    if (e.path.endsWith('.mrpack')) {
      await install_from_file(e.path).catch(handleError)
      trackEvent('InstanceCreate', {
        source: 'CreationModalFileDrop',
      })
    }
  } else {
    // Other commands are URL-based (deep linking)
    urlModal.value.show(e)
  }
}

const updateAvailable = ref(false)
async function checkUpdates() {
  const update = await check()
  updateAvailable.value = !!update

  setTimeout(
    () => {
      checkUpdates()
    },
    5 * 1000 * 60,
  )
}

function handleClick(e) {
  let target = e.target
  while (target != null) {
    if (target.matches('a')) {
      if (
        target.href &&
        ['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
        !target.classList.contains('router-link-active') &&
        !target.href.startsWith('http://localhost') &&
        !target.href.startsWith('https://tauri.localhost') &&
        !target.href.startsWith('http://tauri.localhost')
      ) {
        openUrl(target.href)
      }
      e.preventDefault()
      break
    }
    target = target.parentElement
  }
}

function handleAuxClick(e) {
  // disables middle click -> new tab
  if (e.button === 1) {
    e.preventDefault()
    // instead do a left click
    const event = new MouseEvent('click', {
      view: window,
      bubbles: true,
      cancelable: true,
    })
    e.target.dispatchEvent(event)
  }
}
</script>

<template>
  <SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
  <div id="teleports"></div>
  <div v-if="stateInitialized" class="app-grid-layout relative">
    <Suspense>
      <AppSettingsModal ref="settingsModal" />
    </Suspense>
    <Suspense>
      <InstanceCreationModal ref="installationModal" />
    </Suspense>
    <div
      class="app-grid-navbar bg-bg-raised flex flex-col p-[1rem] pt-0 gap-[0.5rem] z-10 w-[--left-bar-width]"
    >
      <NavButton to="/">
        <HomeIcon />
        <template #label>Home</template>
      </NavButton>
      <NavButton
        to="/browse/modpack"
        :is-primary="() => route.path.startsWith('/browse') && !route.query.i"
        :is-subpage="(route) => route.path.startsWith('/project') && !route.query.i"
      >
        <CompassIcon />
        <template #label>Discover content</template>
      </NavButton>
      <NavButton
        to="/library"
        :is-subpage="
          () =>
            route.path.startsWith('/instance') ||
            ((route.path.startsWith('/browse') || route.path.startsWith('/project')) &&
              route.query.i)
        "
      >
        <LibraryIcon />
        <template #label>Library</template>
      </NavButton>
      <div class="h-px w-6 mx-auto my-2 bg-button-bg"></div>
      <suspense>
        <QuickInstanceSwitcher />
      </suspense>
      <NavButton :to="() => $refs.installationModal.show()" :disabled="offline">
        <PlusIcon />
        <template #label>Create new instance</template>
      </NavButton>
      <div class="flex flex-grow"></div>
      <NavButton v-if="updateAvailable" :to="() => restartApp()">
        <DownloadIcon />
        <template #label>Install update</template>
      </NavButton>
      <NavButton :to="() => $refs.settingsModal.show()">
        <SettingsIcon />
        <template #label>Settings</template>
      </NavButton>
      <ButtonStyled v-if="credentials" type="transparent" circular>
        <OverflowMenu
          :options="[
            {
              id: 'sign-out',
              action: () => logOut(),
              color: 'danger',
            },
          ]"
          direction="left"
        >
          <Avatar
            :src="credentials.user.avatar_url"
            :alt="credentials.user.username"
            size="32px"
            circle
          />
          <template #sign-out> <LogOutIcon /> Sign out </template>
        </OverflowMenu>
      </ButtonStyled>
      <NavButton v-else :to="() => signIn()">
        <LogInIcon />
        <template #label>Sign in</template>
      </NavButton>
    </div>
    <div data-tauri-drag-region class="app-grid-statusbar bg-bg-raised h-[--top-bar-height] flex">
      <div data-tauri-drag-region class="flex p-4">
        <ModrinthAppLogo class="h-full w-auto text-contrast pointer-events-none" />
        <Breadcrumbs />
      </div>
      <section class="flex ml-auto">
        <div class="flex mr-3">
          <Suspense>
            <RunningAppBar />
          </Suspense>
        </div>
        <section v-if="!nativeDecorations" class="window-controls">
          <Button class="titlebar-button" icon-only @click="() => getCurrentWindow().minimize()">
            <MinimizeIcon />
          </Button>
          <Button
            class="titlebar-button"
            icon-only
            @click="() => getCurrentWindow().toggleMaximize()"
          >
            <RestoreIcon v-if="isMaximized" />
            <MaximizeIcon v-else />
          </Button>
          <Button class="titlebar-button close" icon-only @click="handleClose">
            <XIcon />
          </Button>
        </section>
      </section>
    </div>
  </div>
  <div v-if="stateInitialized" class="app-contents experimental-styles-within">
    <div class="app-viewport flex-grow router-view">
      <div
        class="loading-indicator-container h-8 fixed z-50"
        :style="{
          top: 'calc(var(--top-bar-height))',
          left: 'calc(var(--left-bar-width))',
          width: 'calc(100% - var(--left-bar-width) - var(--right-bar-width))',
        }"
      >
        <ModrinthLoadingIndicator />
      </div>
      <div
        v-if="themeStore.featureFlag_pagePath"
        class="absolute bottom-0 left-0 m-2 bg-tooltip-bg text-tooltip-text font-semibold rounded-full px-2 py-1 text-xs z-50"
      >
        {{ route.fullPath }}
      </div>
      <div
        id="background-teleport-target"
        class="absolute h-full -z-10 rounded-tl-[--radius-xl] overflow-hidden"
        :style="{
          width: 'calc(100% - var(--right-bar-width))',
        }"
      ></div>
      <RouterView v-slot="{ Component }">
        <template v-if="Component">
          <Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
            <component :is="Component"></component>
          </Suspense>
        </template>
      </RouterView>
    </div>
    <div
      class="app-sidebar mt-px shrink-0 flex flex-col border-0 border-l-[1px] border-[--brand-gradient-border] border-solid overflow-auto"
      :class="{ 'has-plus': hasPlus }"
    >
      <div
        class="app-sidebar-scrollable flex-grow shrink overflow-y-auto relative"
        :class="{ 'pb-12': !hasPlus }"
      >
        <div id="sidebar-teleport-target" class="sidebar-teleport-content"></div>
        <div class="sidebar-default-content">
          <div class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid">
            <h3 class="text-lg m-0">Playing as</h3>
            <suspense>
              <AccountsCard ref="accounts" mode="small" />
            </suspense>
          </div>
          <div class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid">
            <suspense>
              <FriendsList :credentials="credentials" :sign-in="() => signIn()" />
            </suspense>
          </div>
          <div class="pt-4 flex flex-col">
            <h3 class="px-4 text-lg m-0">News</h3>
            <template v-for="(item, index) in news" :key="`news-${index}`">
              <a
                :class="`flex flex-col outline-offset-[-4px] hover:bg-[--brand-gradient-border] focus:bg-[--brand-gradient-border] px-4 transition-colors ${index === 0 ? 'pt-2 pb-4' : 'py-4'}`"
                :href="item.link"
                target="_blank"
                rel="external"
              >
                <img
                  :src="item.thumbnail"
                  alt="News thumbnail"
                  aria-hidden="true"
                  class="w-full aspect-[3/1] object-cover rounded-2xl border-[1px] border-solid border-[--brand-gradient-border]"
                />
                <h4 class="mt-2 mb-0 text-sm leading-none text-contrast font-semibold">
                  {{ item.title }}
                </h4>
                <p class="my-1 text-sm text-secondary leading-tight">{{ item.summary }}</p>
                <p class="text-right text-sm text-secondary opacity-60 leading-tight m-0">
                  {{ dayjs(item.date).fromNow() }}
                </p>
              </a>
              <hr
                v-if="index !== news.length - 1"
                class="h-px my-[-2px] mx-4 border-0 m-0 bg-[--brand-gradient-border]"
              />
            </template>
          </div>
        </div>
      </div>
      <template v-if="!hasPlus">
        <a
          href="https://modrinth.plus?app"
          class="absolute bottom-[250px] w-full flex justify-center items-center gap-1 px-4 py-3 text-purple font-medium hover:underline z-10"
          target="_blank"
        >
          <ArrowBigUpDashIcon class="text-2xl" /> Upgrade to Modrinth+
        </a>
        <PromotionWrapper />
      </template>
    </div>
    <div class="view">
      <div v-if="criticalErrorMessage" class="critical-error-banner" data-tauri-drag-region>
        <h1>{{ criticalErrorMessage.header }}</h1>
        <div class="markdown-body" v-html="renderString(criticalErrorMessage.body ?? '')"></div>
      </div>
    </div>
  </div>
  <URLConfirmModal ref="urlModal" />
  <Notifications ref="notificationsWrapper" sidebar />
  <ErrorModal ref="errorModal" />
  <ModInstallModal ref="modInstallModal" />
  <IncompatibilityWarningModal ref="incompatibilityWarningModal" />
  <InstallConfirmModal ref="installConfirmModal" />
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
    background-color: transparent;
    color: var(--color-base);
    height: 100%;
    width: 3rem;
    position: relative;
    box-shadow: none;

    &:last-child {
      padding-right: 0.75rem;
      width: 3.75rem;
    }

    svg {
      width: 1.25rem;
      height: 1.25rem;
    }

    &::before {
      content: '';
      border-radius: 999999px;
      width: 3rem;
      height: 3rem;
      aspect-ratio: 1 / 1;
      margin-block: auto;
      position: absolute;
      background-color: transparent;
      scale: 0.9;
      transition: all ease-in-out 0.2s;
      z-index: -1;
    }

    &.close {
      &:hover,
      &:active {
        color: var(--color-accent-contrast);

        &::before {
          background-color: var(--color-red);
        }
      }
    }

    &:hover,
    &:active {
      color: var(--color-contrast);

      &::before {
        background-color: var(--color-button-bg);
        scale: 1;
      }
    }
  }
}

.app-container {
  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    width: calc(100% - var(--sidebar-width));
    background-color: var(--color-raised-bg);

    .critical-error-banner {
      margin-top: -1.25rem;
      padding: 1rem;
      background-color: rgba(203, 34, 69, 0.1);
      border-left: 2px solid var(--color-red);
      border-bottom: 2px solid var(--color-red);
      border-right: 2px solid var(--color-red);
      border-radius: 1rem;
    }

    .appbar {
      display: flex;
      align-items: center;
      flex-grow: 1;
      background: var(--color-raised-bg);
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
      border-top-left-radius: var(--radius-xl);
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

.nav-section {
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 100%;
  gap: 1rem;
}

.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);

  .transparent {
    padding: var(--gap-sm) 0;
  }
}

.app-grid-layout,
.app-contents {
  --top-bar-height: 3.75rem;
  --left-bar-width: 5rem;
  --right-bar-width: 300px;
}

.app-grid-layout {
  display: grid;
  grid-template: 'status status' 'nav dummy';
  grid-template-columns: auto 1fr;
  grid-template-rows: auto 1fr;
  position: relative;
  //z-index: 0;
  background-color: var(--color-raised-bg);
  height: 100vh;
}

.app-grid-navbar {
  grid-area: nav;
}

.app-grid-statusbar {
  grid-area: status;
}

.app-contents {
  position: absolute;
  z-index: 1;
  left: 5rem;
  top: 3.75rem;
  right: 0;
  bottom: 0;
  height: calc(100vh - 3.75rem);
  background-color: var(--color-bg);
  border-top-left-radius: var(--radius-xl);

  display: grid;
  grid-template-columns: 1fr 300px;
  //grid-template-columns: 1fr 0px;
  transition: grid-template-columns 0.4s ease-in-out;
}

.loading-indicator-container {
  border-top-left-radius: var(--radius-xl);
  overflow: hidden;
}

.app-sidebar {
  overflow: visible;
  width: 300px;
  position: relative;
  height: calc(100vh - 3.75rem);
  background: var(--brand-gradient-bg);

  --color-button-bg: var(--brand-gradient-button);
  --color-button-bg-hover: var(--brand-gradient-border);
  --color-divider: var(--brand-gradient-border);
  --color-divider-dark: var(--brand-gradient-border);
}

.app-sidebar::after {
  content: '';
  position: absolute;
  bottom: 250px;
  left: 0;
  right: 0;
  height: 5rem;
  background: var(--brand-gradient-fade-out-color);
  pointer-events: none;
}

.app-sidebar.has-plus::after {
  display: none;
}

.app-sidebar::before {
  content: '';
  box-shadow: -15px 0 15px -15px rgba(0, 0, 0, 0.2) inset;
  top: 0;
  bottom: 0;
  left: -2rem;
  width: 2rem;
  position: absolute;
  pointer-events: none;
}

.app-viewport {
  flex-grow: 1;
  height: 100%;
  overflow: auto;
  overflow-x: hidden;
}

//::-webkit-scrollbar-track {
//  background-color: transparent; /* Make it transparent if needed */
//  margin-block: 5px;
//  margin-right: 5px;
//}

.app-contents::before {
  z-index: 1;
  content: '';
  position: fixed;
  left: 5rem;
  top: 3.75rem;
  right: -5rem;
  bottom: -5rem;
  border-radius: var(--radius-xl);
  //box-shadow: 1px 1px 15px rgba(0, 0, 0, 0.2) inset;
  box-shadow:
    1px 1px 15px rgba(0, 0, 0, 0.2) inset,
    inset 1px 1px 1px rgba(255, 255, 255, 0.23);
  pointer-events: none;
}

.sidebar-teleport-content {
  display: contents;
}

.sidebar-default-content {
  display: none;
}

.sidebar-teleport-content:empty + .sidebar-default-content {
  display: contents;
}
</style>
<style>
.mac {
  .app-grid-statusbar {
    padding-left: 5rem;
  }
}

.windows {
  .fake-appbar {
    height: 2.5rem !important;
  }

  .window-controls {
    display: flex !important;
  }

  .info-card {
    right: 8rem;
  }

  .profile-card {
    right: 8rem;
  }
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
