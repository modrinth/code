<template>
	<section class="relative min-w-0 shrink-0 bg-surface-2" aria-label="Test rule">
		<div
			v-if="expanded"
			role="separator"
			tabindex="0"
			aria-label="Resize test output"
			aria-orientation="horizontal"
			:aria-controls="outputId"
			:aria-valuemin="MIN_HEIGHT"
			:aria-valuemax="MAX_HEIGHT"
			:aria-valuenow="outputHeight"
			class="output-divider absolute -top-2 left-0 z-20 h-4 w-full cursor-row-resize touch-none select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-brand"
			:class="{ 'is-dragging': drag }"
			@pointerdown="startResize"
			@pointermove="resize"
			@pointerup="stopResize"
			@pointercancel="stopResize"
			@lostpointercapture="drag = undefined"
			@keydown.up.prevent="adjustHeight(24)"
			@keydown.down.prevent="adjustHeight(-24)"
			@keydown.home.prevent="outputHeight = MIN_HEIGHT"
			@keydown.end.prevent="outputHeight = MAX_HEIGHT"
		/>
		<div
			class="relative isolate flex flex-wrap items-center gap-2 py-3 pl-3 pr-11"
			:class="{ 'border-0 border-t border-solid border-surface-4': !expanded }"
		>
			<button
				type="button"
				class="absolute inset-0 flex w-full cursor-pointer items-center justify-end border-0 bg-transparent px-3 text-secondary hover:text-contrast focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-inset focus-visible:ring-brand-shadow"
				:aria-label="expanded ? 'Collapse test output' : 'Expand test output'"
				:aria-expanded="expanded"
				:aria-controls="outputId"
				@click="expanded = !expanded"
			>
				<ChevronDownIcon v-if="expanded" class="size-4" aria-hidden="true" />
				<ChevronUpIcon v-else class="size-4" aria-hidden="true" />
			</button>
			<ProjectCombobox
				v-if="!batchMode"
				v-model="projectId"
				trigger-size="sm"
				class="relative z-10 min-w-0 max-w-80 flex-1"
				dropdown-width="26rem"
				search-placeholder="Search project name/ID..."
				:search-input-attrs="{ 'aria-label': 'Project to test' }"
				lookup-project-ids
				:show-chevron="false"
				clearable
			>
				<template #option-suffix="{ item }">
					<TagItem class="shrink-0 font-mono">{{ item.value }}</TagItem>
				</template>
			</ProjectCombobox>
			<Input
				v-else
				v-model="batchInput"
				class="relative z-10 min-w-0 flex-1"
				placeholder="Project IDs separated by commas or spaces"
				aria-label="Up to 100 unique project IDs"
				autocomplete="off"
				:spellcheck="false"
				@keydown.enter.prevent="evaluate"
			/>
			<Button
				class="relative z-10"
				size="xs"
				:aria-pressed="batchMode"
				@click="batchMode = !batchMode"
			>
				Batch IDs
			</Button>
			<Button
				class="relative z-10"
				size="xs"
				v-tooltip="runDisabledReason ?? 'Run test (Cmd/Ctrl+Enter)'"
				type="colored"
				color="brand"
				:disabled="!canEvaluate"
				:loading="isPending"
				@click="evaluate"
			>
				<LoaderCircleIcon v-if="isPending" class="animate-spin" />
				<PlayIcon v-else />
				Run
			</Button>
		</div>
		<div
			v-show="expanded"
			ref="output"
			:id="outputId"
			class="flex max-h-[40dvh] min-h-24 flex-col"
			:style="{ height: `${outputHeight}px` }"
			:aria-busy="isPending"
		>
			<div class="flex flex-wrap items-center gap-3 px-3 pb-3">
				<label v-if="results.length > 1" class="flex items-center gap-2 text-sm">
					Project result
					<select v-model="resultIndex" class="rounded-md bg-surface-3 px-2 py-1">
						<option v-for="(entry, index) in results" :key="entry.project_id" :value="index">
							{{ entry.project_id }}
						</option>
					</select>
				</label>
				<Tabs
					v-model:value="activeView"
					:tabs="outputTabs"
					size="default"
					color="gray"
					aria-label="Test output"
				/>
			</div>
			<div class="min-h-0 flex-1 overflow-auto px-3 pb-3">
				<template v-if="result">
					<p v-if="result.error" role="alert" class="m-0 mb-3 whitespace-pre-wrap text-sm text-red">
						{{ result.error }}
					</p>
					<section
						v-if="activeView === 'flags'"
						class="flex flex-col gap-3"
						aria-label="Flags found"
					>
						<p v-if="result.flags.length === 0" class="m-0 text-sm text-secondary">
							No flags found.
						</p>
						<article
							v-for="(flag, index) in result.flags"
							:key="index"
							class="flex min-w-0 flex-col gap-2 rounded-xl border border-solid border-surface-4 p-3"
						>
							<span class="break-words font-mono text-sm font-semibold text-contrast">{{
								flag.key
							}}</span>
							<p v-if="flag.reason" class="m-0 whitespace-pre-wrap break-words text-sm">
								{{ flag.reason }}
							</p>
							<details class="text-xs">
								<summary class="w-fit cursor-pointer text-secondary">Raw JSON</summary>
								<pre
									class="mb-0 whitespace-pre-wrap break-words font-mono"
									v-text="JSON.stringify(flag, null, 2)"
								/>
							</details>
						</article>
					</section>
					<section v-else aria-label="Rule print output">
						<pre
							v-if="result.log.length"
							class="m-0 whitespace-pre-wrap break-words font-mono text-sm"
							v-text="result.log.join('\n')"
						/>
						<p v-else class="m-0 text-sm text-secondary">No print output.</p>
					</section>
				</template>
				<p v-else-if="!isPending && !error" class="m-0 text-sm text-secondary">
					Choose a project and run the rule to see flags and print output.
				</p>
			</div>
		</div>
	</section>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { EvalProjectFlagRuleRequest, ProjectFlagRuleParams } from '@modrinth/api-client'
