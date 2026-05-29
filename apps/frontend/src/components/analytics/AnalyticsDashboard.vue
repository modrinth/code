<template>
	<div class="flex touch-manipulation flex-col gap-4 pb-20 lg:pl-4 lg:pt-1.5">
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between gap-2">
				<span class="text-xl font-semibold text-contrast md:text-2xl">
					{{ formatMessage(analyticsMessages.title) }}
				</span>
				<div class="flex flex-wrap items-center justify-end gap-2">
					<ButtonStyled type="transparent">
						<button
							type="button"
							:disabled="isAnalyticsQueryBuilderDefault"
							@click="resetAnalyticsQueryBuilder"
						>
							{{ formatMessage(analyticsMessages.resetButton) }}
						</button>
					</ButtonStyled>
					<ButtonStyled type="outlined">
						<button
							type="button"
							:disabled="projects.length === 0 || !fetchRequest || isRefetching"
							@click="refreshAnalyticsQuery"
						>
							<RefreshCwIcon :class="isRefetching ? 'animate-spin' : ''" />
							{{ formatMessage(analyticsMessages.refreshButton) }}
						</button>
					</ButtonStyled>
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
import { ButtonStyled, injectProjectPageContext, useVIntl } from '@modrinth/ui'

import {
	createAnalyticsDashboardContext,
	provideAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'
import { injectOrganizationContext } from '~/providers/organization-context'

import AnalyticsChart from './analytics-chart/AnalyticsChart.vue'
import { analyticsMessages } from './analytics-messages'
import QueryBuilder from './query-builder/QueryBuilder.vue'
import StatCards from './stat-cards/StatCards.vue'
import AnalyticsTable from './table/AnalyticsTable.vue'

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
