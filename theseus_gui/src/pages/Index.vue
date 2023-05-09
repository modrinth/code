<script setup>
import RowDisplay from '@/components/RowDisplay.vue'
import { onMounted, ref } from 'vue'
import { list } from '@/helpers/profile.js'
import { useRoute } from 'vue-router'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import SplashScreen from '@/components/ui/SplashScreen.vue'

const route = useRoute()
const breadcrumbs = useBreadcrumbs()

const loading = ref(true)
const recentInstances = ref(null)

onMounted(async () => {
  recentInstances.value = Object.values(await list())
  loading.value = false
})

breadcrumbs.setRootContext({ name: 'Home', link: route.path })
</script>

<template>
  <transition name="fade">
    <SplashScreen v-if="loading" />
    <div v-else class="page-container">
      <RowDisplay label="Jump back in" :instances="recentInstances" :can-paginate="false" />
      <RowDisplay label="Popular packs" :instances="recentInstances" :can-paginate="true" />
      <RowDisplay label="Test" :instances="recentInstances" :can-paginate="true" />
    </div>
  </transition>
</template>

<style lang="scss" scoped>
.page-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
