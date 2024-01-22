<template>
  <div class="breadcrumbs">
    <div v-for="breadcrumb in breadcrumbs" :key="breadcrumb.name" class="breadcrumbs__item">
      <router-link
        v-if="breadcrumb.link"
        :to="{
          path: breadcrumb.link.replace('{id}', encodeURIComponent($route.params.id)),
          query: breadcrumb.query,
        }"
      >
        {{ breadcrumbName(breadcrumb.name) }}
      </router-link>
      <span v-else class="selected">
        {{ breadcrumbName(breadcrumb.name) }}
      </span>
      <ChevronRightIcon v-if="breadcrumb.link" class="chevron" />
    </div>
  </div>
</template>

<script setup>
import { ChevronRightIcon } from 'omorphia'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useRoute } from 'vue-router'
import { computed } from 'vue'

const route = useRoute()

const breadcrumbData = useBreadcrumbs()

breadcrumbData.$subscribe(() => {
  breadcrumbData?.resetToNames(breadcrumbs.value)
})

const breadcrumbs = computed(() => {
  const additionalContext =
    route.meta.useContext === true
      ? breadcrumbData.context
      : route.meta.useRootContext === true
      ? breadcrumbData.rootContext
      : null
  return additionalContext ? [additionalContext, ...route.meta.breadcrumb] : route.meta.breadcrumb
})

const breadcrumbName = (bcn) => {
  if (bcn.charAt(0) === '?') {
    return breadcrumbData.getName(bcn.slice(1))
  }
  return bcn
}
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
      margin: auto 0.25rem;
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
