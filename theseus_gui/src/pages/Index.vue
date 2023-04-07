<script setup>
import RowDisplay from '@/components/RowDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile.js'

const profiles = await list()
const recentInstances = shallowRef(Object.values(profiles))
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" :can-paginate="false" />
    <RowDisplay label="Popular packs" :instances="recentInstances" :can-paginate="true" />
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
