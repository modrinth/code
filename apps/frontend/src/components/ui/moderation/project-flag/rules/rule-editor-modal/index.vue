<template>
	<NewModal
		ref="modal"
		:header="rule ? 'Edit rule' : 'Create rule'"
		class="h-[calc(100dvh-2*var(--gap-lg))]"
		width="82rem"
		max-width="82rem"
		max-content-height="none"
		scrollable
		actions-divider
		:close-on-click-outside="false"
		:disable-close="isSaving && !saveComplete"
		:initial-focus="() => form?.querySelector<HTMLInputElement>('input') ?? null"
	>
		<div
			class="grid gap-6 lg:h-full lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)] lg:grid-rows-[minmax(0,1fr)]"
		>
			<form
				:id="formId"
				ref="form"
				class="flex min-w-0 flex-col gap-5 lg:-m-1 lg:overflow-y-auto lg:p-1"
				@submit.prevent="saveRule"
			>
				<div class="flex min-w-0 shrink-0 flex-col gap-2">
					<label :for="nameId" class="font-semibold text-contrast">Rule name</label>
					<Input
						:id="nameId"
						v-model="name"
						:disabled="isSaving"
						:error="Boolean(nameError)"
						:aria-describedby="nameError ? nameErrorId : undefined"
						placeholder="Enter a rule name"
					/>
					<p v-if="nameError" :id="nameErrorId" class="m-0 text-sm text-red">{{ nameError }}</p>
				</div>
				<FileScanPatternsEditor
					v-model="patterns"
					class="shrink-0"
					:errors="patternErrors"
					:disabled="isSaving"
				/>
				<ProjectFlagInputSchema
					v-model:project-id="schemaProjectId"
					class="min-h-40 flex-1 max-lg:max-h-96"
				/>
			</form>

			<div class="flex min-w-0 flex-col gap-5">
				<section class="flex min-w-0 flex-1 flex-col gap-3" aria-label="Rhai expression">
					<div class="flex flex-wrap items-center justify-between gap-2">
						<h3 class="m-0 text-base font-semibold text-contrast">Rhai expression</h3>
						<a
							href="https://rhai.rs/book/language/comments.html"
							target="_blank"
							rel="noopener noreferrer"
							class="inline-flex items-center gap-1.5 text-sm text-link"
						>
							Rhai reference <ExternalIcon class="size-4" aria-hidden="true" />
						</a>
					</div>
					<RhaiExpressionEditor
						:id="expressionId"
						v-model="expression"
						class="h-80 lg:h-auto lg:min-h-24 lg:flex-1"
						:disabled="isSaving"
						:error-id="expressionError ? expressionErrorId : undefined"
					/>
					<p v-if="expressionError" :id="expressionErrorId" class="m-0 text-sm text-red">
						{{ expressionError }}
					</p>
				</section>
				<Accordion
					class="min-w-0 shrink-0 rounded-2xl border border-solid border-surface-4 bg-surface-2 p-4"
					button-class="m-0 w-full cursor-pointer border-0 bg-transparent p-0 text-left text-base font-semibold"
					content-class="pt-3 lg:max-h-[min(16rem,25vh)] lg:overflow-y-auto"
				>
					<template #title>Test on project</template>
					<ProjectFlagRuleTester
						:params="params"
						:can-test="!expressionError && invalidPatternIndex === -1"
					/>
				</Accordion>
			</div>
		</div>

		<template #actions>
			<div class="flex flex-wrap items-center justify-end gap-2">
				<Button :disabled="isSaving" @click="modal?.hide()">Cancel</Button>
				<span v-tooltip="saveDisabledReason" class="inline-flex rounded-xl">
					<Button
						type="colored"
						color="brand"
						native-type="submit"
						:form="formId"
						:disabled="isSaving || Boolean(saveDisabledReason)"
						:class="{ 'pointer-events-none': saveDisabledReason }"
					>
						<LoaderCircleIcon v-if="isSaving" class="animate-spin" />
						{{ isSaving ? 'Saving…' : 'Save rule' }}
					</Button>
				</span>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { ProjectFlagRule, ProjectFlagRuleParams } from '@modrinth/api-client'
import { ExternalIcon, LoaderCircleIcon } from '@modrinth/assets'
import { Accordion, Button, Input, NewModal } from '@modrinth/ui'
import { computed, nextTick, ref, shallowRef, useId, useTemplateRef } from 'vue'

