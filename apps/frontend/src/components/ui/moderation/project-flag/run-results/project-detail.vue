<template>
	<div class="min-w-0 self-start lg:sticky lg:top-4">
		<section
			class="flex min-w-0 flex-col gap-4 lg:max-h-[calc(100dvh-2rem)] lg:overflow-y-auto"
			aria-labelledby="run-flags-heading"
		>
			<h3 id="run-flags-heading" class="sr-only">Flags</h3>
			<template v-if="result">
				<section v-if="result.error !== null" class="flex min-w-0 flex-col gap-2">
					<h4 class="m-0 font-semibold text-primary">Scan errors</h4>
					<p class="m-0 whitespace-pre-wrap break-words text-sm text-red">
						{{ result.error }}
					</p>
				</section>
				<section
					v-for="section in selectedRuleSections"
					:key="section.label"
					class="flex min-w-0 flex-col gap-3"
				>
					<h4 class="m-0 text-base font-medium text-primary">
						{{ section.label }} ({{ section.rules.length }})
					</h4>
					<p v-if="!section.rules.length" class="m-0 text-sm text-primary">No flags</p>
					<template v-for="[ruleId, rule] in section.rules" :key="ruleId">
						<FlagCard
							v-for="(entry, index) in ruleEntries(rule)"
							:key="index"
							:flag="entry.flag"
							:rule="rule"
							:show-diagnostics="entry.showDiagnostics"
							:open-by-default="section.open"
						/>
					</template>
				</section>
			</template>
			<p v-else class="m-0 text-sm text-secondary">
				Project flags appear here when a project is selected.
			</p>
		</section>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlag, ProjectFlagRuleResult, ProjectFlagRunResult } from '@modrinth/api-client'
import { computed } from 'vue'

import FlagCard from './flag-card.vue'

const props = defineProps<{ result?: ProjectFlagRunResult }>()

const selectedRuleSections = computed(() => {
	const rules = Object.entries(props.result?.rules ?? {})
	return [
		{ label: 'Flagged', open: true, rules: rules.filter(([, rule]) => rule.flags.length > 0) },
		{ label: 'Not flagged', open: false, rules: rules.filter(([, rule]) => !rule.flags.length) },
	]
})

function ruleEntries(rule: ProjectFlagRuleResult) {
	const entries: { flag?: ProjectFlag; showDiagnostics: boolean }[] = rule.flags.map((flag) => ({
		flag,
		showDiagnostics: rule.flags.length === 1,
	}))
	if (
		rule.flags.length === 0 ||
		(rule.flags.length > 1 && (rule.error !== null || rule.log.length > 0))
	) {
		entries.push({ showDiagnostics: true })
	}
	return entries
}
</script>
