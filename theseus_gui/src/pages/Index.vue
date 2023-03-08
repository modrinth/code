<script setup>
import { useInstances, useNews } from '../store/state'
import RowDisplay from '../components/RowDisplay.vue'

const instances = useInstances()
const news = useNews()
instances.fetchInstances()
news.fetchNews()

const recentInstances = instances.instances.filter((i) => i.downloads <= 50)
const popularInstances = instances.instances.filter((i) => i.downloads > 50)
const trendingMods = instances.instances.filter((i) => i.trending)
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" />
    <RowDisplay label="Popular packs" :instances="popularInstances" />
    <RowDisplay label="News & updates" :news="news.news" />
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
