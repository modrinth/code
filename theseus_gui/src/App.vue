<script setup>
import { watch } from 'vue'
import { RouterView } from 'vue-router'
import { useStore } from 'vuex'
import SideNav from './components/ui/SideNav.vue'
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
    <RouterView />
  </div>
</template>

<style scoped>
.container {
  display: flex;
  width: 100vw;
  height: 100vh;
}
</style>
