<script setup>
import { ref } from 'vue'
import { ofetch } from 'ofetch'
import RowDisplay from '@/components/RowDisplay.vue'
import { shallowRef } from 'vue'
import { list } from '@/helpers/profile.js'

const loading = ref(false)

const featuredModpacks = ref({})
const featuredMods = ref({})

const profiles = await list()
const recentInstances = shallowRef(Object.values(profiles))

const getFeaturedModpacks = async () => {
  const response = await ofetch(
    'https://api.modrinth.com/v2/search?facets=[["project_type:modpack"]]&limit=10&index=follows'
  )
  featuredModpacks.value = response.hits
}
const getFeaturedMods = async () => {
  const response = await ofetch(
    'https://api.modrinth.com/v2/search?facets=[["project_type:mod"]]&limit=10&index=follows'
  )
  featuredMods.value = response.hits
}

loading.value = true
await Promise.all([getFeaturedModpacks(), getFeaturedMods()])
loading.value = false
</script>

<template>
  <div class="page-container">
    <RowDisplay label="Jump back in" :instances="recentInstances" :can-paginate="false" />
    <RowDisplay label="Popular packs" :instances="featuredModpacks" :can-paginate="true" />
    <RowDisplay label="Popular mods" :instances="featuredMods" :can-paginate="true" />
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
