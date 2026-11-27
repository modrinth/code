<template>
	<div class="flex min-w-0 flex-col bg-surface-2 lg:h-dvh lg:min-h-[36rem]">
		<header
			class="flex shrink-0 flex-wrap items-center justify-between gap-x-5 gap-y-2 border-0 border-b border-solid border-surface-4 px-4 py-3"
		>
			<div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-2">
				<button
					type="button"
					class="inline-flex shrink-0 cursor-pointer items-center gap-1 border-0 bg-transparent p-0 text-sm font-medium text-secondary hover:text-contrast"
					:disabled="isSaving"
					@click="emit('close')"
				>
					<ArrowLeftIcon class="size-4" aria-hidden="true" />
					{{ backLabel }}
				</button>
				<h1 class="m-0 min-w-0 max-w-sm text-base font-bold text-contrast">
					<button
						v-if="!editingName"
						ref="nameButton"
						type="button"
						class="group flex max-w-full cursor-pointer items-center gap-1.5 rounded-lg border-0 bg-transparent px-2 py-1 text-left font-[inherit] text-lg transition-all hover:bg-surface-3 focus-visible:bg-surface-3"
						:disabled="isSaving"
						:aria-label="`Edit rule name: ${name}`"
						@click="editName"
					>
						<span class="min-w-0 truncate">{{ name || 'Untitled rule' }}</span>
						<EditIcon
							class="size-4 shrink-0 text-secondary opacity-0 group-hover:opacity-100 group-focus-visible:opacity-100"
							aria-hidden="true"
						/>
					</button>
					<Input
						v-else
						ref="nameInput"
						v-model="name"
						appearance="transparent"
						input-class="!font-bold"
						aria-label="Rule name"
						:disabled="isSaving"
						:error="Boolean(name && nameError)"
						:aria-describedby="name && nameError ? nameErrorId : undefined"
						placeholder="Enter a rule name"
						autocomplete="off"
						@blur="editingName = false"
						@keydown.enter.prevent="finishEditingName"
					/>
					<span v-if="name && nameError" :id="nameErrorId" class="sr-only">{{ nameError }}</span>
				</h1>
				<CopyCode :text="rule.id" title="" :aria-label="`Copy rule ID ${rule.id}`" />
				<time
					:datetime="rule.updated_at"
					:title="formatDate(rule.updated_at, true)"
					class="shrink-0 text-sm text-secondary"
				>
					Edited {{ formatDate(rule.updated_at) }}
				</time>
			</div>
			<div class="ml-auto flex shrink-0 flex-wrap items-center gap-2">
				<Button :disabled="!hasChanges || isSaving" @click="discardRule">Discard</Button>
				<span v-tooltip="saveDisabledReason" class="inline-flex rounded-xl">
					<Button
						type="colored"
						color="brand"
						aria-keyshortcuts="Meta+S Control+S"
						:loading="isSavingRule"
						:disabled="isSaving || Boolean(saveDisabledReason)"
						:class="{ 'pointer-events-none': saveDisabledReason }"
						@click="saveRule"
					>
						{{ isSavingRule ? 'Saving…' : 'Save changes' }}
					</Button>
				</span>
				<IconButton
					v-tooltip="'Close rule editor'"
					type="quiet"
					circular
					label="Close rule editor"
					:disabled="isSaving"
					@click="emit('close')"
				>
					<XIcon aria-hidden="true" />
				</IconButton>
			</div>
		</header>

		<div
			class="rule-workspace grid min-h-0 flex-1 lg:grid-rows-[minmax(0,1fr)]"
			:style="workspaceStyle"
		>
			<ProjectFlagInputSchema
				v-model:project-id="sampleProjectId"
				:saved-rule="rule"
				:questions="params.decision_questions"
				:patterns="patterns"
				class="min-w-0 border-0 border-b border-solid border-surface-4 p-3 md:border-b-0 lg:overflow-y-auto lg:pt-2 xl:px-4 xl:pb-4"
				@open-module="openModule"
				@new-module="newModule"
			/>
			<PanelDivider
				v-model="savedScopeWidth"
				class="hidden md:block"
				label="Resize rule reference width"
				side="left"
				width-property="--scope-width"
				responsive
				:min="192"
				:reserve="scopeResizeReserve"
			/>

			<section
				ref="editorPane"
				class="flex min-h-0 min-w-0 flex-col bg-surface-2"
				aria-label="Rhai editor"
			>
				<div class="relative flex min-h-0 flex-1 flex-col" @keydown.capture="onEditorKeydown">
					<div
						class="absolute right-2 top-2 z-10 flex items-center gap-2 rounded-lg bg-surface-2 px-2"
					>
						<a
							href="https://rhai.rs/book/"
							target="_blank"
							rel="noopener noreferrer"
							class="inline-flex items-center gap-1 text-sm text-link"
						>
							Rhai <ExternalIcon class="size-4" aria-hidden="true" />
						</a>
						<TeleportPopoutMenu label="Editor settings" type="quiet" size="xs" icon-only>
							<template #trigger><SettingsIcon /></template>
							<template #panel>
								<div class="flex items-center gap-6">
									<label :for="wordWrapId" class="text-sm font-medium">Word wrap</label>
									<Toggle :id="wordWrapId" v-model="wordWrap" small />
								</div>
							</template>
						</TeleportPopoutMenu>
					</div>
					<RhaiExpressionEditor
						:runtime="rhaiEditor"
						:id="expressionId"
						v-model="expression"
						class="h-96 lg:h-auto lg:min-h-24 lg:flex-1"
						:disabled="isSaving"
						:error-id="expressionError ? expressionErrorId : undefined"
						:word-wrap="wordWrap"
						variant="source"
						frameless
					/>
					<p v-if="expressionError" :id="expressionErrorId" class="m-0 px-3 py-1 text-sm text-red">
						{{ expressionError }}
					</p>
				</div>

				<HeightDivider
					v-model="testerHeight"
					class="hidden lg:block"
					label="Resize test projects height"
					height-property="--tester-drag-height"
					side="bottom"
					:min="160"
					:reserve="176"
				/>
				<ProjectFlagRuleTester
					ref="tester"
					class="lg:h-[var(--tester-drag-height,var(--tester-height,22rem))] lg:max-h-[calc(100%_-_192px)] lg:border-t-0"
					:style="{ '--tester-height': testerHeight === null ? undefined : `${testerHeight}px` }"
					:params="params"
					:initial-project-id="initialProjectId"
					:disabled="isSaving"
					:has-unsaved-modules="Boolean(moduleEditor?.isDirty)"
					:can-test="!expressionError && invalidPatternIndex === -1 && invalidQuestionIndex === -1"
				/>
			</section>

			<PanelDivider
				v-model="savedInputsWidth"
				class="hidden lg:block"
				label="Resize rule inputs width"
				side="right"
				width-property="--inputs-width"
				responsive
				:min="192"
				:reserve="inputsResizeReserve"
			/>
			<aside
				aria-label="Rule inputs"
				class="flex min-w-0 flex-col gap-4 border-0 border-t border-solid border-surface-4 p-3 md:col-span-3 lg:col-span-1 lg:overflow-y-auto lg:border-t-0 lg:pt-2 xl:px-4 xl:pb-4"
			>
				<FileScanPatternsEditor v-model="patterns" :errors="patternErrors" :disabled="isSaving" />
				<DecisionQuestionsEditor
					v-model="decisionQuestions"
					:errors="decisionQuestionErrors"
					:disabled="isSaving"
				/>
			</aside>
		</div>
		<ConfirmLeaveModal
			ref="leaveDialog"
			title="Leave rule editor?"
			header="You have unsaved changes"
			body="Unsaved changes in the rule editor will be lost."
			stay-label="Cancel"
			leave-label="Leave"
		/>
		<ConfirmModal
			ref="discardModal"
			title="Discard changes?"
			description="Discard unsaved changes to this rule?"
			:markdown="false"
			proceed-label="Discard"
			@proceed="applyDiscard"
		/>
		<SharedModuleEditorModal
			ref="moduleEditor"
			:runtime="rhaiEditor"
			:modules="modules"
			:word-wrap="wordWrap"
		/>
	</div>
