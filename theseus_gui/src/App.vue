<script setup>
import { watch } from 'vue'
import { RouterView } from 'vue-router'
import { useStore } from 'vuex'
import SideNav from './components/ui/SideNav.vue'
import AppBar from './components/ui/AppBar.vue'
import { toggleTheme } from './helpers/theme'

const { state } = useStore()

toggleTheme(state.darkTheme)

watch(state, (newState) => {
  toggleTheme(newState.darkTheme)
})
</script>

<template>
  <div class="container">
    <SideNav />
    <div class="view">
      <AppBar />
      <RouterView />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.container {
  display: flex;
  min-width: 100vw;
  min-height: 100vh;

  .view {
    width: 100%;
    height: 100%;
  }
}
</style>
