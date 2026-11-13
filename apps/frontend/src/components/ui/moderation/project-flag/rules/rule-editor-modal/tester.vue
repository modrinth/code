<template>
	<div class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center gap-3">
			<label :for="projectIdInput" class="font-semibold text-contrast">Project</label>
			<ProjectCombobox
				v-model="projectId"
				class="max-w-80"
				dropdown-width="26rem"
				search-placeholder="Search project name/ID..."
				:search-input-attrs="{ id: projectIdInput }"
				lookup-project-ids
				:show-chevron="false"
				clearable
			>
				<template #option-suffix="{ item }">
					<TagItem class="shrink-0 font-mono">{{ item.value }}</TagItem>
				</template>
			</ProjectCombobox>
		</div>
		<div class="flex flex-col gap-3">
			<div v-if="isPending" role="status" class="flex items-center gap-2 text-secondary">
				<LoaderCircleIcon class="size-5 animate-spin" />
				Testing rule…
			</div>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<Button :disabled="!canEvaluate" @click="evaluate">Try again</Button>
			</div>
			<div v-else-if="isSuccess" class="grid grid-cols-1 items-start gap-4 md:grid-cols-2">
				<section class="flex min-w-0 flex-col gap-3" aria-label="Flags found">
					<p
						v-tooltip="testFlagTooltip"
						class="m-0 w-fit font-semibold text-contrast"
						role="status"
					>
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
		</div>
	</div>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { ProjectFlagRuleParams } from '@modrinth/api-client'
import { LoaderCircleIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, ProjectCombobox, TagItem } from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, onScopeDispose, ref, useId, watch } from 'vue'

import { flagCountTooltip } from '../../run-format'

const props = defineProps<{
	params: ProjectFlagRuleParams
	canTest: boolean
}>()
const client = injectModrinthClient()
const projectIdInput = useId()
const projectId = ref<string>()
const canEvaluate = computed(() => props.canTest && Boolean(projectId.value))

const {
	mutate,
	data: result,
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

const testFlagTooltip = computed(() => {
	const flags = result.value?.flags
	if (!flags) return
	return flagCountTooltip(
		flags.length,
		new Set(flags.map((flag) => flag.key)).size,
		flags.length === 1 ? 'flag found' : 'flags found',
	)
})

let disposed = false
onScopeDispose(() => {
	disposed = true
})

function evaluate() {
	if (disposed || !canEvaluate.value || !projectId.value) return
	mutate({
		project_id: projectId.value,
		params: props.params,
	})
}

const queueEvaluation = useDebounceFn(evaluate, 500)
watch([projectId, () => props.params, canEvaluate], () => {
	reset()
	queueEvaluation()
})
useFlagErrorNotification(error, 'Could not test the rule')
</script>
