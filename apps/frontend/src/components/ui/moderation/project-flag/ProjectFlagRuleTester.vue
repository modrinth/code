<template>
	<div class="flex flex-col gap-3">
		<FileScanPatternsEditor v-model="patterns" :disabled="disabled" :errors="patternErrors" />
		<DecisionQuestionsEditor
			v-model="decisionQuestions"
			:disabled="disabled"
			:errors="decisionQuestionsErrors"
		/>
		<div class="flex flex-wrap items-center gap-3">
			<label for="project-flag-test-project" class="font-semibold text-contrast"
				>Test on projects:</label
			>
			<Input
				id="project-flag-test-project"
				v-model="projectIdsInput"
				:disabled="disabled"
				placeholder="Paste project IDs separated by commas or whitespace"
				autocomplete="off"
				:spellcheck="false"
				@keydown.enter.prevent="evaluate"
			/>
		</div>
		<div class="flex flex-col gap-3">
			<p class="m-0 text-sm text-secondary">
				Enter 1–100 unique base62 project IDs (letters and numbers), separated by commas or
				whitespace. Slugs and URLs are not supported.
			</p>
			<p v-if="projectIdsError" role="alert" class="m-0 text-sm text-red">
				{{ projectIdsError }}
			</p>
			<div v-if="isPending" role="status" class="flex items-center gap-2 text-secondary">
				<LoaderCircleIcon class="size-5 animate-spin" />
				Testing rule…
			</div>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<p class="m-0 text-sm text-red">Could not test the rule: {{ error?.message }}</p>
				<Button :disabled="!canEvaluate" @click="evaluate">Try again</Button>
			</div>
			<div v-else-if="isSuccess" class="flex flex-col gap-4">
				<section
					v-for="result in results"
					:key="result.project_id"
					class="flex min-w-0 flex-col gap-3"
					:aria-label="`Results for project ${result.project_id}`"
				>
					<h3 class="m-0 text-base font-semibold text-contrast">Project {{ result.project_id }}</h3>
					<p v-if="result.error !== null" role="alert" class="m-0 text-sm text-red">
						Could not test this project: {{ result.error }}
					</p>
					<div v-else class="grid grid-cols-1 items-start gap-4 md:grid-cols-2">
						<section class="flex min-w-0 flex-col gap-3" aria-label="Flags found">
							<p class="m-0 font-semibold text-contrast" role="status">
								{{
									result?.flags.length === 0
										? 'No flags found.'
										: `${result?.flags.length} flag${result?.flags.length === 1 ? '' : 's'} found`
								}}
							</p>
							<article
								v-for="(flag, index) in result?.flags"
								:key="index"
								class="flex min-w-0 flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-4"
							>
								<div
									class="max-h-64 overflow-auto whitespace-pre-wrap break-words font-mono text-sm"
									v-text="JSON.stringify(flag, null, 2)"
								/>
							</article>
						</section>
						<section class="flex min-w-0 flex-col gap-2" aria-label="Rule print output">
							<h3 class="m-0 text-base font-semibold text-contrast">Log</h3>
							<div
								v-if="result?.log.length"
								class="max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg bg-surface-2 p-3 font-mono text-sm"
								v-text="result.log.join('\n')"
							/>
							<p v-else class="m-0 text-sm text-secondary">No print output.</p>
						</section>
					</div>
				</section>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, Input } from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, onScopeDispose, ref, watch } from 'vue'

import DecisionQuestionsEditor, { type DecisionQuestionEntry } from './DecisionQuestionsEditor.vue'
import FileScanPatternsEditor, { type FileScanPattern } from './FileScanPatternsEditor.vue'

const props = defineProps<{
	expression: string
	disabled?: boolean
	patternErrors: string[]
	decisionQuestionsErrors: string[]
}>()
const client = injectModrinthClient()
const projectIdsInput = ref<string | number | undefined>('')
const patterns = defineModel<FileScanPattern[]>('patterns', { required: true })
const decisionQuestions = defineModel<DecisionQuestionEntry[]>('decisionQuestions', {
	required: true,
})
const decisionQuestionsMap = computed(() =>
	Object.fromEntries(
		decisionQuestions.value.map((entry) => [
			entry.key.trim(),
			{ question: entry.question, include_context: [...entry.include_context] },
		]),
	),
)
const scanFilesFor = computed(() =>
	Object.fromEntries(patterns.value.map((entry) => [entry.name.trim(), entry.pattern])),
)

const projectIds = computed(() => [
	...new Set(
		String(projectIdsInput.value ?? '')
			.split(/[,\s]+/)
			.filter(Boolean),
	),
])
const projectIdsError = computed(() => {
	if (projectIds.value.length === 0)
		return 'Enter at least 1 project ID; up to 100 unique project IDs are allowed.'
	if (projectIds.value.length > 100) return 'Enter no more than 100 unique project IDs.'
	if (projectIds.value.some((id) => !/^[a-zA-Z0-9]+$/.test(id))) {
		return 'Project IDs must contain only base62 letters and numbers. Do not enter slugs or URLs.'
	}
	return ''
})
const canEvaluate = computed(
	() =>
		!props.disabled &&
		props.patternErrors.every((error) => !error) &&
		props.decisionQuestionsErrors.every((error) => !error) &&
		!projectIdsError.value &&
		props.expression.length > 0 &&
		Array.from(props.expression).length <= 65535,
)

const {
	mutate,
	data: results,
	isPending,
	isError,
	error,
	isSuccess,
	reset,
} = useMutation({
	mutationFn: (body: Parameters<typeof client.labrinth.project_flags_internal.evalRule>[0]) =>
		client.labrinth.project_flags_internal.evalRule(body),
	retry: false,
})

let disposed = false
onScopeDispose(() => {
	disposed = true
})

function evaluate() {
	if (disposed || !canEvaluate.value) return
	mutate({
		project_ids: projectIds.value,
		params: {
			expression: props.expression,
			scan_files_for: scanFilesFor.value,
			decision_questions: decisionQuestionsMap.value,
		},
	})
}

const queueEvaluation = useDebounceFn(evaluate, 500)
watch(
	[projectIds, () => props.expression, () => props.disabled, scanFilesFor, decisionQuestions],
	() => {
		reset()
		queueEvaluation()
	},
	{ deep: true },
)
</script>
