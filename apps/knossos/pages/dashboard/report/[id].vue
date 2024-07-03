<template>
  <ReportView
    :auth="auth"
    :report-id="route.params.id"
    :breadcrumbs-stack="[{ href: '/dashboard/reports', label: 'Active reports' }]"
  />
</template>
<script setup>
import ReportView from '~/components/ui/report/ReportView.vue'

const route = useNativeRoute()
const auth = await useAuth()

useHead({
  title: `Report ${route.params.id} - Modrinth`,
})
</script>
