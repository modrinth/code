<template>
	<div>
		<Suspense>
			<ChartDisplay :projects="projects" :personal="true" />
			<template #fallback>
				<div class="universal-card">
					<h2><span class="label__title">Loading analytics...</span></h2>
				</div>
			</template>
		</Suspense>
	</div>
</template>

<script setup>
import {
	commonProjectSettingsMessages,
	injectIcarusClient,
	useDebugLogger,
	useVIntl,
} from '@icarus/ui'

import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

const { formatMessage } = useVIntl()

const debug = useDebugLogger('analytics.vue')

definePageMeta({
	middleware: 'auth',
})

useHead({
	title: () => `${formatMessage(commonProjectSettingsMessages.analytics)} - Icarus`,
})

const auth = await useAuth()
const client = injectIcarusClient()
const id = auth.value?.user?.id

debug('auth resolved', { id })

const projects = await client.labrinth.users_v2.getProjects(id)

debug('projects resolved', { count: projects?.length })
</script>

