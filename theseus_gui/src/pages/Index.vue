<script setup>
import RowDisplay from '@/components/RowDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const profiles = await list()
const recentInstances = shallowRef(Object.values(profiles))

breadcrumbs.setRootContext({ name: 'Home', link: route.path })
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" :can-paginate="false" />
    <RowDisplay label="Popular packs" :instances="recentInstances" :can-paginate="true" />
    <RowDisplay label="Test" :instances="recentInstances" :can-paginate="true" />
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
