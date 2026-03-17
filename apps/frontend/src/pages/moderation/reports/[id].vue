<script setup lang="ts">
import type { Report } from '@modrinth/utils'
import { injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import ModerationReportCard from '~/components/ui/moderation/ModerationReportCard.vue'
import { enrichReportBatch } from '~/helpers/moderation.ts'

const client = injectModrinthClient()
const { params } = useRoute()
const reportId = params.id as string

const { data: report } = useQuery({
	queryKey: computed(() => ['report', reportId]),
	queryFn: async () => {
		try {
			const report = (await client.labrinth.reports_v3.get(reportId)) as Report
			const enrichedReport = (await enrichReportBatch([report]))[0]
			return enrichedReport
		} catch (error) {
			console.error('Error fetching report:', error)
			throw createError({
				statusCode: 404,
				statusMessage: 'Report not found',
			})
		}
	},
})
</script>

<template>
	<div class="flex flex-col gap-3">
		<ModerationReportCard v-if="report" :report="report" />
	</div>
</template>
