<script setup>
import { onMounted, onUnmounted, ref, shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { profile_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { Button } from '@modrinth/ui'
import { PlusIcon } from '@modrinth/assets'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { NewInstanceImage } from '@/assets/icons'
import { hide_ads_window } from '@/helpers/ads.js'

onMounted(() => {
  hide_ads_window(true)
})

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const instances = shallowRef(await list().catch(handleError))

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
  offline.value = true
})
window.addEventListener('online', () => {
  offline.value = false
})

const unlistenProfile = await profile_listener(async () => {
  instances.value = await list().catch(handleError)
})
onUnmounted(() => {
  unlistenProfile()
})
</script>

<template>
  <GridDisplay v-if="instances.length > 0" label="Instances" :instances="instances" />
  <div v-else class="no-instance">
    <div class="icon">
      <NewInstanceImage />
    </div>
    <h3>No instances found</h3>
    <Button color="primary" :disabled="offline" @click="$refs.installationModal.show()">
      <PlusIcon />
      Create new instance
    </Button>
    <InstanceCreationModal ref="installationModal" />
  </div>
</template>

<style lang="scss" scoped>
.no-instance {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--gap-md);

  p,
  h3 {
    margin: 0;
  }

  .icon {
    svg {
      width: 10rem;
      height: 10rem;
    }
  }
}
</style>
