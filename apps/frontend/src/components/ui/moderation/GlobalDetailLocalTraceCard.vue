<template>
	<div class="rounded-lg border border-divider bg-bg-raised p-3">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div class="min-w-0">
				<p class="m-0 break-words font-semibold text-contrast">
					{{ trace.project_name }}
				</p>
				<p class="m-0 mt-1 flex flex-wrap items-center gap-1 text-sm text-secondary">
					<span class="break-all">{{ trace.version_number }}</span>
					<ChevronRightIcon class="size-4 shrink-0" aria-hidden="true" />
					<span class="break-all">{{ decodeTracePath(trace.file_name) }}</span>
					<template v-if="trace.jar">
						<ChevronRightIcon class="size-4 shrink-0" aria-hidden="true" />
						<span class="break-all">{{ decodeTracePath(trace.jar) }}</span>
					</template>
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
import { ChevronRightIcon, ExternalIcon } from '@modrinth/assets'
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

function decodeTracePath(path: string): string {
	try {
		return decodeURIComponent(path)
	} catch {
		return path
	}
}
</script>
