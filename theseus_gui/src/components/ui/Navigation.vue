<script setup>
import { SearchIcon, BookIcon, ClientIcon, PlusIcon, SettingsIcon, Button } from 'omorphia'
import { useInstances } from '@/store/state'
import { RouterLink } from 'vue-router'
import UserSection from '@/components/ui/UserSection.vue'
import Instance from '@/components/ui/Instance.vue'
const instances = useInstances()

instances.fetchInstances()
</script>

<template>
  <div class="nav-container">
    <UserSection />
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
</template>

<style lang="scss" scoped>
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
</style>
