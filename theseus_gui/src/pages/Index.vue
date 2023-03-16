<script setup>
import { useInstances, useNews } from '@/store/state'
import RowDisplay from '@/components/RowDisplay.vue'

const instances = useInstances()
const news = useNews()
instances.fetchInstances()
news.fetchNews()

// Remove once state is populated with real data
const recentInstances = instances.instances.slice(0, 4)
const popularInstances = instances.instances.filter((i) => i.downloads > 50 || i.trending)
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" :canPaginate="false" />
    <RowDisplay label="Popular packs" :instances="popularInstances" :canPaginate="true" />
    <RowDisplay label="News & updates" :news="news.news" :canPaginate="true" />
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
