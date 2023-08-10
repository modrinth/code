<template>
  <div class="breadcrumbs">
    <Button
      v-if="breadcrumbs.length > 1"
      class="breadcrumbs__back transparent"
      color="primary"
      icon-only
      @click="$router.back()"
    >
      <LeftArrowIcon />
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
import { ChevronRightIcon, Button, LeftArrowIcon } from 'omorphia'
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

  .breadcrumbs__back {
    margin: auto 0;
    color: var(--color-base);
    padding: 0.5rem;
    height: unset;
    width: unset;

    svg {
      width: 1rem;
      height: 1rem;
      min-width: 1rem;
      min-height: 1rem;
      margin: 0;
    }
  }
}

.selected {
  color: var(--color-contrast);
}
</style>
