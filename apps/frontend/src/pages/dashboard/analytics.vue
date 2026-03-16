<template>
	<div>
		<ChartDisplay :projects="projects ?? undefined" :personal="true" />
	</div>
</template>

<script setup>
import { useQuery } from '@tanstack/vue-query'

import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Analytics - Modrinth',
})

const auth = await useAuth()
const id = auth.value?.user?.id

const { data: projects } = useQuery({
	queryKey: computed(() => ['user', id, 'projects']),
	queryFn: () => useBaseFetch(`user/${id}/projects`),
	enabled: computed(() => !!id),
})
</script>
