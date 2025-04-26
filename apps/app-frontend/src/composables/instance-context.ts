import { useRoute } from 'vue-router'
import { ref, computed, type Ref, watch } from 'vue'
import { handleError } from '@/store/notifications'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile'
import type { GameInstance, InstanceContent } from '@/helpers/types'

export type InstanceContentMap = Record<string, InstanceContent>

export async function useInstanceContext() {
  const route = useRoute()

  const instance: Ref<GameInstance | undefined> = ref()
  const instanceContent: Ref<InstanceContentMap | undefined> = ref()

  await loadInstance()

  watch(route, () => {
    loadInstance()
  })

  async function loadInstance() {
    ;[instance.value, instanceContent.value] = await Promise.all([
      route.query.i ? getInstance(route.query.i).catch(handleError) : Promise.resolve(),
      route.query.i ? getInstanceProjects(route.query.i).catch(handleError) : Promise.resolve(),
    ])
  }

  const instanceQueryAppendage = computed(() => {
    if (instance.value) {
      return `?i=${instance.value.path}`
    } else {
      return ''
    }
  })

  return {
    instance,
    instanceContent,
    instanceQueryAppendage,
  }
}
