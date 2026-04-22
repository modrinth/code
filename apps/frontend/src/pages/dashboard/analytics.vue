<template>
	<AnalyticsDashboard />
</template>

<script setup lang="ts">
import AnalyticsDashboard from '~/components/analytics/AnalyticsDashboard.vue'

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Analytics - Modrinth',
})
</script>