import { ChevronDownIcon, ChevronUpIcon, LoaderCircleIcon, PlayIcon } from '@modrinth/assets'
import {
	Button,
	Input,
	injectModrinthClient,
	ProjectCombobox,
	Tabs,
	type TabsValue,
	TagItem,
} from '@modrinth/ui'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, shallowRef, useId, useTemplateRef } from 'vue'

const props = defineProps<{
	params: ProjectFlagRuleParams
	canTest: boolean
}>()
const client = injectModrinthClient()
const outputId = useId()
const projectId = ref<string>()
const batchMode = ref(false)
const batchInput = ref('')
const projectIds = computed(() =>
	batchMode.value
		? [
				...new Set(
					String(batchInput.value ?? '')
						.split(/[,\s]+/)
						.filter(Boolean),
				),
			]
		: projectId.value
			? [projectId.value]
			: [],
)
const resultIndex = ref(0)
const expanded = ref(false)
const activeView = ref<TabsValue>('flags')
const output = useTemplateRef<HTMLElement>('output')
const MIN_HEIGHT = 96
const MAX_HEIGHT = 480
const outputHeight = ref(240)
const drag = shallowRef<{ pointerId: number; startY: number; startHeight: number }>()

function runTest(request: EvalProjectFlagRuleRequest) {
	return client.labrinth.project_flags_internal.evalRule(request)
}

const previousResult = shallowRef<Awaited<ReturnType<typeof runTest>>>()
const { mutate, data, isPending, error } = useMutation({
	mutationFn: runTest,
	retry: false,
	onSuccess: () => {
		resultIndex.value = 0
	},
})
const results = computed(() => data.value ?? previousResult.value ?? [])
const result = computed(() => results.value[resultIndex.value])
const runDisabledReason = computed(() => {
	if (isPending.value) return 'A test is running.'
	if (!projectIds.value.length)
		return batchMode.value ? 'Enter at least one project ID.' : 'Choose a project to test.'
	if (projectIds.value.length > 100) return 'Enter up to 100 unique project IDs.'
	if (projectIds.value.some((id) => !/^[A-Za-z0-9]+$/.test(id))) return 'Enter base62 project IDs.'
	if (!props.canTest) return 'Finish editing the rule before testing.'
	return undefined
})
const canEvaluate = computed(() => !runDisabledReason.value)
const outputTabs = computed(() => [
	{ value: 'flags', label: result.value ? `Flags (${result.value.flags.length})` : 'Flags' },
	{ value: 'log', label: result.value ? `Log (${result.value.log.length})` : 'Log' },
])

function evaluate() {
	if (!canEvaluate.value) return
	expanded.value = true
	previousResult.value = data.value ?? previousResult.value
	mutate({
		project_ids: projectIds.value,
		params: props.params,
	})
}

function adjustHeight(delta: number) {
	outputHeight.value = Math.max(MIN_HEIGHT, Math.min(MAX_HEIGHT, outputHeight.value + delta))
}

function startResize(event: PointerEvent) {
	if (event.button !== 0) return
	event.preventDefault()
	const handle = event.currentTarget as HTMLElement
	handle.setPointerCapture(event.pointerId)
	drag.value = {
		pointerId: event.pointerId,
		startY: event.clientY,
		startHeight: output.value?.clientHeight ?? outputHeight.value,
	}
}

function resize(event: PointerEvent) {
	if (drag.value?.pointerId !== event.pointerId) return
	outputHeight.value = Math.max(
		MIN_HEIGHT,
		Math.min(MAX_HEIGHT, drag.value.startHeight + drag.value.startY - event.clientY),
	)
}

function stopResize(event: PointerEvent) {
	if (drag.value?.pointerId !== event.pointerId) return
	drag.value = undefined
	const handle = event.currentTarget as HTMLElement
	handle.releasePointerCapture(event.pointerId)
}

useFlagErrorNotification(error, 'Could not test the rule')

defineExpose({ evaluate })
</script>

<style scoped>
.output-divider::after {
	content: '';
	position: absolute;
	top: 50%;
	left: 0;
	width: 100%;
	height: 1px;
	background: var(--surface-3);
}

.output-divider:hover::after {
	background: var(--color-brand);
	transition: background-color 0s 0.5s;
}

.output-divider:focus-visible::after,
.output-divider.is-dragging::after {
	background: var(--color-brand);
	transition: none;
}
</style>
