<template>
  <div>
    <section class="universal-card">
      <h2>Reports</h2>
      <ReportsList moderation />
    </section>
  </div>
</template>
<script setup>
import ReportsList from '~/components/ui/report/ReportsList.vue'

useHead({
  title: 'Reports - Modrinth',
})
</script>
<style lang="scss" scoped></style>
