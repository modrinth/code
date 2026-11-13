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
						<Accordion
							v-for="(entry, index) in ruleEntries(rule)"
							:key="index"
							:open-by-default="section.open"
							class="min-w-0 rounded-xl bg-surface-3 p-3 text-sm"
							button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-base font-medium [&>div>svg]:size-4 [&>div>svg]:ml-0"
							content-class="flex min-w-0 flex-col gap-2 pt-2"
						>
							<template #title>
								<span class="flex min-w-0 items-start gap-1.5">
									<TagCategoryFlagIcon
										class="mt-1 size-4 shrink-0 text-purple"
										aria-hidden="true"
									/>
									<span class="min-w-0 break-words">{{
										entry.flag?.name || rule.name || (entry.flag ? 'Flag' : 'Deleted rule')
									}}</span>
								</span>
							</template>
							<template v-if="entry.flag">
								<p v-if="entry.flag.reason" class="m-0 whitespace-pre-wrap break-words">
									{{ entry.flag.reason }}
								</p>
								<span
									v-if="entry.flag.location?.review_panel_key"
									class="inline-flex w-fit max-w-full items-center gap-1 rounded-full bg-surface-4 px-2 py-1 text-xs"
								>
									<PanelTopIcon class="size-3.5 shrink-0" aria-hidden="true" />
									<span class="min-w-0 break-words">{{
										entry.flag.location.review_panel_key
									}}</span>
								</span>
								<Accordion
									button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-sm font-medium [&>div>svg]:size-4 [&>div>svg]:ml-0"
									content-class="pt-1"
								>
									<template #title>Flag data</template>
									<pre
										class="m-0 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-xl bg-surface-2"
										>{{ JSON.stringify(entry.flag, null, 2) }}</pre
									>
								</Accordion>
							</template>
							<template v-if="entry.showDiagnostics">
								<div v-if="rule.error !== null" class="flex min-w-0 flex-col gap-1">
									<h5 class="m-0 text-sm font-medium text-contrast">Scan error</h5>
									<p class="m-0 whitespace-pre-wrap break-words text-red">{{ rule.error }}</p>
								</div>
								<Accordion
									v-if="rule.log.length"
									open-by-default
									button-class="m-0 w-full cursor-pointer border-none bg-transparent p-0 text-left text-sm font-medium [&>div>svg]:size-4 [&>div>svg]:ml-0"
									content-class="pt-1"
								>
									<template #title>
										{{ rule.flags.length > 1 ? 'Rule logs' : 'Logs' }}
									</template>
									<pre class="m-0 whitespace-pre-wrap break-words rounded-xl bg-surface-2">{{
										rule.log.join('\n')
									}}</pre>
								</Accordion>
								<p
									v-if="!entry.flag && rule.error === null && !rule.log.length"
									class="m-0 text-sm text-primary"
								>
									No flags
								</p>
							</template>
						</Accordion>
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
import { PanelTopIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Accordion } from '@modrinth/ui'
import { computed } from 'vue'

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
