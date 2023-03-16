<script setup>
import { watch } from 'vue'
import { RouterView, useRoute, useRouter, RouterLink } from 'vue-router'
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  SearchIcon,
  BookIcon,
  ClientIcon,
  PlusIcon,
  SettingsIcon,
  Button,
  Avatar,
} from 'omorphia'
import { useTheming, useInstances } from '@/store/state'
import { toggleTheme } from '@/helpers/theme'
import Instance from '@/components/ui/Instance.vue'

const route = useRoute()
const router = useRouter()

const theme = useTheming()
const instances = useInstances()
instances.fetchInstances()

toggleTheme(theme.darkTheme)

watch(theme, (newState) => {
  toggleTheme(newState.darkTheme)
})
</script>

<template>
  <div class="container">
    <div class="navigation">
      <div class="nav-container">
        <section class="user-section">
          <Avatar size="sm" src="https://cdn.modrinth.com/data/AANobbMI/icon.png" />
          <section>
            <p class="username">OreoViking</p>
            <a href="#">Manage accounts</a>
          </section>
        </section>
        <div class="pages-list">
          <RouterLink to="/" class="btn"> <ClientIcon />Home</RouterLink>
          <RouterLink to="/browse" class="btn"> <SearchIcon />Browse</RouterLink>
          <RouterLink to="/library" class="btn"> <BookIcon />Library</RouterLink>
        </div>
        <div class="instance-list">
          <p>Instances</p>
          <Instance v-for="instance in instances.instances" display="list" :instance="instance" />
        </div>
        <Button class="add-instance-btn">
          <PlusIcon />
          Create Instance
        </Button>

        <div class="settings">
          <RouterLink to="/settings" class="btn"><SettingsIcon /> Settings</RouterLink>
        </div>
      </div>
    </div>
    <div class="view">
      <div class="appbar">
        <section class="navigation-controls">
          <ChevronLeftIcon @click="router.back()" />
          <ChevronRightIcon @click="router.forward()" />
          <p>{{ route.name }}</p>
        </section>
        <section class="mod-stats">
          <p>Updating 2 mods...</p>
          <p>123 mods installed</p>
        </section>
      </div>
      <RouterView />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.container {
  min-width: 100%;
  min-height: 100vh;
  overflow-x: hidden;

  .navigation {
    position: fixed;
  }

  .view {
    height: 100%;
    margin-left: 210px;

    .appbar {
      display: flex;
      justify-content: space-between;
      align-items: center;
      width: 100%;
      height: 30px;
      border-bottom: 1px solid rgba(64, 67, 74, 0.2);
      background: var(--color-button-bg);
      padding: 1.2rem;

      .navigation-controls {
        display: inherit;
        align-items: inherit;
        justify-content: stretch;
        width: 30%;
        font-size: 0.9rem;

        svg {
          width: 18px;
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
        display: inherit;
        align-items: inherit;
        justify-content: flex-end;
        width: 50%;
        font-size: 0.8rem;
        margin-right: 1rem;

        p:nth-child(1) {
          margin-right: 0.55rem;
        }
      }
    }
  }
}

.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 210px;
  height: 100vh;
  box-shadow: var(--shadow-inset-sm), var(--shadow-floating);
}

.dark-mode {
  .nav-container {
    background: var(--color-bg);
  }
}
.pages-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  width: 100%;

  a {
    display: flex;
    align-items: center;
    width: 80%;
    margin: 0.2rem auto;
    text-align: left;
    font-size: 0.9rem;
    font-weight: 400;
    word-spacing: 3px;
    background: inherit;
    transition: all ease-in-out 0.1s;
    color: var(--color-primary);

    &.router-link-active {
      color: #000;
      background: var(--color-button-bg);
    }

    &:hover {
      background-color: var(--color-button-bg);
      color: var(--color-contrast);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }

    svg {
      margin-right: 1rem;
      width: 1.2rem;
      height: 1.2rem;
    }
  }
}

.dark-mode {
  .pages-list {
    a.router-link-active {
      color: #fff;
    }
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
  display: flex;
  align-items: center;
  margin: auto auto 0.5rem 1rem;
  width: 9.375rem;

  a {
    display: flex;
    background: inherit;
    color: var(--color-primary);

    svg {
      margin-right: 0.9em;
    }

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
  margin-left: 3rem;

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
</style>
