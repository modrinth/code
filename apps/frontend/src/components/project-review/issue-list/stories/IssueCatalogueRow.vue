<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BulletDivider,
	Button,
	Combobox,
	Input,
	MarkdownEditor,
	MultiSelect,
	Textarea,
	Tooltip,
} from '@modrinth/ui'
import { computed } from 'vue'

import ProjectIssueCard from '../../../ui/project-issue-card/index.vue'
import ActionButton from '../../review-panel/controls/action-button.vue'
import { createIssueCatalogue } from './issue-catalogue'

const props = defineProps<{ issueId: string }>()
const panels = createIssueCatalogue()
const issue = computed(() => panels.availableIssues.value.find(({ id }) => id === props.issueId)!)
const preview = computed(() => panels.activeIssues.value.find(({ id }) => id === props.issueId))
const threadIssues = computed<Labrinth.Threads.v3.ThreadIssue[]>(() => {
	const active = preview.value
	if (!active) return []
	return [
		{
			id: props.issueId,
			created_by: null,
			created_at: '2026-01-01T00:00:00.000Z',
			verdict: 'open',
			why: {
				issue_id: props.issueId,
				title: issue.value.title,
				message: panels.message(props.issueId),
				locations: active.locations,
			},
			facets: active.facets.map(({ what }, index) => ({
				id: `${props.issueId}:facet:${index}`,
				what,
				user_addressed: false,
				moderator_verified: false,
				verdict: 'open',
			})),
		},
	]
})
const controlPanels = computed(() => {
	const bindings = new Map(issue.value.controls.map(({ binding }) => [binding.key, binding]))
	return [...bindings.values()].map((binding) => ({
		binding,
		sections: binding.panel.sections
			.map((section) => ({
				...section,
				controls: section.controls.filter(
					(control) => !control.issue || control.issueId === props.issueId,
				),
			}))
			.filter((section) => section.controls.length > 0),
	}))
})
const controls = computed(() =>
	controlPanels.value.flatMap(({ binding, sections }) =>
		sections.flatMap((section) => section.controls.map((control) => ({ binding, control }))),
	),
)

function reset() {
	panels.reset()
	panels.addIssue(props.issueId)
	const first = issue.value.controls.find(({ control }) => control.type === 'toggle')
	if (first) panels.write(first.binding, first.control, true)
	for (const { binding, control } of controls.value) {
		if (control.type === 'select' && !panels.selectValues(binding, control).length) {
			const option = control.options[0]
			if (option) panels.write(binding, control, [option.value])
		}
	}
}

reset()
</script>

