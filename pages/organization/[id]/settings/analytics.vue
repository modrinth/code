<template>
  <div class="normal-page__content">
    <div class="universal-card">
      <h2>Analytics</h2>

      <p>
        This page shows you the analytics for your organization's projects. You can see the number
        of downloads, page views and revenue earned for all of your projects, as well as the total
        downloads and page views for each project by country.
      </p>
    </div>

    <ChartDisplay :projects="projects.map((x) => ({ title: x.name, ...x }))" />
  </div>
</template>

<script setup>
import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

const { projects } = inject('organizationContext')
</script>

<style scoped lang="scss">
.markdown-body {
  margin-bottom: var(--gap-md);
}
</style>
