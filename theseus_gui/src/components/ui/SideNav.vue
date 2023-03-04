<script setup>
import { SunIcon, MoonIcon, SearchIcon, BookIcon, ClientIcon } from 'omorphia'
import { useStore } from 'vuex'
import { RouterLink } from 'vue-router'
import UserSection from './UserSection.vue'

const { state, commit } = useStore()
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
    <div id="instances"></div>
    <SunIcon v-if="!state.darkTheme" @click="commit('toggleTheme')" class="theme-icon" />
    <MoonIcon v-else @click="commit('toggleTheme')" class="theme-icon" />
  </div>
</template>

<style lang="scss" scoped>
.nav-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 252px;
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
      font-size: 18px;
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

  .theme-icon {
    align-self: flex-end;
    position: absolute;
    bottom: 20px;
    margin-right: 1.5rem;
    cursor: pointer;
    font-size: larger;
    border-radius: 10px;
  }
}

.dark-mode {
  .nav-container {
    background: #17191c;
  }

  #pages > a:hover {
    background-color: #2a2d32;
    box-shadow: 0px 4px 4px rgba(0, 0, 0, 0.25);
  }
}
</style>
