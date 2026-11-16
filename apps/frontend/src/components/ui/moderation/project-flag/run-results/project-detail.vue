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
						<template v-for="(part, index) in errorParts" :key="index">
							<button
								v-if="part.ruleId"
								type="button"
								class="cursor-pointer border-0 bg-transparent p-0 text-red underline [font:inherit]"
								:aria-label="`Edit rule ${part.ruleId}`"
								@click="openRuleEditor(part.ruleId, $event)"
							>{{ part.text }}</button
							><template v-else>{{ part.text }}</template>
						</template>
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
							v-for="(flag, index) in rule.flags.length ? rule.flags : [undefined]"
							:key="index"
							:flag="flag"
							:rule="rule"
							:open-by-default="section.open"
						/>
					</template>
				</section>
			</template>
			<p v-else class="m-0 text-sm text-secondary">
				Project flags appear here when a project is selected.
			</p>
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
import FlagCard from './flag-card.vue'

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

const errorParts = computed(() => {
	const error = props.result?.error ?? ''
	const parts: { text: string; ruleId?: string }[] = []
	let offset = 0
	for (const match of error.matchAll(/(^|\n)(\d+)(?=:)/g)) {
		const start = match.index + match[1].length
		parts.push({ text: error.slice(offset, start) })
		parts.push({ text: match[2], ruleId: match[2] })
		offset = start + match[2].length
	}
	parts.push({ text: error.slice(offset) })
	return parts
})

const selectedRuleSections = computed(() => {
	const rules = Object.entries(props.result?.rules ?? {})
	return [
		{ label: 'Flagged', open: true, rules: rules.filter(([, rule]) => rule.flags.length > 0) },
		{ label: 'Not flagged', open: false, rules: rules.filter(([, rule]) => !rule.flags.length) },
	]
})
</script>
