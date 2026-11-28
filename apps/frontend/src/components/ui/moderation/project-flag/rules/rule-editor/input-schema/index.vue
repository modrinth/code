<template>
	<aside class="flex flex-col gap-4 overflow-x-clip" aria-label="Rule reference">
		<Accordion
			role="region"
			aria-label="Rule context"
			open-by-default
			:button-class="sectionButtonClass"
			content-class="flex min-w-0 flex-col gap-2 pt-1"
		>
			<template #title>Rule context</template>
			<SampleProjectPicker v-model="projectId" />
			<Input
				v-model="fieldQuery"
				:icon="ListFilterIcon"
				size="small"
				aria-label="Filter rule context fields"
				placeholder="Filter fields…"
				autocomplete="off"
				clearable
			/>
			<p v-if="isPending" role="status" class="m-0 flex items-center gap-2 text-sm text-secondary">
				<LoaderCircleIcon class="size-4 animate-spin" />
				Loading fields…
			</p>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<p class="m-0 text-sm text-red">Could not load fields.</p>
				<Button size="sm" :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<p
				v-else-if="fieldFilter && !visibleFields.length"
				role="status"
				class="m-0 text-sm text-secondary"
			>
				No matching fields.
			</p>
			<div
				v-else-if="data"
				role="region"
				aria-label="Sample JSON"
				tabindex="0"
				class="min-w-0 overflow-x-auto whitespace-nowrap rounded-xl border border-solid border-surface-4 bg-surface-1 p-3 font-mono text-xs leading-relaxed text-primary"
			>
				<code>{{ '{' }}</code>
				<div class="min-w-max pl-3">
					<SchemaField
						v-for="([name, value], index) in visibleFields"
						:key="name"
						:name="name"
						:value="value"
						:matching-branches="fieldFilter?.branches"
						:trailing-comma="index < visibleFields.length - 1"
					/>
				</div>
				<code>{{ '}' }}</code>
			</div>
		</Accordion>

		<div class="relative">
			<Accordion
				role="region"
				aria-label="Modules"
				open-by-default
				:button-class="sectionButtonClass"
				content-class="flex min-w-0 flex-col gap-1 pt-1"
			>
				<template #title>Modules</template>
				<p v-if="modulesPending" role="status" class="m-0 text-sm text-secondary">
					Loading modules…
				</p>
				<div v-else-if="modulesError" role="alert" class="flex flex-col items-start gap-2">
					<p class="m-0 text-sm text-red">Could not load modules.</p>
					<Button size="sm" @click="refetchModules()">Try again</Button>
				</div>
				<ul v-else class="m-0 flex list-none flex-col gap-1 p-0">
					<li v-for="module in modules" :key="module.name">
						<button
							type="button"
							class="custom-focus-indicator cursor-pointer break-all border-0 bg-transparent p-0 text-left font-mono text-sm text-link hover:underline focus-visible:underline focus-visible:decoration-brand focus-visible:decoration-2 focus-visible:underline-offset-2"
							@click="emit('openModule', module)"
						>
							{{ module.name }}
						</button>
					</li>
				</ul>
				<p
					v-if="!modulesPending && !modulesError && !modules?.length"
					class="m-0 text-sm text-secondary"
				>
					No modules saved.
				</p>
			</Accordion>
			<IconButton
				v-tooltip="'New module'"
				class="!absolute right-6 top-[5px]"
				type="quiet"
				size="xs"
				label="New module"
				@click="emit('newModule')"
			>
				<PackagePlusIcon aria-hidden="true" />
			</IconButton>
		</div>

		<Accordion
			role="region"
			aria-label="Packages"
			open-by-default
			:button-class="sectionButtonClass"
			content-class="pt-1"
		>
			<template #title>Packages</template>
			<ul class="m-0 flex list-none flex-col gap-1 p-0">
				<li v-for="pkg in packages" :key="pkg.name" class="font-mono text-sm">
					<a
						v-tooltip="pkg.description"
						:href="pkg.docs"
						target="_blank"
						rel="noopener noreferrer"
						:aria-label="`${pkg.name} documentation`"
						class="inline-flex items-center gap-1 text-link"
					>
						{{ pkg.name }} <ExternalIcon class="size-3 shrink-0" aria-hidden="true" />
					</a>
				</li>
			</ul>
		</Accordion>
	</aside>
