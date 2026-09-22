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
							:ref="(field) => setFieldRef(fieldKey(control), field)"
							:disabled="control.disabled"
							:model-value="panels.textValue(panelBinding, control)"
							:heading-buttons="false"
							:max-height="300"
							@update:model-value="panelBinding && panels.write(panelBinding, control, $event)"
						/>
						<Input
							v-else-if="control.type === 'text'"
							:ref="(field) => setFieldRef(fieldKey(control), field)"
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
							:keybind="keybinds.get(control)"
							:show-keybind-hint="settings.get(moderationSettings.General.ShowToggleIssueButtonShortcutHint)"
							:disabled="control.disabled"
							:model-value="panels.selected(panelBinding, control)"
							:aria-pressed="panels.selected(panelBinding, control)"
							@update:model-value="toggleAction(control, $event)"
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
			v-for="(correction, correctionIndex) in panels.correctionPanels.value"
			:key="correction.key"
			class="flex flex-col gap-2"
		>
			<p class="m-0 font-semibold text-contrast">{{ correction.panel.title }}</p>
			<Controls
				:target="target"
				:binding="correction"
				:keybind-offset="correctionOffsets[correctionIndex]"
				@dropdown-open="emit('dropdown-open')"
				@dropdown-close="emit('dropdown-close')"
			/>
		</div>
	</div>
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
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
import { type ComponentPublicInstance, computed, nextTick, useId } from 'vue'

import type { ReviewTarget } from '~/providers/project-review/review'
import { useModerationSettings } from '~/composables/moderation'
import {
	injectReviewPanels,
	type ReviewPanelBinding,
} from '~/providers/project-review/review-panels'

import { projectReviewMessages as messages } from '../messages'

const props = defineProps<{
	target: ReviewTarget
	binding?: ReviewPanelBinding
	keybindOffset?: number
}>()
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
const settings = useModerationSettings()
const panelBinding = computed(() => props.binding ?? panels.resolve(props.target))
type PanelControl = ReviewPanelBinding['panel']['sections'][number]['controls'][number]
const fields = new Map<string, { focus: () => void }>()
let pendingFocus: string | undefined

function fieldKey(control: { issueId: string; key: string }) {
	return `${control.issueId}:${control.key}`
}

async function focusPendingField() {
	await nextTick()
	if (!pendingFocus) return
	const field = fields.get(pendingFocus)
	if (!field) return
	pendingFocus = undefined
	field.focus()
}

function setFieldRef(key: string, field: Element | ComponentPublicInstance | null) {
	if (field && 'focus' in field && typeof field.focus === 'function') {
		fields.set(key, field as { focus: () => void })
		if (pendingFocus === key) void focusPendingField()
	} else {
		fields.delete(key)
	}
}

function textFields() {
	return (
		panelBinding.value?.panel.sections.flatMap((section) =>
			section.controls.filter(
				(control): control is Extract<PanelControl, { type: 'text' | 'markdown' }> =>
					(control.type === 'text' || control.type === 'markdown') && !control.disabled,
			),
		) ?? []
	)
}

function toggleAction(control: PanelControl, value: boolean) {
	const binding = panelBinding.value
	if (!binding) return
	const previousFields = new Set(textFields().map(fieldKey))
	pendingFocus = undefined
	panels.write(binding, control, value)
	const revealed = textFields().find((field) => !previousFields.has(fieldKey(field)))
	if (!revealed) return
	pendingFocus = fieldKey(revealed)
	void focusPendingField()
}

function toggleControls(binding?: ReviewPanelBinding) {
	return (
		binding?.panel.sections.flatMap((section) =>
			section.controls.filter((control) => control.type === 'toggle'),
		) ?? []
	)
}
const keybinds = computed(() =>
	new Map(
		toggleControls(panelBinding.value).map((control, index) => [
			control,
			actionKeybind(index + (props.keybindOffset ?? 0)),
		]),
	),
)
function actionKeybind(index: number) {
	if (index >= 20) return undefined
	return `${index >= 10 ? 'Shift+' : ''}${'1234567890'[index % 10]}`
}
const correctionOffsets = computed(() => {
	let offset = (props.keybindOffset ?? 0) + toggleControls(panelBinding.value).length
	return panels.correctionPanels.value.map((binding) => {
		const start = offset
		offset += toggleControls(binding).length
		return start
	})
})
const hasToggles = computed(() =>
	panelBinding.value?.panel.sections.some((section) =>
		section.controls.some((control) => control.type === 'toggle'),
	),
)
</script>
