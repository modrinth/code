<template>
  <ReportView
    :report-id="route.params.id"
    :breadcrumbs-stack="[{ href: '/moderation/reports', label: 'Reports' }]"
  />
</template>
<script setup>
import ReportView from '~/components/ui/report/ReportView.vue'

const route = useRoute()

useHead({
  title: `Report ${route.params.id} - Modrinth`,
})
</script>
