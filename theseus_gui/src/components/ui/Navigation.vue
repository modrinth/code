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
import { useStore } from 'vuex'
import { RouterLink } from 'vue-router'
import SideNav from '../SideNav.vue'
import UserSection from './UserSection.vue'
import Instance from './Instance.vue'

const { state, commit } = useStore()

commit('fetchInstances')
</script>

<template>
  <SideNav class="navigation">
    <UserSection />
    <div id="pages">
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
    <div id="instances">
      <p>Instances</p>
      <Instance v-for="instance in state.instances" display="list" :instance="instance" />
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
      <SunIcon v-if="!state.darkTheme" @click="commit('toggleTheme')" class="theme-icon" />
      <MoonIcon v-else @click="commit('toggleTheme')" class="theme-icon" />
    </div>
  </SideNav>
</template>

<style lang="scss" scoped>
#pages {
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

#instances {
  display: flex;
  flex-direction: column;
  width: 80%;
  margin: 0.5rem;

  p {
    color: #b5b5b5;
    margin-bottom: 0.4rem;
    font-size: 0.7rem;
    line-height: 13px;
    font-weight: 400;
  }
}

#settings {
  position: relative;
  align-self: flex-end;
  display: flex;
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

  svg {
    background: #32d874;
    border-radius: var(--radius-sm);
    width: 25px;
    height: auto;
    box-shadow: 0px -1px 1px 0px #00000040 inset;
  }

  &:hover {
    text-decoration: none;
  }
}

.dark-mode {
  #add-instance-btn {
    &:hover {
      background-color: #2a2d32;
      box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
    }
  }

  #pages > a:hover {
    background-color: #2a2d32;
    box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
  }
}
</style>
