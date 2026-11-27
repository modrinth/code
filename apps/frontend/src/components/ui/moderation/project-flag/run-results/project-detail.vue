<template>
	<div class="min-w-0 self-start lg:sticky lg:top-4">
		<section
			class="flex min-w-0 flex-col gap-4 lg:max-h-[calc(100dvh-2rem)] lg:overflow-y-auto"
			aria-label="Project scan results"
		>
			<template v-if="result">
				<p class="m-0 text-sm text-secondary">
					Preview:
					{{
						result.proposed_review_outcome
							? outcomeLabels[result.proposed_review_outcome]
							: 'No proposed review outcome'
					}}
				</p>
				<section v-if="retainedFlags.length" class="flex flex-col gap-2">
					<h3 class="m-0 text-base font-medium text-primary">
						Resolved · Retained ({{ retainedFlags.length }})
					</h3>
					<p class="m-0 text-sm text-secondary">
						These violations preserve their original evidence and still affect the review outcome.
					</p>
					<FlagCard
						v-for="flag in retainedFlags"
						:key="flag.key"
						:flag="flag"
						class="rounded-xl bg-surface-3 p-3"
					/>
				</section>
				<section v-if="result.error !== null" class="flex min-w-0 flex-col gap-3">
					<h3 class="m-0 text-base font-medium text-primary">Scan error</h3>
					<p class="m-0 whitespace-pre-wrap break-words text-sm text-red">
						{{ scanError }}
					</p>
				</section>
				<section
					v-for="section in ruleSections"
					:key="section.label"
					class="flex min-w-0 flex-col gap-3"
				>
					<h3 class="m-0 text-base font-medium text-primary">
						{{ section.label }} ({{ section.rules.length }})
					</h3>
					<p v-if="!section.rules.length" class="m-0 text-sm text-primary">No flagged rules</p>
					<RuleCard
						v-for="[ruleId, rule] in section.rules"
						:key="`${result.project.id}:${ruleId}`"
						:rule="rule"
						:no-findings="section.noFindings"
						:edit-to="{
							query: { ...$route.query, rule: ruleId, ruleProject: result.project.id },
							hash: $route.hash,
						}"
					/>
				</section>
			</template>
			<p v-else class="m-0 text-sm text-secondary">Select a project to see its rule results.</p>
		</section>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRunResult } from '@modrinth/api-client'
import { computed } from 'vue'

import FlagCard from './flag-card.vue'
import RuleCard from './rule-card.vue'

const props = defineProps<{ result?: ProjectFlagRunResult }>()
const outcomeLabels = {
	approved: 'Would approve',
	withheld: 'Would withhold',
	rejected: 'Would reject',
	never_reviewed: 'Would mark as never reviewed',
}
const retainedFlags = computed(
	() => props.result?.effective_flags.filter((flag) => !flag.active) ?? [],
)
const scanError = computed(() =>
	props.result?.error?.replace(/^([A-Za-z0-9]+): /gm, (prefix, ruleId: string) => {
		const rule = props.result?.rules[ruleId]
		return rule ? `${rule.name ?? 'Deleted rule'} (${ruleId}): ` : prefix
	}),
)
const ruleEntries = computed(() => Object.entries(props.result?.rules ?? {}))
const ruleSections = computed(() =>
	[
		{
			label: 'Flagged rules',
			noFindings: false,
			rules: ruleEntries.value.filter(([, rule]) => rule.flags.length > 0 || rule.error !== null),
		},
		{
			label: 'Not flagged',
			noFindings: true,
			rules: ruleEntries.value.filter(([, rule]) => rule.error === null && rule.flags.length === 0),
		},
	].filter((section) => section.label === 'Flagged rules' || section.rules.length > 0),
)
</script>