</template>

<script setup lang="ts">
import type {
	ProjectFlagModule,
	ProjectFlagRule,
	ProjectFlagRuleParams,
} from '@modrinth/api-client'
import { ArrowLeftIcon, EditIcon, ExternalIcon, SettingsIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmLeaveModal,
	ConfirmModal,
	CopyCode,
	IconButton,
	Input,
	injectModrinthClient,
	TeleportPopoutMenu,
	Toggle,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useEventListener, useLocalStorage } from '@vueuse/core'
import { computed, nextTick, ref, shallowRef, useId, useTemplateRef } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'

import HeightDivider from '~/components/project-review/tech-review/height-divider.vue'
import PanelDivider from '~/components/project-review/tech-review/panel-divider.vue'
import { projectFlagModulesQueryOptions } from '~/composables/queries/project-flag'

import { formatDate } from '../../run-format'
import { useFlagErrorNotification } from '../../use-flag-error-notification'
import { getRuleNameError, useSaveRule } from '../use-rules'
import DecisionQuestionsEditor, {
	type DecisionQuestionEntry,
} from './decision-questions-editor.vue'
import RhaiExpressionEditor from './expression-editor.vue'
import FileScanPatternsEditor, { type FileScanPattern } from './file-scan-patterns-editor.vue'
import ProjectFlagInputSchema from './input-schema/index.vue'
import type { RhaiEditorRuntime } from './rhai-editor-runtime'
import SharedModuleEditorModal from './shared-module-editor-modal.vue'
import ProjectFlagRuleTester from './tester.vue'

