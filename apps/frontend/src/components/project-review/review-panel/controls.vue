<template>
	<div v-if="panelBinding && hasToggles" class="flex flex-col gap-3">
		<div
			v-for="(section, index) in panelBinding.panel.sections"
			:key="`${panelBinding.projectId}:${panelBinding.key}:${index}`"
			class="flex flex-col gap-2"
		>
			<p v-if="section.label" class="m-0 font-semibold text-contrast">
				{{ section.label }}
			</p>
			<div class="flex flex-wrap gap-2">
				<template
					v-for="control in section.controls"
					:key="`${control.type}:${control.issueId}:${control.type === 'toggle' ? (control.id ?? '') : control.key}`"
				>
					<div
						v-if="control.type !== 'toggle'"
						class="flex w-full flex-col gap-2"
						role="group"
						:aria-label="control.label"
						:aria-describedby="
							panels.missing(panelBinding, control)
								? `${id}-${control.issueId}-${control.key}-required`
								: undefined
						"
					>
						<p class="m-0 font-semibold text-contrast">
							{{ control.label }}
						</p>
						<MarkdownEditor
							v-if="control.type === 'markdown'"
							:disabled="control.disabled"
							:model-value="panels.textValue(panelBinding, control)"
							:heading-buttons="false"
							:max-height="300"
							@update:model-value="panelBinding && panels.write(panelBinding, control, $event)"
						/>
						<Input
							v-else-if="control.type === 'text'"
							:model-value="panels.textValue(panelBinding, control)"
							:placeholder="control.placeholder"
							:disabled="control.disabled"
							:aria-label="control.label"
							:aria-required="control.required"
							@update:model-value="
								panelBinding && panels.write(panelBinding, control, String($event ?? ''))
							"
						/>
						<MultiSelect
							v-else-if="control.type === 'select' && control.multiple"
							:model-value="panels.selectValues(panelBinding, control)"
							:options="control.options"
							:placeholder="control.placeholder ?? formatMessage(controlMessages.select)"
							:disabled="control.disabled"
							:aria-label="control.label"
							:aria-required="control.required"
							:dropdown-gap="0"
							@open="emit('dropdown-open')"
							@close="emit('dropdown-close')"
							@update:model-value="panelBinding && panels.write(panelBinding, control, $event)"
						/>
						<Combobox
							v-else-if="control.type === 'select'"
							:model-value="panels.selectValues(panelBinding, control)[0] ?? ''"
							:options="[
								{ value: '', label: control.placeholder ?? formatMessage(controlMessages.select) },
								...control.options,
							]"
							:placeholder="control.placeholder ?? formatMessage(controlMessages.select)"
							:disabled="control.disabled"
							:aria-label="control.label"
							:aria-required="control.required"
							:dropdown-gap="0"
							@open="emit('dropdown-open')"
							@close="emit('dropdown-close')"
							@update:model-value="panelBinding && panels.write(panelBinding, control, $event)"
						/>
					</div>
					<Tooltip v-else :disabled="!control.tooltip" :text="control.tooltip">
						<ActionButton
							:label="control.label"
							:disabled="control.disabled"
							:model-value="panels.selected(panelBinding, control)"
							:aria-pressed="panels.selected(panelBinding, control)"
							@update:model-value="panelBinding && panels.write(panelBinding, control, $event)"
						/>
					</Tooltip>
				</template>
			</div>
		</div>
	</div>
	<p v-else class="m-0 text-base text-secondary">
		{{ formatMessage(panelBinding ? messages.noIssues : messages.noReviewActions) }}
	</p>
	<div
		v-if="!binding && target.kind === 'status-alerts' && panels.correctionsRequested.value"
		class="flex flex-col gap-3"
	>
		<p class="m-0 text-secondary">{{ formatMessage(controlMessages.corrections) }}</p>
		<p v-if="!panels.correctionPanels.value.length" class="m-0 text-orange" role="status">
			{{ formatMessage(controlMessages.noCorrections) }}
		</p>
		<p v-if="panels.corrections.value.conflicts.length" class="m-0 text-red" role="alert">
			{{ formatMessage(controlMessages.conflicts) }}
		</p>
		<div
			v-for="correction in panels.correctionPanels.value"
			:key="correction.key"
			class="flex flex-col gap-2"
		>
			<p class="m-0 font-semibold text-contrast">{{ correction.panel.title }}</p>
			<Controls
				:target="target"
				:binding="correction"
				@dropdown-open="emit('dropdown-open')"
				@dropdown-close="emit('dropdown-close')"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import ActionButton from '@modrinth/moderation/src/types/node/components/ActionButton.vue'
import {
	Combobox,
	defineMessages,
	Input,
	MarkdownEditor,
	MultiSelect,
	Tooltip,
	useVIntl,
} from '@modrinth/ui'
import { computed, useId } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import {
	injectReviewPanels,
	type ReviewPanelBinding,
} from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'

const props = defineProps<{ target: ReviewTarget; binding?: ReviewPanelBinding }>()
const emit = defineEmits<{
	'dropdown-open': []
	'dropdown-close': []
}>()
const id = useId()
const controlMessages = defineMessages({
	corrections: {
		id: 'project-review.controls.corrections',
		defaultMessage: 'These corrections will be applied before sending your reply.',
	},
	noCorrections: {
		id: 'project-review.controls.no-corrections',
		defaultMessage: 'No automatic corrections are selected.',
	},
	conflicts: {
		id: 'project-review.controls.correction-conflicts',
		defaultMessage: 'Resolve conflicting corrections before sending your reply.',
	},
	select: {
		id: 'project-review.controls.select',
		defaultMessage: 'Select an option',
	},
	required: {
		id: 'project-review.controls.required-value',
		defaultMessage: 'Complete this required field.',
	},
})
const { formatMessage } = useVIntl()
const panels = injectReviewPanels()
const panelBinding = computed(() => props.binding ?? panels.resolve(props.target))
const hasToggles = computed(() =>
	panelBinding.value?.panel.sections.some((section) =>
		section.controls.some((control) => control.type === 'toggle'),
	),
)
</script>
