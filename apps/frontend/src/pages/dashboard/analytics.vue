<template>
	<div>
		<ChartDisplay :projects="projects ?? undefined" :personal="true" />
	</div>
</template>

<script setup>
import { commonMessages, injectModrinthClient, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

const { formatMessage } = useVIntl()

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonMessages.analyticsButton)} - Modrinth`,
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
