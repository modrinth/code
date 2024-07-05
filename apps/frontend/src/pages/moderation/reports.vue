<template>
  <div>
    <section class="universal-card">
      <h2>Reports</h2>
      <ReportsList :auth="auth" moderation />
    </section>
  </div>
</template>
<script setup>
import ReportsList from '~/components/ui/report/ReportsList.vue'

const auth = await useAuth()
useHead({
  title: 'Reports - Modrinth',
})
</script>
