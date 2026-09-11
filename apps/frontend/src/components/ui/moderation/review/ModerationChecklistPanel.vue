<template>
	<aside
		class="flex shrink-0 flex-col overflow-hidden border-0 border-l border-solid border-divider bg-surface-1 text-sm"
	>
		<!-- Project-wide (no on-screen element) checks -->
		<div v-if="globalStages.length > 0" class="flex shrink-0 flex-col gap-2 p-2">
			<section class="rounded-lg border border-solid border-divider bg-surface-2">
				<h2
					class="m-0 px-2.5 pt-1.5 text-[0.7rem] font-semibold uppercase tracking-wide text-secondary"
				>
					Project-wide
				</h2>
				<ModerationElementFrame element-key="global" bar-only />
			</section>
		</div>

		<div class="h-px w-full shrink-0 bg-divider" />

		<div class="flex min-h-0 flex-1 flex-col">
			<Suspense>
				<ModerationReviewThread class="min-h-0 flex-1" />
				<template #fallback>
					<div class="flex items-center gap-2 p-3 text-secondary">
						<SpinnerIcon class="size-4 animate-spin" /> Loading thread…
					</div>
				</template>
			</Suspense>
		</div>
	</aside>
</template>

<script setup lang="ts">
import { SpinnerIcon } from '@modrinth/assets'
import { computed } from 'vue'

import { injectModerationChecklist } from '~/components/ui/moderation/checklist/checklist-context'

import ModerationElementFrame from './ModerationElementFrame.vue'
import ModerationReviewThread from './ModerationReviewThread.vue'

const engine = injectModerationChecklist()

const globalStages = computed(() => engine.stagesForElement('global'))
</script>
