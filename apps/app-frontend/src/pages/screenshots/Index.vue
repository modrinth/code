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

breadcrumbs.setRootContext({ name: 'Screenshots', link: route.path })

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
    <h1 class="m-0 text-2xl hidden">Screen shots</h1>
    <NavTabs
      :links="[
        { label: 'Recent', href: `/screenshots` },
        { label: 'Downloaded Instances', href: `/screenshots/downloaded` },
        { label: 'Custom Instances', href: `/screenshots/custom` },
        { label: 'Shared with me', href: `/screenshots/shared`, shown: false },
        { label: 'Saved', href: `/screenshots/saved`, shown: false },
      ]"
    />
    <template v-if="instances && instances.length > 0">
      <RouterView />
    </template>
    <div v-else-if="instances && instances.length === 0" class="no-instance">
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
    <div v-else class="loading">
      <p>Loading instances...</p>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.debug-info {
  background: var(--color-bg-tertiary);
  padding: var(--gap-sm);
  border-radius: var(--radius-sm);
  margin-bottom: var(--gap-md);

  p {
    margin: 0;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
  }
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;

  p {
    margin: 0;
    color: var(--color-text-secondary);
  }
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

.no-screenshots {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 300px;
  gap: var(--gap-sm);
  text-align: center;
  color: var(--color-text-secondary);

  h3 {
    margin: 0;
    color: var(--color-text-primary);
  }

  p {
    margin: 0;
  }
}

.screenshots-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: var(--gap-lg);
}

.screenshot-card {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-secondary);
  transition:
    transform 0.2s ease,
    box-shadow 0.2s ease;

  &:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
}

.screenshot-image {
  width: 100%;
  height: 200px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-tertiary);

  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.2s ease;
  }

  &:hover img {
    transform: scale(1.05);
  }
}

.screenshot-info {
  padding: var(--gap-md);

  .screenshot-title {
    margin: 0 0 var(--gap-xs) 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .screenshot-instance {
    margin: 0 0 var(--gap-xs) 0;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    font-weight: 500;
  }

  .screenshot-date,
  .screenshot-size {
    margin: 0;
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
  }

  .screenshot-date {
    margin-bottom: var(--gap-xs);
  }

  .open-folder-btn {
    margin-top: var(--gap-sm);
    width: 100%;
  }
}
</style>
