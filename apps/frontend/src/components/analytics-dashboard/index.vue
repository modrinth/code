<template>
	<div class="flex touch-manipulation flex-col gap-4 pb-20 lg:pl-4 lg:pt-1.5">
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between gap-2">
				<span class="text-xl font-semibold text-contrast md:text-2xl">
					{{ formatMessage(analyticsMessages.title) }}
				</span>
				<div class="flex flex-wrap items-center justify-end gap-2">
					<Button
						type="quiet"
						native-type="button"
						:disabled="isAnalyticsQueryBuilderDefault"
						@click="resetAnalyticsQueryBuilder"
					>
						{{ formatMessage(analyticsMessages.resetButton) }}
					</Button>
					<Button
						type="outlined"
						native-type="button"
						:disabled="projects.length === 0 || !fetchRequest || isRefetching"
						@click="refreshAnalyticsQuery"
					>
						<RefreshCwIcon :class="isRefetching ? 'animate-spin' : ''" />
						{{ formatMessage(analyticsMessages.refreshButton) }}
					</Button>
				</div>
			</div>
			<QueryBuilder />
		</div>
		<StatCards />
		<AnalyticsChart />
		<AnalyticsTable />
	</div>
</template>

<script setup lang="ts">
import { RefreshCwIcon } from '@modrinth/assets'
import { Button, injectProjectPageContext, useVIntl } from '@modrinth/ui'

import {
	createAnalyticsDashboardContext,
	provideAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'
import { injectOrganizationContext } from '~/providers/organization-context'

import AnalyticsChart from './analytics-chart/index.vue'
import { analyticsMessages } from './analytics-messages.ts'
import AnalyticsTable from './analytics-table/index.vue'
import QueryBuilder from './query-builder/index.vue'
import StatCards from './stat-cards/StatCards.vue'

const auth = await useAuth()
const { formatMessage } = useVIntl()
const projectPageContext = injectProjectPageContext(null)
const organizationContext = injectOrganizationContext(null)

const analyticsDashboardContext = createAnalyticsDashboardContext({
	auth,
	projectPageContext,
	organizationContext,
})
const {
	fetchRequest,
	isAnalyticsQueryBuilderDefault,
	isRefetching,
	projects,
	refreshAnalyticsQuery,
	resetAnalyticsQueryBuilder,
} = analyticsDashboardContext

provideAnalyticsDashboardContext(analyticsDashboardContext)
</script>
