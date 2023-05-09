<script setup>
import GridDisplay from '@/components/GridDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const profiles = await list()
const instances = shallowRef(Object.values(profiles))

breadcrumbs.setRootContext({ name: 'Library', link: route.path })
</script>

<template>
  <div>
    <GridDisplay label="Instances" :instances="instances" />
    <GridDisplay label="Modpacks" :instances="instances" />
  </div>
</template>
