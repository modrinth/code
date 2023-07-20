<script setup>
import { onUnmounted, shallowRef } from 'vue'
import GridDisplay from '@/components/GridDisplay.vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { profile_listener } from '@/helpers/events.js'
import { handleError } from '@/store/notifications.js'
import { Button, PlusIcon } from 'omorphia'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import { NewInstanceImage } from '@/assets/icons'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

const profiles = await list(true).catch(handleError)
const instances = shallowRef(Object.values(profiles))

const unlisten = await profile_listener(async () => {
  const profiles = await list(true).catch(handleError)
  instances.value = Object.values(profiles)
})
onUnmounted(() => unlisten())
</script>

<template>
  <GridDisplay
    v-if="instances.length > 0"
    label="Instances"
    :instances="instances"
    class="display"
  />
  <div v-else class="no-instance">
    <div class="icon">
      <NewInstanceImage />
    </div>
    <h3>No instances found</h3>
    <Button color="primary" @click="$refs.installationModal.show()">
      <PlusIcon />
      Create new instance
    </Button>
    <InstanceCreationModal ref="installationModal" />
  </div>
</template>

<style lang="scss" scoped>
.display {
  background-color: rgb(30, 31, 34);
  min-height: 100%;
}

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
