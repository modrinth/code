<script setup>
import { onUnmounted, ref, shallowRef } from 'vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs.js'
import { profile_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { Button } from '@modrinth/ui'
import { PlusIcon } from '@modrinth/assets'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { NewInstanceImage } from '@/assets/icons'
import NavTabs from '@/components/ui/NavTabs.vue'

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
  <div class="p-6 flex flex-col gap-3">
    <h1 class="m-0 text-2xl hidden">Library</h1>
    <NavTabs
      :links="[
        { label: 'All instances', href: `/library` },
        { label: 'Downloaded', href: `/library/downloaded` },
        { label: 'Custom', href: `/library/custom` },
        { label: 'Shared with me', href: `/library/shared`, shown: false },
        { label: 'Saved', href: `/library/saved`, shown: false },
      ]"
    />
    <template v-if="instances.length > 0">
      <RouterView :instances="instances" />
    </template>
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
