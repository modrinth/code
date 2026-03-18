<template>
	<div>
		<ChartDisplay :projects="projects ?? undefined" :personal="true" />
	</div>
</template>

<script setup>
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: 'Analytics - Modrinth',
})

const auth = await useAuth()
const client = injectModrinthClient()
const id = auth.value?.user?.id

const { data: projects } = useQuery({
	queryKey: computed(() => ['user', id, 'projects']),
	queryFn: () => client.labrinth.users_v2.getProjects(id),
	enabled: computed(() => !!id),
})
</script>
