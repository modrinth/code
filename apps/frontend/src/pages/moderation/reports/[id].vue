<script setup lang="ts">
import type { Report } from '@modrinth/utils'

import ModerationReportCard from '~/components/ui/moderation/ModerationReportCard.vue'
import { enrichReportBatch } from '~/helpers/moderation.ts'

const { params } = useRoute()
const reportId = params.id as string

const { data: report } = await useAsyncData(`moderation-report-${reportId}`, async () => {
	try {
		const report = (await useBaseFetch(`report/${reportId}`, { apiVersion: 3 })) as Report
		const enrichedReport = (await enrichReportBatch([report]))[0]
		return enrichedReport
	} catch (error) {
		console.error('Error fetching report:', error)
		throw createError({
			statusCode: 404,
			statusMessage: 'Report not found',
		})
	}
})
</script>

<template>
	<div class="flex flex-col gap-3">
		<ModerationReportCard v-if="report" :report="report" />
	</div>
</template>
