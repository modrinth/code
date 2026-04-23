<template>
	<div class="flex flex-col gap-4">
		<QueryBuilder />
		<StatCards />
		<AnalyticsGraph />
		<AnalyticsTable />
	</div>
</template>

<script setup lang="ts">
import { injectProjectPageContext } from '@modrinth/ui'

import { createAnalyticsDashboardContext, provideAnalyticsDashboardContext } from '~/providers/analytics/analytics'
import { injectOrganizationContext } from '~/providers/organization-context'

import AnalyticsGraph from './graph/AnalyticsGraph.vue'
import QueryBuilder from './query-builder/QueryBuilder.vue'
import StatCards from './stat-cards/StatCards.vue'
import AnalyticsTable from './table/AnalyticsTable.vue'

const auth = await useAuth()
const projectPageContext = injectProjectPageContext(null)
const organizationContext = injectOrganizationContext(null)

const analyticsDashboardContext = createAnalyticsDashboardContext({
	auth,
	projectPageContext,
	organizationContext,
})

provideAnalyticsDashboardContext(analyticsDashboardContext)
</script>
