<template>
	<div class="min-w-0 self-start lg:sticky lg:top-4">
		<section
			class="flex min-w-0 flex-col gap-4 lg:max-h-[calc(100dvh-2rem)] lg:overflow-y-auto"
			aria-label="Project scan results"
		>
			<template v-if="result">
				<section v-if="result.error !== null" class="flex min-w-0 flex-col gap-3">
					<h3 class="m-0 text-base font-medium text-primary">Scan error</h3>
					<p class="m-0 whitespace-pre-wrap break-words text-sm text-red">
						{{ result.error }}
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
						@edit="openRuleEditor(ruleId, $event)"
					/>
				</section>
			</template>
			<p v-else class="m-0 text-sm text-secondary">Select a project to see its rule results.</p>
		</section>
		<ProjectFlagRuleEditorModal ref="ruleEditor" />
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRunResult } from '@modrinth/api-client'
import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import ProjectFlagRuleEditorModal from '../rules/rule-editor-modal/index.vue'
import RuleCard from './rule-card.vue'

const props = defineProps<{ result?: ProjectFlagRunResult }>()
const ruleEditor = useTemplateRef<InstanceType<typeof ProjectFlagRuleEditorModal>>('ruleEditor')
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

async function openRuleEditor(ruleId: string, event: MouseEvent) {
	try {
		const rule = await queryClient.fetchQuery({
			queryKey: ['moderation', 'project-flag', 'rule', Number(ruleId)] as const,
			queryFn: ({ queryKey: [, , , id] }) => client.labrinth.project_flags_internal.get(id),
			staleTime: 0,
		})
		ruleEditor.value?.show(rule, event)
	} catch (error) {
		addNotification({
			type: 'error',
			title: 'Could not load the rule',
			text: error instanceof Error ? error.message : String(error),
		})
	}
}

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
