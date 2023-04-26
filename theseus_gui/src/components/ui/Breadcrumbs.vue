<template>
  <div class="breadcrumbs">
    <div
      v-for="breadcrumb in $route.meta.breadcrumb"
      :key="breadcrumb.name"
      class="breadcrumbs__item"
    >
      <router-link
        v-if="breadcrumb.link"
        :to="breadcrumb.link.replace('{id}', encodeURIComponent($route.params.id))"
        >{{
          breadcrumb.name.charAt(0) === '?'
            ? breadcrumbs.getName(breadcrumb.name.slice(1))
            : breadcrumb.name
        }}
      </router-link>
      <span v-else>{{
        breadcrumb.name.charAt(0) === '?'
          ? breadcrumbs.getName(breadcrumb.name.slice(1))
          : breadcrumb.name
      }}</span>
      <ChevronRightIcon v-if="breadcrumb.link" class="chevron" />
    </div>
  </div>
</template>

<script setup>
import { ChevronRightIcon } from 'omorphia'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const breadcrumbs = useBreadcrumbs()
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
}
</style>
