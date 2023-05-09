<script setup>
import GridDisplay from '@/components/GridDisplay.vue'
import { onMounted, ref } from 'vue'
import { list } from '@/helpers/profile'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { loading_listener } from '@/helpers/events.js'
import { progress_bars_list } from '@/helpers/state.js'
import SplashScreen from '@/components/ui/SplashScreen.vue'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const loading = ref(true)
const instances = ref(null)
const loadingInstances = ref(null)

breadcrumbs.setRootContext({ name: 'Library', link: route.path })

onMounted(async () => {
  instances.value = Object.values(await list())
  loadingInstances.value = Object.values(await progress_bars_list())
  loading.value = false
})

loading_listener(async (profile) => {
  console.log(profile)
  instances.value = Object.values(await list())
  loadingInstances.value = Object.values(await progress_bars_list())
})
</script>

<template>
  <transition name="fade">
    <SplashScreen v-if="loading" />
    <div v-else>
      <GridDisplay label="Instances" :instances="instances" />
      <GridDisplay label="Modpacks" :instances="instances" />
    </div>
  </transition>
</template>

<style lang="scss" scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
