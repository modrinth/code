<template>
	<article class="universal-card min-w-0" :aria-label="rule?.name ?? 'New rule'">
		<form class="flex min-w-0 flex-col gap-5" @submit.prevent="saveRule">
			<header class="flex items-start justify-between gap-3">
				<div class="flex min-w-0 flex-1 flex-col gap-2">
					<h2 v-if="!rule" class="m-0 text-lg font-bold text-contrast">New rule</h2>
					<InlineEditableText
						ref="nameInput"
						v-model="name"
						class="self-start text-lg font-semibold text-contrast"
						button-class="cursor-pointer"
						:activation-mode="isSaving || deleting ? 'manual' : 'text'"
						:aria-describedby="nameError ? nameErrorId : undefined"
						edit-label="Edit rule name"
						placeholder="Enter a rule name"
					/>
					<p v-if="nameError" :id="nameErrorId" class="m-0 text-sm text-red">{{ nameError }}</p>
					<p v-if="rule" class="m-0 text-sm text-secondary">
						Created <time :datetime="rule.created_at">{{ formatDate(rule.created_at) }}</time> ·
						Updated <time :datetime="rule.updated_at">{{ formatDate(rule.updated_at) }}</time>
					</p>
				</div>
				<TeleportOverflowMenu
					v-if="rule"
					type="quiet"
					:disabled="deleteDisabled || deleting || hasChanges || isSaving"
					:label="`Options for ${rule.name}`"
					tooltip="Rule options"
					:options="ruleOptions"
				>
					<MoreHorizontalIcon aria-hidden="true" />
				</TeleportOverflowMenu>
			</header>

			<section class="flex min-w-0 flex-col gap-2" :aria-labelledby="expressionHeadingId">
				<div class="flex flex-wrap items-center justify-between gap-2">
					<h3 :id="expressionHeadingId" class="m-0 text-base font-semibold text-contrast">
						Rhai expression
					</h3>
					<ProjectFlagInputSchema>
						<template #reference>
							<a
								href="https://rhai.rs/book/language/comments.html"
								target="_blank"
								rel="noopener noreferrer"
								class="text-sm text-link"
							>
								Rhai reference
							</a>
						</template>
					</ProjectFlagInputSchema>
				</div>
				<RhaiExpressionEditor
					:id="expressionId"
					v-model="expression"
					class="w-full"
					:disabled="isSaving || deleting"
					:error-id="expressionError ? expressionErrorId : undefined"
				/>
				<p v-if="expressionError" :id="expressionErrorId" class="m-0 text-sm text-red">
					{{ expressionError }}
				</p>
			</section>

			<FileScanPatternsEditor
				v-model="patterns"
				:errors="patternErrors"
				:disabled="isSaving || deleting"
			/>

			<details class="rounded-xl border border-solid border-surface-4 p-3" @toggle="openTester">
				<summary class="cursor-pointer font-semibold text-contrast">Test on project</summary>
				<ProjectFlagRuleTester
					v-if="testOpened"
					class="mt-3"
					:params="params"
					:can-test="!expressionError && invalidPatternIndex === -1"
				/>
			</details>

			<p v-if="saveError && hasChanges" role="alert" class="m-0 text-sm text-red">
				Could not save the rule: {{ saveError.message }}
			</p>
			<div
				v-if="!rule || hasChanges || isSaving"
				class="flex flex-wrap justify-end gap-2 border-0 border-t border-solid border-surface-4 pt-4"
			>
				<Button :disabled="isSaving" @click="cancel">Cancel</Button>
				<span v-tooltip="saveDisabledReason" class="inline-flex rounded-xl">
					<Button
						type="colored"
						color="brand"
						native-type="submit"
						:disabled="isSaving || Boolean(saveDisabledReason)"
						:class="{ 'pointer-events-none': saveDisabledReason }"
					>
						<LoaderCircleIcon v-if="isSaving" class="animate-spin" />
						{{ isSaving ? 'Saving…' : 'Save rule' }}
					</Button>
				</span>
			</div>
		</form>
		<slot name="delete-confirmation" />
	</article>
