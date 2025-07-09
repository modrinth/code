<template>
  <div data-tauri-drag-region class="flex items-center gap-1 pl-3">
    <Button v-if="false" class="breadcrumbs__back transparent" icon-only @click="$router.back()">
      <ChevronLeftIcon />
    </Button>
    <Button
      v-if="false"
      class="breadcrumbs__forward transparent"
      icon-only
      @click="$router.forward()"
    >
      <ChevronRightIcon />
    </Button>
    {{ breadcrumbData.resetToNames(breadcrumbs) }}
    <template v-for="breadcrumb in breadcrumbs" :key="breadcrumb.name">
      <router-link
        v-if="breadcrumb.link"
        :to="{
          path: breadcrumb.link.replace('{id}', encodeURIComponent($route.params.id)),
          query: breadcrumb.query,
        }"
        class="text-primary"
        >{{
          breadcrumb.name.charAt(0) === '?'
            ? breadcrumbData.getName(breadcrumb.name.slice(1))
            : breadcrumb.name
        }}
      </router-link>
      <span
        v-else
        data-tauri-drag-region
        class="text-contrast font-semibold cursor-default select-none"
        >{{
          breadcrumb.name.charAt(0) === '?'
            ? breadcrumbData.getName(breadcrumb.name.slice(1))
            : breadcrumb.name
        }}</span
      >
      <ChevronRightIcon v-if="breadcrumb.link" data-tauri-drag-region class="w-5 h-5" />
    </template>
  </div>
</template>

<script setup>
import { ChevronRightIcon, ChevronLeftIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()

const breadcrumbData = useBreadcrumbs()
const breadcrumbs = computed(() => {
  const additionalContext =
    route.meta.useContext === true
      ? breadcrumbData.context
      : route.meta.useRootContext === true
        ? breadcrumbData.rootContext
        : null
  return additionalContext ? [additionalContext, ...route.meta.breadcrumb] : route.meta.breadcrumb
})
</script>
