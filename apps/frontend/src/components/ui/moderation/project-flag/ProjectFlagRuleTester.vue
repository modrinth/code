<template>
	<div class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center gap-3">
			<label for="project-flag-test-project" class="font-semibold text-contrast"
				>Test on project:</label
			>
			<Input
				id="project-flag-test-project"
				v-model="projectId"
				:disabled="disabled"
				placeholder="Paste a project ID"
				autocomplete="off"
				:spellcheck="false"
				@keydown.enter.prevent="evaluate"
			/>
		</div>
		<div class="flex flex-col gap-3">

			<p v-if="normalizedProjectId && !validProjectId" role="alert" class="m-0 text-sm text-red">
				Enter a project ID containing only letters and numbers, not a project URL or slug.
			</p>
			<div v-if="isPending" role="status" class="flex items-center gap-2 text-secondary">
				<LoaderCircleIcon class="size-5 animate-spin" />
				Testing rule…
			</div>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<p class="m-0 text-sm text-red">Could not test the rule: {{ error?.message }}</p>
				<Button :disabled="!canEvaluate" @click="evaluate">Try again</Button>
			</div>
			<div v-else-if="isSuccess" class="grid grid-cols-1 items-start gap-4 md:grid-cols-2">
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
						<div class="max-h-64 overflow-auto whitespace-pre-wrap break-words font-mono text-sm">{{ JSON.stringify(flag, null, 2) }}</div>
					</article>
				</section>
				<section class="flex min-w-0 flex-col gap-2" aria-label="Rule print output">
					<h3 class="m-0 text-base font-semibold text-contrast">Log</h3>
					<pre
						v-if="result?.log.length"
						class="m-0 max-h-64 overflow-auto whitespace-pre-wrap break-words rounded-lg bg-surface-2 p-3 text-sm"
					><code>{{ result.log.join('\n') }}</code></pre>
					<p v-else class="m-0 text-sm text-secondary">No print output.</p>
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

const props = defineProps<{ expression: string; disabled?: boolean }>()
const client = injectModrinthClient()
const projectId = ref<string | number | undefined>('')

const normalizedProjectId = computed(() => String(projectId.value ?? '').trim())
const validProjectId = computed(() => /^[a-zA-Z0-9]+$/.test(normalizedProjectId.value))
const canEvaluate = computed(
	() =>
		!props.disabled &&
		validProjectId.value &&
		props.expression.length > 0 &&
		Array.from(props.expression).length <= 65535,
)

const {
	mutate,
	data: result,
	isPending,
	isError,
	error,
	isSuccess,
	reset,
} = useMutation({
	mutationFn: (body: { project_id: string; expression: string }) =>
		client.labrinth.project_flags_internal.evalRule(body),
	retry: false,
})

let disposed = false
onScopeDispose(() => {
	disposed = true
})

function evaluate() {
	if (disposed || !canEvaluate.value) return
	mutate({ project_id: normalizedProjectId.value, expression: props.expression })
}

const queueEvaluation = useDebounceFn(evaluate, 500)
watch([normalizedProjectId, () => props.expression, () => props.disabled], () => {
	reset()
	queueEvaluation()
})
</script>
