<script setup>
import { watch } from 'vue'
import { RouterView } from 'vue-router'
import { useTheming } from '@/store/state'
import Navigation from '@/components/ui/Navigation.vue'
import AppBar from '@/components/ui/AppBar.vue'
import { toggleTheme } from '@/helpers/theme'

const theme = useTheming()

toggleTheme(theme.darkTheme)

watch(theme, (newState) => {
  toggleTheme(newState.darkTheme)
})
</script>

<template>
  <div class="container">
    <Navigation class="navigation" />
    <div class="view">
      <AppBar />
      <RouterView />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.container {
  display: flex;
  min-width: 100%;
  min-height: 100vh;
  overflow-x: hidden;

  .navigation {
    position: fixed;
  }

  .view {
    width: 100%;
    height: 100%;
    margin-left: 210px;
  }
}
</style>
