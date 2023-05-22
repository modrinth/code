<script setup>
import { onMounted, ref, watch } from 'vue'
import { RouterView, RouterLink } from 'vue-router'
import {
  HomeIcon,
  SearchIcon,
  LibraryIcon,
  PlusIcon,
  SettingsIcon,
  Button,
  Notifications,
} from 'omorphia'
import { handleError, useLoading, useTheming } from '@/store/state'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { get } from '@/helpers/settings'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import ModrinthLoadingIndicator from '@/components/modrinth-loading-indicator'
import { useNotifications } from '@/store/notifications.js'
import { warning_listener } from '@/helpers/events.js'

const themeStore = useTheming()

const isLoading = ref(true)
onMounted(async () => {
  const { settings, collapsed_navigation } = await get().catch(handleError)
  themeStore.setThemeState(settings)
  themeStore.collapsedNavigation = collapsed_navigation

  await warning_listener((e) =>
    notificationsWrapper.value.addNotification({
      title: 'Warning',
      text: e.message,
      type: 'warn',
    })
  )
})

defineExpose({
  initialize: async () => {
    isLoading.value = false
    const { theme } = await get()
    themeStore.setThemeState(theme)
  },
})
const loading = useLoading()

const notifications = useNotifications()
const notificationsWrapper = ref(null)

watch(notificationsWrapper, () => {
  notifications.setNotifs(notificationsWrapper.value)
})
</script>

<template>
  <SplashScreen v-if="isLoading" app-loading />
  <div v-else class="container">
    <div class="nav-container" :class="{ expanded: !themeStore.collapsedNavigation }">
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" :expanded="!themeStore.collapsedNavigation" />
        </suspense>
        <div class="pages-list">
          <RouterLink
            to="/"
            class="btn"
            :class="{
              'icon-only': themeStore.collapsedNavigation,
              'collapsed-button': themeStore.collapsedNavigation,
              'expanded-button': !themeStore.collapsedNavigation,
            }"
          >
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
      <div class="appbar">
        <section class="navigation-controls">
          <Breadcrumbs />
        </section>
        <section class="mod-stats">
          <Suspense>
            <RunningAppBar />
          </Suspense>
        </section>
      </div>
      <div class="router-view">
        <ModrinthLoadingIndicator
          offset-height="var(--appbar-height)"
          offset-width="var(--sidebar-width)"
        />
        <Notifications ref="notificationsWrapper" />
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
</template>

<style lang="scss" scoped>
.sleek-primary {
  background-color: var(--color-brand-highlight);
  transition: all ease-in-out 0.1s;
}
.container {
  --appbar-height: 3.25rem;
  --sidebar-width: 5rem;

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
      justify-content: space-between;
      align-items: center;
      background: var(--color-super-raised-bg);
      text-align: center;
      padding: 0 0 0 1rem;
      height: 3.25rem;

      .navigation-controls {
        display: inherit;
        align-items: inherit;
        justify-content: stretch;

        svg {
          width: 1.25rem;
          height: 1.25rem;
          transition: all ease-in-out 0.1s;

          &:hover {
            filter: brightness(150%);
          }
        }

        p {
          margin-left: 0.3rem;
        }

        svg {
          margin: auto 0.1rem;
          transition: all ease-in-out 0.1s;
          cursor: pointer;

          &:hover {
            font-weight: bolder;
          }
        }
      }

      .mod-stats {
        height: 100%;
        display: inherit;
        align-items: inherit;
        justify-content: flex-end;
      }
    }

    .router-view {
      width: 100%;
      height: calc(100% - 3.125rem);
      overflow: auto;
      overflow-x: hidden;
    }
  }
}

.dark-mode {
  .nav-container {
    background: var(--color-bg) !important;
  }
  .pages-list {
    a.router-link-active {
      color: #fff;
    }
  }
}

.light-mode {
  .nav-container {
    box-shadow: var(--shadow-floating), var(--shadow-floating), var(--shadow-floating),
      var(--shadow-floating) !important;
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: 1rem;
  background: var(--color-raised-bg);

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
