<script setup>
import { shallowRef, ref } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs, useNotifications } from '@/store/state'
import { profile_listener } from '@/helpers/events.js'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()
const notificationStore = useNotifications()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const profiles = ref([])
const instances = shallowRef([])

try {
  profiles.value = await list(true)
  instances.value = Object.values(profiles.value)
} catch (err) {
  notificationStore.addTauriErrorNotif(err)
}

profile_listener(async () => {
  try {
    profiles.value = await list(true)
    instances.value = Object.values(profiles.value)
  } catch (err) {
    notificationStore.addTauriErrorNotif(err)
  }
})
</script>

<template>
  <GridDisplay
    v-if="instances.length > 0"
    label="Instances"
    :instances="instances"
    class="display"
  />
</template>

<style lang="scss" scoped>
.dark-mode {
  .display {
    background-color: rgb(30, 31, 34);
  }
}

.display {
  min-height: 100%;
}
</style>