</template>

<script setup lang="ts">
import type {
	ProjectFlagModule,
	ProjectFlagRule,
	ProjectFlagRuleParams,
	ProjectFlagValue,
} from '@modrinth/api-client'
import { ExternalIcon, ListFilterIcon, LoaderCircleIcon, PackagePlusIcon } from '@modrinth/assets'
import { Accordion, Button, IconButton, Input, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	projectFlagInputSchemaQueryOptions,
	projectFlagModulesQueryOptions,
} from '~/composables/queries/project-flag'

import type { FileScanPattern } from '../file-scan-patterns-editor.vue'
import SchemaField from './field.vue'
import { createFieldSearch, recordValue } from './fields'
import SampleProjectPicker from './sample-project-picker.vue'

const props = defineProps<{
	savedRule?: ProjectFlagRule
	questions: ProjectFlagRuleParams['decision_questions']
	patterns: FileScanPattern[]
}>()
const emit = defineEmits<{ openModule: [module: ProjectFlagModule]; newModule: [] }>()
const projectId = defineModel<string | undefined>('projectId', { required: true })
const client = injectModrinthClient()
const { data, isPending, isError, isFetching, refetch } = useQuery(
	computed(() => projectFlagInputSchemaQueryOptions(client, projectId.value)),
)
const {
	data: modules,
	isPending: modulesPending,
	isError: modulesError,
	refetch: refetchModules,
} = useQuery(projectFlagModulesQueryOptions(client))

const sectionButtonClass =
	'flex min-h-[38px] w-full cursor-pointer items-center rounded border-0 bg-transparent px-0 py-1 text-left text-sm font-semibold transition-[filter] hover:brightness-125 focus-visible:brightness-125 motion-reduce:transition-none [&>div]:text-primary [&_svg]:text-secondary [&_svg]:size-4'
const packages = [
	{
		name: 'StandardPackage',
		description: 'Built-in functions for strings, arrays, maps, and math.',
		docs: 'https://rhai.rs/book/language/builtin.html',
	},
	{
		name: 'rhai-chrono',
		description: 'Datetime parsing, formatting, timezones, and durations.',
		docs: 'https://docs.rs/crate/rhai-chrono/0.1.1',
	},
	{
		name: 'rhai-url',
		description: 'URL parsing, properties, and query parameters.',
		docs: 'https://docs.rs/rhai-url/0.0.6/rhai_url/',
	},
]
const fields = computed<[string, ProjectFlagValue][]>(() => {
	if (!data.value) return []
	const { project, ai, ...rest } = data.value
	const values: [string, ProjectFlagValue][] = [['project', project]]
	const aiFields = recordValue(ai)
	const byRule = recordValue(aiFields?.decisions_by_rule)
	const saved = props.savedRule
	const answers = saved && projectId.value ? recordValue(byRule?.[saved.id]) : undefined
	const savedQuestions = saved?.params.decision_questions ?? {}
	const decisions = Object.fromEntries(
		Object.entries(props.questions)
			.filter(([key]) => key)
			.map(([key, { question, include_context }]) => {
				const previous = Object.hasOwn(savedQuestions, key) ? savedQuestions[key] : undefined
				const sameQuestion =
					previous?.question === question &&
					previous.include_context.length === include_context.length &&
					include_context.every((field) => previous.include_context.includes(field))
				return [key, sameQuestion && answers && Object.hasOwn(answers, key) ? answers[key] : null]
			}),
	)
	values.push(['ai', { ...aiFields, decisions }])
	if (!projectId.value && Array.isArray(rest.scanned_files)) {
		const foundPatterns = [
			...new Set(
				props.patterns
					.filter(({ pattern }) => pattern.trim())
					.map(({ name }) => name.trim())
					.filter(Boolean),
			),
		]
		rest.scanned_files = rest.scanned_files.map((file) => {
			const fields = recordValue(file)
			return fields ? { ...fields, found_patterns: foundPatterns } : file
		})
	}
	return values.concat(Object.entries(rest))
})
const fieldQuery = ref('')
const fieldSearch = computed(() => createFieldSearch(fields.value))
const fieldFilter = computed(() => {
	const query = fieldQuery.value.trim()
	return query ? fieldSearch.value(query) : undefined
})
const visibleFields = computed(() => fieldFilter.value?.entries ?? fields.value)
</script>