<template>
	<article class="issue-catalogue-row grid gap-6 rounded-xl bg-surface-2 p-6">
		<div class="issue-catalogue-preview min-w-0 !max-w-[800px]">
			<div class="mb-4 flex flex-wrap items-center gap-2 text-sm text-secondary">
				<span>{{ issue.category }}</span>
				<BulletDivider />
				<code class="text-xs">{{ issue.id }}</code>
				<template v-if="preview?.issue.suggestedStatus">
					<BulletDivider />
					<span>{{ preview.issue.suggestedStatus }}</span>
				</template>
			</div>
			<ProjectIssueCard
				v-if="panels.message(issueId)"
				:issues="threadIssues"
				show-project-area-link
			/>
			<p v-else class="text-secondary">Choose an option or enter text to preview this message.</p>
		</div>
		<div
			class="issue-catalogue-controls flex min-w-0 flex-col gap-4 border-0 border-t border-solid border-surface-5 pt-4"
		>
			<div class="flex items-center justify-between gap-2">
				<h3 class="m-0 text-base font-bold text-contrast">Issue controls</h3>
				<Button size="sm" @click="reset">Reset</Button>
			</div>
			<section
				v-for="{ binding, sections } in controlPanels"
				:key="binding.key"
				class="flex min-w-0 flex-col gap-2.5 text-sm"
				:aria-label="binding.panel.title ?? binding.key"
			>
				<h3 class="m-0 text-sm font-semibold text-contrast">
					{{ binding.panel.title ?? binding.key }}
				</h3>
				<div v-for="(section, index) in sections" :key="index" class="flex flex-col gap-2">
					<p v-if="section.label" class="m-0 font-semibold text-contrast">{{ section.label }}</p>
					<div class="flex flex-wrap gap-2">
						<template
							v-for="control in section.controls"
							:key="`${control.type}:${control.issueId}:${control.type === 'toggle' ? (control.id ?? '') : control.key}`"
						>
							<Tooltip
								v-if="control.type === 'toggle'"
								:disabled="!control.tooltip"
								:text="control.tooltip"
							>
								<ActionButton
									:label="control.label"
									:model-value="panels.selected(binding, control)"
									:mixed="panels.mixed(binding, control)"
									@update:model-value="panels.write(binding, control, $event)"
								/>
							</Tooltip>
							<div
								v-else
								class="flex w-full flex-col gap-2"
								role="group"
								:aria-label="control.label"
							>
								<p class="m-0 font-semibold text-contrast">
									{{ control.label
									}}<span v-if="control.required" class="text-red" aria-hidden="true"> *</span>
								</p>
								<MultiSelect
									v-if="control.type === 'select' && control.multiple"
									:model-value="panels.selectValues(binding, control)"
									:options="control.options"
									:placeholder="control.placeholder ?? 'Select an option'"
									:aria-label="control.label"
									:aria-required="control.required"
									:dropdown-gap="0"
									class="w-full"
									@update:model-value="panels.write(binding, control, $event)"
								/>
								<Combobox
									v-else-if="control.type === 'select'"
									:model-value="panels.selectValues(binding, control)[0] ?? ''"
									:options="[
										{ value: '', label: control.placeholder ?? 'Select an option' },
										...control.options,
									]"
									:placeholder="control.placeholder ?? 'Select an option'"
									:aria-label="control.label"
									:aria-required="control.required"
									:dropdown-gap="0"
									class="w-full"
									@update:model-value="panels.write(binding, control, $event)"
								/>
								<Input
									v-else-if="control.type === 'text'"
									:model-value="panels.textValue(binding, control)"
									:placeholder="control.placeholder || 'Enter text…'"
									:aria-label="control.label"
									:aria-required="control.required"
									class="w-full"
									@update:model-value="panels.write(binding, control, String($event ?? ''))"
								/>
								<Textarea
									v-else-if="control.type === 'textarea'"
									:model-value="panels.textValue(binding, control)"
									:placeholder="control.placeholder || 'Enter text…'"
									:aria-label="control.label"
									:aria-required="control.required"
									:maxlength="control.maxlength"
									:rows="control.rows"
									resize="vertical"
									class="w-full"
									@update:model-value="panels.write(binding, control, String($event ?? ''))"
								/>
								<MarkdownEditor
									v-else-if="control.type === 'markdown'"
									:model-value="panels.textValue(binding, control)"
									:heading-buttons="false"
									:max-height="240"
									:min-height="72"
									:placeholder="control.placeholder ?? 'Explain what needs to change…'"
									:aria-label="control.label"
									hide-markdown-hint
									class="w-full"
									@update:model-value="panels.write(binding, control, $event)"
								/>
								<p
									v-if="panels.missing(binding, control)"
									class="m-0 ml-0.5 text-xs text-orange"
									role="status"
								>
									Complete this required field.
								</p>
							</div>
						</template>
					</div>
				</div>
			</section>
		</div>
	</article>
</template>

<style scoped>
:global(body:has(.issue-catalogue-row)) {
	overflow: visible !important;
}

@media (min-width: 700px) {
	.issue-catalogue-row {
		grid-template-columns: minmax(0, 1fr) 340px;
	}

	.issue-catalogue-preview {
		position: sticky;
		top: 1.5rem;
		align-self: start;
	}

	.issue-catalogue-controls {
		border-top: 0;
		border-left: 1px solid var(--surface-5);
		padding-top: 0;
		padding-left: 1.5rem;
	}
}
</style>
