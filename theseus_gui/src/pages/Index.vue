<script setup>
import { useStore } from 'vuex'
import ModRowDisplay from '../components/ModRowDisplay.vue'

const { state } = useStore()

const recentInstances = state.instances.slice(0, 4)
const popularInstances = state.instances.filter((i) => i.downloads > 50)
</script>

<template>
  <div class="page-container">
    <ModRowDisplay label="Jump back in" :instances="recentInstances" />
    <ModRowDisplay label="Popular packs" :instances="popularInstances" />
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
