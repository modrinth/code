<script setup>
import {
  SunIcon,
  MoonIcon,
  SearchIcon,
  BookIcon,
  ClientIcon,
  PlusIcon,
  SettingsIcon,
} from 'omorphia'
import { useTheming, useInstances } from '@/store/state'
import { RouterLink } from 'vue-router'
import SideNav from '@/components/SideNav.vue'
import UserSection from '@/components/ui/UserSection.vue'
import Instance from '@/components/ui/Instance.vue'

const theme = useTheming()
const instances = useInstances()

instances.fetchInstances()
</script>

<template>
  <SideNav class="navigation">
    <UserSection />
    <div class="pages-list">
      <RouterLink
        to="/"
        class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md standard-button"
      >
        <ClientIcon />Home</RouterLink
      >
      <RouterLink
        to="/browse"
        class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md standard-button"
      >
        <SearchIcon />Browse</RouterLink
      >
      <RouterLink
        to="/library"
        class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md standard-button"
      >
        <BookIcon />Library</RouterLink
      >
    </div>
    <div class="instance-list">
      <p>Instances</p>
      <Instance v-for="instance in instances.instances" display="list" :instance="instance" />
    </div>
    <RouterLink
      to="/add-instance"
      class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md standard-button"
      id="add-instance-btn"
    >
      <PlusIcon />
      Create Instance
    </RouterLink>

    <div id="settings">
      <RouterLink
        to="/settings"
        class="omorphia__button button-base padding-block-sm padding-inline-lg radius-md standard-button"
        ><SettingsIcon /> Settings</RouterLink
      >
      <SunIcon v-if="!theme.darkTheme" @click="theme.toggleTheme()" class="theme-icon" />
      <MoonIcon v-else @click="theme.toggleTheme()" class="theme-icon" />
    </div>
  </SideNav>
</template>

<style lang="scss" scoped>
.pages-list {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;

  a {
    display: flex;
    align-items: flex-start;
    width: 80%;
    margin: 0.3rem auto;
    text-align: left;
    font-size: 1rem;
    font-weight: 400;
    background: inherit;
    transition: all ease-in-out 0.1s;

    &:hover {
      background-color: #fff;
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      text-decoration: none;
    }

    svg {
      margin-right: 0.5rem;
    }
  }
}

.instance-list {
  display: flex;
  flex-direction: column;
  justify-content: center;
  width: 70%;
  margin: 0.4rem;

  & > p {
    color: var(--color-primary);
    margin-bottom: 0.6rem;
    font-size: 1rem;
    line-height: 13px;
    font-weight: 500;
  }
}

#settings {
  position: fixed;
  align-self: flex-end;
  display: flex;
  bottom: 0.5rem;
  left: 0.5rem;
  align-items: center;
  margin-right: 1rem;
  width: 150px;
  justify-content: space-between;

  a {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 80%;
    margin: auto;
    background: inherit;
    margin-right: 1rem;

    &:hover {
      text-decoration: none;
    }
  }

  .theme-icon {
    cursor: pointer;
    font-size: larger;
    border-radius: var(--radius-sm);
  }
}

#add-instance-btn {
  background: inherit;
  display: flex;
  align-items: center;
  justify-content: space-evenly;
  width: 85%;
  font-size: 0.9rem;
  margin-top: 1rem;

  svg {
    background: var(--color-green);
    border-radius: var(--radius-sm);
    width: 25px;
    height: auto;
    box-shadow: var(--shadow-inset);
  }

  &:hover {
    text-decoration: none;
  }
}

.dark-mode {
  #add-instance-btn {
    &:hover {
      background-color: var(--color-button-bg);
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
    }
  }

  .pages-list > a:hover {
    background-color: var(--color-button-bg);
    box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
  }
}
</style>
