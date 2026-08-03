<template>
	<div class="rounded-lg border border-divider bg-bg-raised p-3">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div class="min-w-0">
				<p class="m-0 break-words font-semibold text-contrast">
					{{ trace.project_name }}
				</p>
				<p class="m-0 mt-1 text-sm text-secondary">
					<IssueDetailPath :segments="[trace.version_number, trace.file_name, trace.jar]" />
				</p>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<template v-if="trace.local_status !== 'pending'">
					<span class="text-sm text-secondary">Local</span>
					<Badge :type="trace.local_status" />
				</template>
				<ButtonStyled>
					<NuxtLink :to="localTraceLink">
						<ExternalIcon aria-hidden="true" />
						View
					</NuxtLink>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon } from '@modrinth/assets'
import { Badge, ButtonStyled } from '@modrinth/ui'

import IssueDetailPath from '~/components/ui/moderation/IssueDetailPath.vue'

const props = defineProps<{
	trace: Labrinth.TechReview.Internal.GlobalIssueDetailTrace
}>()

const localTraceLink = computed(
	() =>
		`/moderation/technical-review/${props.trace.project_id}?detail=${encodeURIComponent(
			props.trace.detail_id,
		)}`,
)
</script>
