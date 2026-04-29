<template>
	<div class="flex flex-col gap-4 pb-20 lg:pl-4 lg:pt-1.5">
		<div class="flex flex-col gap-2">
			<div class="flex justify-between">
				<span class="text-xl font-semibold text-contrast md:text-2xl">Analytics</span>
				<ButtonStyled type="outlined">
					<button
						type="button"
						:disabled="!fetchRequest || isRefetching"
						@click="refreshAnalyticsQuery"
					>
						<RefreshCwIcon :class="isRefetching ? 'animate-spin' : ''" />
						Refresh
					</button>
				</ButtonStyled>
			</div>
			<QueryBuilder />
		</div>
		<StatCards />
		<AnalyticsGraph />
		<AnalyticsTable />
	</div>
</template>

<script setup lang="ts">
import { ButtonStyled, injectProjectPageContext } from '@modrinth/ui'

import {
	createAnalyticsDashboardContext,
	provideAnalyticsDashboardContext,
} from '~/providers/analytics/analytics'
import { injectOrganizationContext } from '~/providers/organization-context'

import { RefreshCwIcon } from '@modrinth/assets'
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
const { fetchRequest, isRefetching, refreshAnalyticsQuery } = analyticsDashboardContext

provideAnalyticsDashboardContext(analyticsDashboardContext)
</script>
