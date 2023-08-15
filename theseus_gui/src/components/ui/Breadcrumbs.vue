<template>
  <div class="breadcrumbs">
    <Button class="breadcrumbs__back transparent" icon-only @click="$router.back()">
      <ChevronLeftIcon />
    </Button>
    <Button class="breadcrumbs__forward transparent" icon-only @click="$router.forward()">
      <ChevronRightIcon />
    </Button>
    {{ breadcrumbData.resetToNames(breadcrumbs) }}
    <div v-for="breadcrumb in breadcrumbs" :key="breadcrumb.name" class="breadcrumbs__item">
      <router-link
        v-if="breadcrumb.link"
        :to="{
          path: breadcrumb.link.replace('{id}', encodeURIComponent($route.params.id)),
          query: breadcrumb.query,
        }"
        >{{
          breadcrumb.name.charAt(0) === '?'
            ? breadcrumbData.getName(breadcrumb.name.slice(1))
            : breadcrumb.name
        }}
      </router-link>
      <span v-else class="selected">{{
        breadcrumb.name.charAt(0) === '?'
          ? breadcrumbData.getName(breadcrumb.name.slice(1))
          : breadcrumb.name
      }}</span>
      <ChevronRightIcon v-if="breadcrumb.link" class="chevron" />
    </div>
  </div>
</template>

<script setup>
import { ChevronRightIcon, Button, ChevronLeftIcon } from 'omorphia'
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

<style scoped lang="scss">
.breadcrumbs {
  display: flex;
  flex-direction: row;

  .breadcrumbs__item {
    display: flex;
    flex-direction: row;
    vertical-align: center;
    margin: auto 0;

    .chevron,
    a {
      margin: auto 0;
    }
  }

  .breadcrumbs__back,
  .breadcrumbs__forward {
    margin: auto 0;
    color: var(--color-base);
    height: unset;
    width: unset;
  }

  .breadcrumbs__forward {
    margin-right: 1rem;
  }
}

.selected {
  color: var(--color-contrast);
}
</style>
