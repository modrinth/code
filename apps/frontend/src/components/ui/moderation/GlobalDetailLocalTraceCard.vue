<template>
	<div class="rounded-lg border border-divider bg-bg-raised p-3">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div class="min-w-0">
				<p class="m-0 break-words font-semibold text-contrast">
					{{ trace.project_name }}
				</p>
				<p class="m-0 mt-1 break-all text-sm text-secondary">
					Project {{ trace.project_slug ?? trace.project_id }} / Version
					{{ trace.version_number }} / File {{ trace.file_name }}
				</p>
			</div>
			<div class="flex flex-wrap items-center gap-2">
				<span class="text-sm text-secondary">Local</span>
				<Badge :type="trace.local_status" />
				<span class="text-sm text-secondary">Effective</span>
				<Badge :type="trace.effective_status" />
				<ButtonStyled>
					<NuxtLink :to="localTraceLink">
						<ExternalIcon aria-hidden="true" />
						View
					</NuxtLink>
				</ButtonStyled>
			</div>
		</div>

		<div class="mt-3 grid gap-2 text-sm text-secondary md:grid-cols-2">
			<p class="m-0 break-all">
				<span class="font-semibold text-contrast">Issue</span>
				{{ trace.issue_type }}
			</p>
			<p class="m-0 break-all">
				<span class="font-semibold text-contrast">Severity</span>
				{{ trace.severity }}
			</p>
			<p class="m-0 break-all">
				<span class="font-semibold text-contrast">Path</span>
				{{ trace.file_path }}
			</p>
			<p v-if="trace.jar" class="m-0 break-all">
				<span class="font-semibold text-contrast">JAR</span>
				{{ trace.jar }}
			</p>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ExternalIcon } from '@modrinth/assets'
import { Badge, ButtonStyled } from '@modrinth/ui'

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
