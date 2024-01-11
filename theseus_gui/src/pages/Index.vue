<script setup>
import { ref, onUnmounted, shallowRef } from 'vue'
import { useRoute } from 'vue-router'
import RowDisplay from '@/components/RowDisplay.vue'
import { list } from '@/helpers/profile.js'
import { profile_listener } from '@/helpers/events'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { handleError } from '@/store/notifications.js'
import dayjs from 'dayjs'

const filter = ref('')

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const recentInstances = shallowRef([])

const getInstances = async () => {
  const profiles = await list(true).catch(handleError)
  recentInstances.value = Object.values(profiles).sort((a, b) => {
    return dayjs(b.metadata.last_played ?? 0).diff(dayjs(a.metadata.last_played ?? 0))
  })

  let filters = []
  for (const instance of recentInstances.value) {
    if (instance.metadata.linked_data && instance.metadata.linked_data.project_id) {
      filters.push(`NOT"project_id"="${instance.metadata.linked_data.project_id}"`)
    }
  }
  filter.value = filters.join(' AND ')
}

await getInstances()

const unlistenProfile = await profile_listener(async () => {
  await getInstances()
})

onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <div class="page-container">
    <RowDisplay
      v-if="recentInstances.length > 0"
      :instances="[
        {
          label: 'Jump back in',
          route: '/library',
          instances: recentInstances,
          downloaded: true,
        },
      ]"
      :can-paginate="true"
    />
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