</template>

<script setup lang="ts">
import type { ProjectFlagRule, ProjectFlagRuleParams } from '@modrinth/api-client'
import { LoaderCircleIcon, MoreHorizontalIcon, TrashIcon } from '@modrinth/assets'
import {
	Button,
	type ButtonMenuOption,
	InlineEditableText,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, onMounted, ref, useId, useTemplateRef, watch } from 'vue'

import FileScanPatternsEditor, { type FileScanPattern } from './file-scan-patterns-editor.vue'
import ProjectFlagInputSchema from './input-schema.vue'
import ProjectFlagRuleTester from './rule-tester.vue'
import RhaiExpressionEditor from './rhai-expression-editor.vue'
import { useSaveRule } from './use-rules'

const DEFAULT_EXPRESSION = `print(\`\${project.name}: \${project.summary}\`);
if project.slug == "foo" {
	[#{ key: "illegal_slug", reason: "Project has illegal slug" }]
} else {
	[]
}`

const props = defineProps<{
	rule?: ProjectFlagRule
	deleteDisabled?: boolean
	deleting?: boolean
}>()
const emit = defineEmits<{
	saved: [rule: ProjectFlagRule]
	cancelCreate: []
	dirtyChange: [dirty: boolean]
	requestDelete: []
}>()

const id = useId()
const nameErrorId = `${id}-name-error`
const expressionId = `${id}-expression`
const expressionErrorId = `${id}-expression-error`
const expressionHeadingId = `${id}-expression-heading`
const nameInput = useTemplateRef<InstanceType<typeof InlineEditableText>>('nameInput')
const ruleOptions: ButtonMenuOption[] = [
	{ label: 'Delete rule', icon: TrashIcon, tone: 'red', action: () => emit('requestDelete') },
]
const testOpened = ref(false)
const savedRule = ref(props.rule)
const name = ref('')
const expression = ref('')
const patterns = ref<FileScanPattern[]>([])
restoreFields()

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
	return undefined
})
const params = computed<ProjectFlagRuleParams>(() => ({
	expression: expression.value,
	scan_files_for: Object.fromEntries(
		patterns.value.map((entry) => [entry.name.trim(), entry.pattern]),
	),
}))
const hasChanges = computed(() => {
	const rule = savedRule.value
	if (!rule) return true
	const savedPatterns = rule.params.scan_files_for
	return (
		name.value !== rule.name ||
		expression.value !== rule.params.expression ||
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
	() => props.rule?.id,
	(rule) => {
		savedRule.value = rule
		restoreFields()
		emit('saved', rule)
	},
)

watch(
	() => props.rule,
	(rule) => {
		const keepDraft = hasChanges.value || isSaving.value
		savedRule.value = rule
		if (!keepDraft) restoreFields()
	},
)
watch(
	() => hasChanges.value || isSaving.value,
	(dirty) => emit('dirtyChange', dirty),
)
onBeforeUnmount(() => emit('dirtyChange', false))
onMounted(() => {
	if (!props.rule) nameInput.value?.startEditing()
})

function restoreFields() {
	const rule = savedRule.value
	name.value = rule?.name ?? ''
	expression.value = rule?.params.expression ?? DEFAULT_EXPRESSION
	patterns.value = Object.entries(rule?.params.scan_files_for ?? {}).map(([name, pattern]) => ({
		name,
		pattern,
	}))
}

function cancel() {
	if (!props.rule) {
		emit('cancelCreate')
		return
	}
	restoreFields()
	resetSave()
}

function openTester(event: Event) {
	if ((event.currentTarget as HTMLDetailsElement).open) testOpened.value = true
}

function saveRule() {
	if (!hasChanges.value || saveDisabledReason.value || isSaving.value || props.deleting) return
	save({ name: name.value, params: params.value })
}

function formatDate(value: string) {
	return new Date(value).toLocaleString('en-US', { timeZone: 'UTC', timeZoneName: 'short' })
}
</script>