const props = defineProps<{
	initialRule: ProjectFlagRule
	initialProjectId?: string
	backLabel: string
	rhaiEditor: RhaiEditorRuntime
}>()
const emit = defineEmits<{ close: []; saved: [rule: ProjectFlagRule] }>()

const id = useId()
const nameErrorId = `${id}-name-error`
const expressionId = `${id}-expression`
const expressionErrorId = `${id}-expression-error`
const wordWrapId = `${id}-word-wrap`
const wordWrap = useLocalStorage('moderation-project-flag-rule-word-wrap', true)
const savedInputsWidth = useLocalStorage('moderation-flag-rule-inputs-width', 0)
const savedScopeWidth = useLocalStorage('moderation-flag-rule-scope-width', 0)
const workspaceStyle = computed(() => ({
	'--inputs-width': savedInputsWidth.value ? `${savedInputsWidth.value}px` : undefined,
	'--scope-width': savedScopeWidth.value ? `${savedScopeWidth.value}px` : undefined,
}))
const testerHeight = ref<number | null>(null)

function scopeResizeReserve(workspace: HTMLElement) {
	const inputsWidth = (workspace.lastElementChild as HTMLElement).offsetWidth
	return 360 + (inputsWidth < workspace.clientWidth - 1 ? inputsWidth + 8 : 0)
}

function inputsResizeReserve(workspace: HTMLElement) {
	return 360 + (workspace.firstElementChild as HTMLElement).offsetWidth + 8
}

