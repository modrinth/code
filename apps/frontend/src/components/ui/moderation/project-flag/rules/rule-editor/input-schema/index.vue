<template>
	<aside class="flex flex-col gap-4" aria-label="In scope">
		<div class="flex flex-col gap-1">
			<h2 class="m-0 flex min-h-[38px] items-center text-sm font-semibold text-primary">
				In scope
			</h2>
			<Input
				v-model="filterInput"
				aria-label="Filter scope"
				placeholder="Filter scope…"
				autocomplete="off"
			/>
		</div>

		<Accordion
			role="region"
			aria-label="Fields"
			open-by-default
			:button-class="sectionButtonClass"
			content-class="flex min-w-0 flex-col gap-2 pt-1"
		>
			<template #title>Fields</template>
			<div class="flex min-w-0 items-center gap-2">
				<span
					v-tooltip="
						'The schema example shows configured pattern names. Selecting a project previews its cached context. File scans run during rule tests.'
					"
					class="text-sm text-secondary"
					>Sample</span
				>
				<SampleProjectPicker v-model="projectId" class="min-w-0 max-w-52" />
			</div>
			<p v-if="isPending" role="status" class="m-0 flex items-center gap-2 text-sm text-secondary">
				<LoaderCircleIcon class="size-4 animate-spin" />
				Loading fields…
			</p>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<p class="m-0 text-sm text-red">Could not load fields.</p>
				<Button size="sm" :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
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
						:filter="filter"
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
					<li v-for="module in filteredModules" :key="module.name">
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
				class="!absolute right-6 top-0"
				type="quiet"
				size="xs"
				label="New module"
				@click="emit('newModule')"
			>
				<PackagePlusIcon aria-hidden="true" />
			</IconButton>
		</div>

		<Accordion
			v-if="filteredPackages.length"
			role="region"
			aria-label="Packages"
			open-by-default
			:button-class="sectionButtonClass"
			content-class="pt-1"
		>
			<template #title>Packages</template>
			<ul class="m-0 flex list-none flex-col gap-1 p-0">
				<li v-for="pkg in filteredPackages" :key="pkg.name" class="font-mono text-sm">
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
		<p
			v-if="
				filter &&
				!isPending &&
				!modulesPending &&
				!isError &&
				!modulesError &&
				!visibleFields.length &&
				!filteredModules.length &&
				!filteredPackages.length
			"
			class="m-0 text-sm text-secondary"
		>
			No matches.
		</p>
	</aside>
</template>

<script setup lang="ts">
import type {
	ProjectFlagModule,
	ProjectFlagRule,
	ProjectFlagRuleParams,
	ProjectFlagValue,
} from '@modrinth/api-client'
import { ExternalIcon, LoaderCircleIcon, PackagePlusIcon } from '@modrinth/assets'
import { Accordion, Button, IconButton, Input, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import {
	projectFlagInputSchemaQueryOptions,
	projectFlagModulesQueryOptions,
} from '~/composables/queries/project-flag'

import type { FileScanPattern } from '../file-scan-patterns-editor.vue'
import SchemaField from './field.vue'
import { fieldMatches, recordValue } from './fields'
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
	'flex w-full cursor-pointer items-center rounded border-0 bg-transparent px-0 py-1 text-left text-sm font-semibold transition-[filter] hover:brightness-125 focus-visible:brightness-125 motion-reduce:transition-none [&>div]:text-primary [&_svg]:text-secondary [&_svg]:size-4'
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
const filterInput = ref('')
const filter = computed(() => filterInput.value.trim().toLowerCase())
const filteredModules = computed(
	() => modules.value?.filter(({ name }) => name.toLowerCase().includes(filter.value)) ?? [],
)
const filteredPackages = computed(() =>
	packages.filter(({ name, description }) =>
		`${name} ${description}`.toLowerCase().includes(filter.value),
	),
)
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
const visibleFields = computed(() =>
	fields.value.filter(([name, value]) => fieldMatches(name, value, filter.value)),
)
</script>
