<script setup>
import { useStore } from 'vuex'
import RowDisplay from '../components/RowDisplay.vue'

const { state, commit } = useStore()

commit('fetchInstances')
commit('fetchNews')

const recentInstances = state.instances.filter((i) => i.downloads <= 50)
console.log(recentInstances)
const popularInstances = state.instances.filter((i) => i.downloads > 50)
const trendingMods = state.instances.filter((i) => i.trending)
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" />
    <RowDisplay label="Popular packs" :instances="popularInstances" />
    <RowDisplay label="News & updates" :news="state.news" />
    <RowDisplay label="Trending mods" :instances="trendingMods" />
  </div>
</template>

<style lang="scss" scoped>
.page-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
}
</style>