const editorPane = useTemplateRef<HTMLElement>('editorPane')
const tester = useTemplateRef<InstanceType<typeof ProjectFlagRuleTester>>('tester')
const rule = shallowRef(props.initialRule)
const name = ref('')
const editingName = ref(false)
const nameInput = useTemplateRef<InstanceType<typeof Input>>('nameInput')
const nameButton = useTemplateRef<HTMLButtonElement>('nameButton')
const leaveDialog = useTemplateRef<InstanceType<typeof ConfirmLeaveModal>>('leaveDialog')
const discardModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('discardModal')
const moduleEditor = useTemplateRef<InstanceType<typeof SharedModuleEditorModal>>('moduleEditor')

async function editName() {
	editingName.value = true
	await nextTick()
	nameInput.value?.focus()
}

async function finishEditingName() {
	editingName.value = false
	await nextTick()
	nameButton.value?.focus()
}
const sampleProjectId = ref(props.initialProjectId)
const expression = ref('')
const patterns = ref<FileScanPattern[]>([])
const decisionQuestions = ref<DecisionQuestionEntry[]>([])

function reconcileDraftRows<T extends object>(
	draftRows: T[],
	savedRows: T[],
	getKey: (row: T) => string,
) {
	const remaining = new Map(savedRows.map((row) => [getKey(row), row]))
	const rows: T[] = []
	for (const draft of draftRows) {
		const key = getKey(draft).trim()
		const saved = remaining.get(key)
		if (!saved) continue
		rows.push(Object.assign(draft, saved))
		remaining.delete(key)
	}
	return rows.concat([...remaining.values()])
}

function resetRule(preserveRows = false) {
	const saved = rule.value
	name.value = saved.name
	expression.value = saved.params.expression
	const savedPatterns = Object.entries(saved.params.scan_files_for ?? {}).map(
		([name, pattern]) => ({
			name,
			pattern,
		}),
	)
	const savedQuestions = Object.entries(saved.params.decision_questions ?? {}).map(
		([key, { question, include_context }]) => ({
			key,
			question,
			include_context: [...include_context],
		}),
	)
	patterns.value = preserveRows
		? reconcileDraftRows(patterns.value, savedPatterns, (row) => row.name)
		: savedPatterns
	decisionQuestions.value = preserveRows
		? reconcileDraftRows(decisionQuestions.value, savedQuestions, (row) => row.key)
		: savedQuestions
}

resetRule()