import FileScanPatternsEditor, { type FileScanPattern } from './file-scan-patterns-editor.vue'
import ProjectFlagInputSchema from './input-schema/index.vue'
import RhaiExpressionEditor from './expression-editor.vue'
import ProjectFlagRuleTester from './tester.vue'
import { useSaveRule } from '../use-rules'

const DEFAULT_EXPRESSION = `print(\`\${project.name}: \${project.summary}\`);
if project.slug == "foo" {
	[#{ key: "illegal_slug", reason: "Project has illegal slug" }]
} else {
	[]
}`

const emit = defineEmits<{ created: [] }>()
const id = useId()
const formId = `${id}-form`
const nameId = `${id}-name`
const nameErrorId = `${id}-name-error`
const expressionId = `${id}-expression`
const expressionErrorId = `${id}-expression-error`
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const form = useTemplateRef<HTMLFormElement>('form')
const rule = shallowRef<ProjectFlagRule>()
const name = ref('')
const schemaProjectId = ref<string>()
const expression = ref('')
const patterns = ref<FileScanPattern[]>([])
const saveComplete = ref(false)

const nameError = computed(() => {
	if (!name.value.trim()) return 'Enter a rule name.'
	if (Array.from(name.value).length > 1023) return 'The name must be at most 1,023 characters.'
	return ''
})
const expressionError = computed(() => {
	if (!expression.value.trim()) return 'Enter a Rhai expression.'
	if (Array.from(expression.value).length > 65535) {
		return 'The expression must be at most 65,535 characters.'
	}
	return ''
})
const patternErrors = computed(() => {
	const names = patterns.value.map((entry) => entry.name.trim())
	return patterns.value.map((entry, index) => {
		const name = names[index]
		return {
			name: !name
				? 'Enter a pattern name.'
				: names.indexOf(name) !== names.lastIndexOf(name)
					? 'Each pattern must have a unique name.'
					: '',
			pattern: !entry.pattern.trim() ? 'Enter a regex pattern.' : '',
		}
	})
})
const invalidPatternIndex = computed(() =>
	patternErrors.value.findIndex((error) => error.name || error.pattern),
)
const saveDisabledReason = computed(() => {
	if (nameError.value) return nameError.value
	if (expressionError.value) return expressionError.value
	const index = invalidPatternIndex.value
	if (index !== -1) {
		const error = patternErrors.value[index]
		return `Pattern ${index + 1}: ${error.name || error.pattern}`
	}
	return hasChanges.value ? undefined : 'Make a change to save this rule.'
})
const params = computed<ProjectFlagRuleParams>(() => ({
	expression: expression.value,
	scan_files_for: Object.fromEntries(
		patterns.value.map((entry) => [entry.name.trim(), entry.pattern]),
	),
}))
const hasChanges = computed(() => {
	const original = rule.value
	if (!original) return true
	const savedPatterns = original.params.scan_files_for
	return (
		name.value !== original.name ||
		expression.value !== original.params.expression ||
		patterns.value.length !== Object.keys(savedPatterns).length ||
		new Set(patterns.value.map((entry) => entry.name)).size !== patterns.value.length ||
		patterns.value.some(
			(entry) =>
				!Object.hasOwn(savedPatterns, entry.name) || savedPatterns[entry.name] !== entry.pattern,
		)
	)
})

const {
	mutate: save,
	isPending: isSaving,
	error: saveError,
	reset: resetSave,
} = useSaveRule(
	() => rule.value?.id,
	async () => {
		if (!rule.value) emit('created')
		saveComplete.value = true
		await nextTick()
		modal.value?.hide()
	},
)

function show(original: ProjectFlagRule | undefined, event: MouseEvent) {
	if (isSaving.value) return
	resetSave()
	rule.value = original
	saveComplete.value = false
	name.value = original?.name ?? ''
	schemaProjectId.value = undefined
	expression.value = original?.params.expression ?? DEFAULT_EXPRESSION
	patterns.value = Object.entries(original?.params.scan_files_for ?? {}).map(([name, pattern]) => ({
		name,
		pattern,
	}))
	modal.value?.show(event)
}

function saveRule() {
	if (saveDisabledReason.value || isSaving.value) return
	save({ name: name.value, params: params.value })
}

defineExpose({ show })
useFlagErrorNotification(saveError, 'Could not save the rule')
</script>
