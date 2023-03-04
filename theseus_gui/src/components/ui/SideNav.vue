<script setup>
import { SunIcon, MoonIcon, SearchIcon, BookIcon, ClientIcon, PlusIcon } from 'omorphia'
import { useStore } from 'vuex'
import { RouterLink } from 'vue-router'
import UserSection from './UserSection.vue'
import Instance from './Instance.vue'

const { state, commit } = useStore()

commit('fetchInstances')
</script>

<template>
  <div class="nav-container">
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
    <SunIcon v-if="!state.darkTheme" @click="commit('toggleTheme')" class="theme-icon" />
    <MoonIcon v-else @click="commit('toggleTheme')" class="theme-icon" />
  </div>
</template>

<style lang="scss" scoped>
.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 260px;
  height: 100vh;
  box-shadow: 0px 0px 8px 3px #00000026;

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
      margin: 5px auto;
      text-align: left;
      font-size: 16px;
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
    margin: 2rem;

    p {
      color: #b5b5b5;
      margin-bottom: 1rem;
      font-size: 11px;
      line-height: 13px;
      font-weight: 400;
    }
  }

  .theme-icon {
    align-self: flex-end;
    position: absolute;
    bottom: 20px;
    margin-right: 1.5rem;
    cursor: pointer;
    font-size: larger;
    border-radius: 10px;
  }

  #add-instance-btn {
    background: inherit;
    display: flex;
    align-items: center;
    justify-content: space-evenly;
    width: 85%;
    font-size: 14px;

    svg {
      background: #32d874;
      border-radius: 8px;
      width: 25px;
      height: auto;
      box-shadow: 0px -1px 1px 0px #00000040 inset;
    }

    &:hover {
      text-decoration: none;
    }
  }
}

.dark-mode {
  .nav-container {
    background: #17191c;

    #add-instance-btn {
      &:hover {
        background-color: #2a2d32;
        box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
      }
    }
  }

  #pages > a:hover {
    background-color: #2a2d32;
    box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
  }
}
</style>