const nameError = computed(() => getRuleNameError(name.value))
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
const decisionQuestionErrors = computed(() => {
	const keys = decisionQuestions.value.map(({ key }) => key.trim())
	return decisionQuestions.value.map((entry, index) => {
		const key = keys[index]
		if (!key) return 'Enter a decision question key.'
		if (keys.indexOf(key) !== keys.lastIndexOf(key))
			return 'Each decision question key must be unique.'
		if (!entry.question.trim()) return 'Enter a question.'
		if (entry.include_context.length === 0) return 'Select at least one context field.'
		return ''
	})
})
const invalidQuestionIndex = computed(() => decisionQuestionErrors.value.findIndex(Boolean))
const saveDisabledReason = computed(() => {
	if (nameError.value) return nameError.value
	if (expressionError.value) return expressionError.value
	const index = invalidPatternIndex.value
	if (index !== -1) {
		const error = patternErrors.value[index]
		return `Pattern ${index + 1}: ${error.name || error.pattern}`
	}
	if (invalidQuestionIndex.value !== -1) {
		return `Question ${invalidQuestionIndex.value + 1}: ${decisionQuestionErrors.value[invalidQuestionIndex.value]}`
	}
	return hasChanges.value ? undefined : 'Make a change to save this rule.'
})
const params = computed<ProjectFlagRuleParams>(() => ({
	expression: expression.value,
	decision_questions: Object.fromEntries(
		decisionQuestions.value.map(({ key, question, include_context }) => [
			key.trim(),
			{ question, include_context: [...include_context] },
		]),
	),
	scan_files_for: Object.fromEntries(
		patterns.value.map((entry) => [entry.name.trim(), entry.pattern]),
	),
}))
const hasChanges = computed(() => {
	const original = rule.value
	const savedPatterns = original.params.scan_files_for ?? {}
	const savedQuestions = original.params.decision_questions ?? {}
	return (
		name.value !== original.name ||
		decisionQuestions.value.length !== Object.keys(savedQuestions).length ||
		new Set(decisionQuestions.value.map(({ key }) => key)).size !==
			decisionQuestions.value.length ||
		decisionQuestions.value.some(({ key, question, include_context }) => {
			const saved = Object.hasOwn(savedQuestions, key) ? savedQuestions[key] : undefined
			return (
				!saved ||
				question !== saved.question ||
				include_context.length !== saved.include_context.length ||
				include_context.some((field) => !saved.include_context.includes(field))
			)
		}) ||
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
	mutateAsync: save,
	isPending: isSavingRule,
	error: saveError,
	reset: resetSave,
} = useSaveRule()
useFlagErrorNotification(saveError, 'Could not save the rule')

function discardRule() {
	if (hasChanges.value && !isSaving.value) discardModal.value?.show()
}

function applyDiscard() {
	if (isSaving.value) return
	resetSave()
	resetRule()
}

async function saveRule() {
	if (saveDisabledReason.value || isSaving.value) return
	try {
		const saved = await save({
			id: rule.value.id,
			body: { name: name.value, params: params.value },
		})
		rule.value = saved
		resetRule(true)
		emit('saved', saved)
	} catch {
		// The save error is reported by the mutation notification.
	}
}

function onEditorKeydown(event: KeyboardEvent) {
	if (event.key !== 'Enter' || !(event.metaKey || event.ctrlKey) || event.isComposing) return
	event.preventDefault()
	event.stopPropagation()
	if (!event.repeat) tester.value?.evaluate()
}

const client = injectModrinthClient()
const { data: modules } = useQuery(projectFlagModulesQueryOptions(client))
const isSaving = computed(() => isSavingRule.value || Boolean(moduleEditor.value?.isBusy))

function openModule(module: ProjectFlagModule) {
	if (!isSaving.value) moduleEditor.value?.show(module)
}

function newModule() {
	if (!isSaving.value) moduleEditor.value?.show()
}

const isDirty = computed(() => hasChanges.value || Boolean(moduleEditor.value?.isDirty))

useEventListener(
	computed(() => (isDirty.value ? editorPane.value?.ownerDocument.defaultView : null)),
	'beforeunload',
	(event) => event.preventDefault(),
)

function allowEditorNavigation() {
	if (isSaving.value) return false
	return !isDirty.value || (leaveDialog.value?.prompt() ?? false)
}

onBeforeRouteLeave(allowEditorNavigation)
onBeforeRouteUpdate((to, from) => {
	if (
		to.path === from.path &&
		to.query.rule === from.query.rule &&
		(to.query.ruleProject ?? to.query.project) === (from.query.ruleProject ?? from.query.project)
	) {
		return
	}
	return allowEditorNavigation()
})

defineExpose({ saveRule })
</script>

<style scoped>
@media (min-width: 768px) {
	.rule-workspace {
		--editor-min-width: 360px;
		--handle-width: 8px;
		grid-template-columns:
			min(
				var(--scope-width-drag, var(--scope-width, 33.333%)),
				calc(100% - var(--editor-min-width) - var(--handle-width))
			)
			var(--handle-width) minmax(0, 1fr);
	}
}

@media (min-width: 1024px) {
	.rule-workspace {
		--side-space: calc(100% - var(--editor-min-width) - 2 * var(--handle-width));
		--scope-track: min(
			var(--scope-width-drag, var(--scope-width, 23%)),
			calc(var(--side-space) - 192px)
		);
		grid-template-columns:
			var(--scope-track) var(--handle-width) minmax(0, 1fr) var(--handle-width)
			min(
				var(--inputs-width-drag, var(--inputs-width, 25%)),
				calc(var(--side-space) - var(--scope-track))
			);
	}
}
</style>
