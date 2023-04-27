<script setup>
import { ref, onMounted } from 'vue'
import { RouterView, RouterLink } from 'vue-router'
import { HomeIcon, SearchIcon, LibraryIcon, PlusIcon, SettingsIcon } from 'omorphia'
import { useTheming } from '@/store/state'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { list } from '@/helpers/profile'
import { get } from '@/helpers/settings'
import { loading_listener } from '@/helpers/events'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'

const themeStore = useTheming()

onMounted(async () => {
  const { theme } = await get()
  themeStore.setThemeState(theme)
  await getInstalledModsCount()

  // When a modpack is finished installing, get the count of installed mods to update the app bar
  await loading_listener(async (e) => {
    installProgress.value = 0
    if (e.message === 'Downloading modpack...' && e.fraction === 1) {
      setTimeout(async () => {
        installProgress.value = 0
        await getInstalledModsCount()
      }, 500)
    } else if (e.message === 'Downloading modpack...' && e.fraction < 1)
      installProgress.value = Math.round(e.fraction * 100)
  })
})

const installedMods = ref(0)
const installProgress = ref(0)

const getInstalledModsCount = async () => {
  const profiles = await list()
  installedMods.value = Object.values(profiles).reduce(
    (acc, val) => acc + Object.keys(val.projects).length,
    0
  )
}
</script>

<template>
  <div class="container">
    <div class="nav-container">
      <div class="nav-section">
        <suspense>
          <AccountsCard ref="accounts" />
        </suspense>
        <div class="pages-list">
          <RouterLink to="/" class="button-base nav-button"><HomeIcon /></RouterLink>
          <RouterLink to="/browse" class="button-base nav-button"> <SearchIcon /></RouterLink>
          <RouterLink to="/library" class="button-base nav-button"> <LibraryIcon /></RouterLink>
          <button
            color="primary"
            class="button-base primary nav-button"
            icon-only
            @click="() => $refs.installationModal.show()"
          >
            <PlusIcon />
          </button>
          <Suspense>
            <InstanceCreationModal ref="installationModal" />
          </Suspense>
        </div>
      </div>
      <div class="settings pages-list">
        <RouterLink to="/settings" class="button-base nav-button"><SettingsIcon /></RouterLink>
      </div>
    </div>
    <div class="view">
      <div class="appbar">
        <section class="navigation-controls">
          <Breadcrumbs />
        </section>
        <section class="mod-stats">
          <p v-if="installProgress !== 0">Installing: {{ installProgress }}%</p>
          <p v-else>{{ installedMods }} mods installed</p>
          <Suspense>
            <RunningAppBar />
          </Suspense>
        </section>
      </div>
      <div class="router-view">
        <Suspense>
          <RouterView />
        </Suspense>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.container {
  height: 100vh;
  display: flex;
  flex-direction: row;
  overflow: hidden;

  .view {
    margin-left: 5rem;
    width: calc(100% - 5rem);
    height: calc(100%);

    .appbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      background: var(--color-super-raised-bg);
      text-align: center;
      padding: 0 0 0 1rem;
      height: 3.25rem;
      z-index: 11;

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
  position: absolute;
  justify-content: space-between;
  z-index: 10;
  height: 100%;
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
  padding: 1rem;
  background: var(--color-raised-bg);
}

.pages-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  width: 100%;
  gap: 0.5rem;

  a {
    display: flex;
    align-items: center;
    font-size: 0.9rem;
    font-weight: 400;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-base);

    &.router-link-active {
      color: var(--color-contrast);
      background: var(--color-button-bg);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }
  }
}

.nav-button {
  height: 3rem;
  width: 3rem;
  padding: 0.75rem;
  border-radius: var(--radius-md);

  svg {
    width: 1.5rem;
    height: 1.5rem;
    max-width: 1.5rem;
    max-height: 1.5rem;
  }

  &.primary {
    color: var(--color-accent-contrast);
    background-color: var(--color-brand);
  }
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

.add-instance-btn {
  background-color: var(--color-bg);
  font-size: 0.9rem;
  margin-right: 0.6rem;

  svg {
    background-color: var(--color-green);
    width: 1.5rem;
    height: 1.5rem;
    color: var(--color-accent-contrast);
    border-radius: var(--radius-xs);
  }
}

.settings {
  svg {
    color: var(--color-base) !important;
  }

  a {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    align-items: center;
    gap: 1rem;

    &:hover {
      text-decoration: none;
    }
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
    font-size: 1.1rem;
    font-weight: 400;
    line-height: 1.25rem;
    color: var(--color-contrast);
  }

  a {
    font-size: 0.75rem;
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
