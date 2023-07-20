<template>
  <div>
    <section class="universal-card">
      <h2>Reports you've filed</h2>
      <ReportsList :auth="auth" />
    </section>
  </div>
</template>
<script setup>
import ReportsList from '~/components/ui/report/ReportsList.vue'

const auth = await useAuth()
useHead({
  title: 'Active reports - Modrinth',
})
</script>
<style lang="scss" scoped></style>
