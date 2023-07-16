<template>
  <div>
    <section class="universal-card">
      <h2>Reports you've filed</h2>
      <ReportsList />
    </section>
  </div>
</template>
<script setup>
import ReportsList from '~/components/ui/report/ReportsList.vue'

useHead({
  title: 'Active reports - Modrinth',
})
</script>
<style lang="scss" scoped></style>
