<script setup>
import { useStore } from 'vuex'
import RowDisplay from '../components/RowDisplay.vue'

const { state, commit } = useStore()

commit('fetchInstances')
commit('fetchNews')

const recentInstances = state.instances.filter((i) => i.downloads <= 50)
const popularInstances = state.instances.filter((i) => i.downloads > 50)
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" />
    <RowDisplay label="Popular packs" :instances="popularInstances" />
    <RowDisplay label="News & updates" :news="state.news" />
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
