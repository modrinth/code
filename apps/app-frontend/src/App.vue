<script setup>
import { computed, ref, onMounted } from 'vue'
import { RouterView, RouterLink, useRouter, useRoute } from 'vue-router'
import {
  HomeIcon,
  SearchIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  XIcon,
  DownloadIcon,
} from '@modrinth/assets'
import { Button, Notifications } from '@modrinth/ui'
import { useLoading, useTheming } from '@/store/state'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
import ModrinthLoadingIndicator from '@/components/modrinth-loading-indicator'
import { handleError, useNotifications } from '@/store/notifications.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { MinimizeIcon, MaximizeIcon } from '@/assets/icons'
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
import { open } from '@tauri-apps/plugin-shell'
import { get_opening_command, initialize_state } from '@/helpers/state'
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'
import { renderString } from '@modrinth/utils'
import { useFetch } from '@/helpers/fetch.js'
import { check } from '@tauri-apps/plugin-updater'

const themeStore = useTheming()

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

onMounted(async () => {
  await useCheckDisableMouseover()
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

  if (default_page && default_page !== 'Home') {
    await router.push({ name: default_page })
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
const isOnBrowse = computed(() => route.path.startsWith('/browse'))

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

onMounted(() => {
  invoke('show_window')

  notifications.setNotifs(notificationsWrapper.value)

  error.setErrorModal(errorModal.value)

  install.setIncompatibilityWarningModal(incompatibilityWarningModal)
  install.setInstallConfirmModal(installConfirmModal)
  install.setModInstallModal(modInstallModal)
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
        !target.href.startsWith('https://tauri.localhost') &&
        !target.href.startsWith('http://tauri.localhost')
      ) {
        open(target.href)
      }
      e.preventDefault()
      break
    }
    target = target.parentElement
  }
})

document.querySelector('body').addEventListener('auxclick', function (e) {
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
  console.log(update)
  updateAvailable.value = !!update

  setTimeout(
    () => {
      checkUpdates()
    },
    5 * 1000 * 60,
  )
}
</script>

<template>
  <SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
  <div v-if="stateInitialized" class="app-container">
    <div class="nav-container">
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" mode="small" />
        </suspense>
        <div class="pages-list">
          <RouterLink v-tooltip="'Home'" to="/" class="btn icon-only collapsed-button">
            <HomeIcon />
          </RouterLink>
          <RouterLink
            v-tooltip="'Browse'"
            to="/browse/modpack"
            class="btn icon-only collapsed-button"
            :class="{
              'router-link-active': isOnBrowse,
            }"
          >
            <SearchIcon />
          </RouterLink>
          <RouterLink v-tooltip="'Library'" to="/library" class="btn icon-only collapsed-button">
            <LibraryIcon />
          </RouterLink>
          <Suspense>
            <InstanceCreationModal ref="installationModal" />
          </Suspense>
        </div>
      </div>
      <div class="settings pages-list">
        <button
          v-if="updateAvailable"
          v-tooltip="'Install update'"
          class="btn btn-outline btn-primary icon-only collapsed-button"
          @click="restartApp()"
        >
          <DownloadIcon />
        </button>
        <Button
          v-tooltip="'Create profile'"
          class="sleek-primary collapsed-button"
          icon-only
          :disabled="offline"
          @click="() => $refs.installationModal.show()"
        >
          <PlusIcon />
        </Button>
        <RouterLink v-tooltip="'Settings'" to="/settings" class="btn icon-only collapsed-button">
          <SettingsIcon />
        </RouterLink>
      </div>
    </div>
    <div class="view">
      <div v-if="criticalErrorMessage" class="critical-error-banner" data-tauri-drag-region>
        <h1>{{ criticalErrorMessage.header }}</h1>
        <div class="markdown-body" v-html="renderString(criticalErrorMessage.body ?? '')"></div>
      </div>
      <div class="appbar-row">
        <div data-tauri-drag-region class="appbar">
          <section class="navigation-controls">
            <Breadcrumbs data-tauri-drag-region />
          </section>
          <section class="mod-stats">
            <Suspense>
              <RunningAppBar />
            </Suspense>
          </section>
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
            <MaximizeIcon />
          </Button>
          <Button class="titlebar-button close" icon-only @click="handleClose">
            <XIcon />
          </Button>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <RouterView v-slot="{ Component }">
          <template v-if="Component">
            <Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
              <component :is="Component"></component>
            </Suspense>
          </template>
        </RouterView>
      </div>
    </div>
  </div>
  <URLConfirmModal ref="urlModal" />
  <Notifications ref="notificationsWrapper" />
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

.app-container {
  --appbar-height: 3.25rem;
  --sidebar-width: 4.5rem;

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
</style>
<style>
.mac {
  .nav-container {
    padding-top: calc(var(--gap-md) + 1.75rem);
  }

  .account-card,
  .card-section {
    top: calc(var(--gap-md) + 1.75rem);
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
