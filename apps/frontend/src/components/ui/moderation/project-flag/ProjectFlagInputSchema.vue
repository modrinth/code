<template>
	<div>
		<div class="flex flex-wrap items-center gap-4">
			<slot name="reference" />
			<button
				type="button"
				class="flex cursor-pointer items-center gap-1 border-0 bg-transparent p-0 text-sm font-semibold text-contrast"
				:aria-expanded="isOpen"
				:aria-controls="schemaId"
				@click="isOpen = !isOpen"
			>
				<ChevronRightIcon class="size-4 transition-transform" :class="{ 'rotate-90': isOpen }" />
				Input schema
			</button>
		</div>
		<div
			v-if="isOpen"
			:id="schemaId"
			class="mt-3 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
		>
			<div v-if="isPending" role="status" class="flex items-center gap-2 text-secondary">
				<LoaderCircleIcon class="size-4 animate-spin" />
				Loading schema…
			</div>
			<div v-else-if="isError" role="alert" class="flex flex-col items-start gap-2">
				<p class="m-0 text-sm text-red">Could not load the input schema: {{ error?.message }}</p>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<div v-else class="max-h-96 overflow-auto whitespace-pre-wrap break-words font-mono text-sm">
				{{ example }}
			</div>
		</div>
		<section class="mt-3 flex flex-col gap-3" aria-label="Project context inspector">
			<div class="flex flex-wrap items-center gap-3">
				<label :for="projectIdInputId" class="text-sm font-semibold text-contrast"
					>Project ID</label
				>
				<Input
					:id="projectIdInputId"
					v-model="projectIdInput"
					placeholder="Base62 project ID"
					autocomplete="off"
					:spellcheck="false"
					:aria-describedby="contextHelpId"
					@keydown.enter.prevent="loadContext"
				/>
				<Button :disabled="!isProjectIdValid || isContextFetching" @click="loadContext">
					Load context
				</Button>
			</div>
			<p :id="contextHelpId" class="m-0 text-sm text-secondary">
				Enter a base62 project ID, not a slug. This loads the project's current context without
				running rules or inference. No files are scanned, so scanned_files is empty. Project
				link_urls maps platform keys to objects containing platform, donation, and url. Decision
				questions independently select title, summary, description, links, slug, license (ID, name,
				URL), categories, additional categories, gallery titles, gallery descriptions, version
				names, version numbers, version changelogs, version filenames, and typed Java/Bedrock server
				listing addresses. Version text includes all available project versions. Only selected
				fields are sent; account/private data, moderation data, server ping responses, and arbitrary
				raw components are excluded. Named answers are available in ai.decisions when evaluating a
				rule.
			</p>
			<div
				v-if="submittedProjectId"
				class="rounded-xl border border-solid border-surface-5 bg-surface-2 p-3"
				:aria-busy="isContextFetching"
			>
				<p class="mb-3 mt-0 text-sm font-semibold text-contrast">
					Context for project {{ submittedProjectId }}
				</p>
				<div v-if="isContextFetching" role="status" class="flex items-center gap-2 text-secondary">
					<LoaderCircleIcon class="size-4 animate-spin" aria-hidden="true" />
					Loading project context…
				</div>
				<div v-else-if="isContextError" role="alert" class="flex flex-col items-start gap-2">
					<p class="m-0 text-sm text-red">
						Could not load the project context: {{ contextError?.message }}
					</p>
					<Button :disabled="isContextFetching" @click="refetchContext()"
						>Retry loading context</Button
					>
				</div>
				<pre
					v-else-if="context"
					aria-label="Project context JSON"
					tabindex="0"
					class="m-0 max-h-96 overflow-auto whitespace-pre-wrap break-words font-mono text-sm"
					>{{ formattedContext }}</pre
				>
			</div>
		</section>
		<details class="group mt-3">
			<summary
				class="flex w-fit cursor-pointer list-none items-center gap-1 text-sm font-semibold text-contrast [&::-webkit-details-marker]:hidden"
			>
				<ChevronRightIcon class="size-4 transition-transform group-open:rotate-90" />
				Imported packages
			</summary>
			<ul
				class="mb-0 mt-3 flex list-none flex-col gap-2 rounded-xl border border-solid border-surface-5 bg-surface-2 p-3 text-sm"
			>
				<li v-for="pkg in packages" :key="pkg.name">
					<a :href="pkg.docs" target="_blank" rel="noopener noreferrer" class="text-link">{{
						pkg.name
					}}</a>
					<span class="text-secondary"> — {{ pkg.description }}</span>
				</li>
			</ul>
		</details>
	</div>
</template>

<script setup lang="ts">
import { ChevronRightIcon, LoaderCircleIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, Input } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, ref, useId } from 'vue'

const packages = [
	{
		name: 'StandardPackage',
		description: 'Built-in functions for strings, arrays, maps, math, and more.',
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

const client = injectModrinthClient()
const isOpen = ref(false)
const schemaId = useId()
const { data, isPending, isError, error, isFetching, refetch } = useQuery({
	queryKey: ['moderation', 'project-flag', 'input-example'],
	queryFn: () => client.labrinth.project_flags_internal.getRuleSchema(),
	enabled: isOpen,
	staleTime: 60_000,
})
const example = computed(() => (data.value ? JSON.stringify(data.value, null, 2) : ''))

const projectIdInputId = useId()
const contextHelpId = useId()
const projectIdInput = ref('')
const submittedProjectId = ref('')
const isProjectIdValid = computed(() => /^[0-9A-Za-z]+$/.test(projectIdInput.value.trim()))
const {
	data: context,
	isFetching: isContextFetching,
	isError: isContextError,
	error: contextError,
	refetch: refetchContext,
} = useQuery({
	queryKey: computed(
		() => ['moderation', 'project-flag', 'context', submittedProjectId.value] as const,
	),
	queryFn: () => client.labrinth.project_flags_internal.getProjectContext(submittedProjectId.value),
	enabled: computed(() => !!submittedProjectId.value),
	staleTime: 60_000,
})
const formattedContext = computed(() =>
	context.value ? JSON.stringify(context.value, null, 2) : '',
)

function loadContext() {
	if (!isProjectIdValid.value || isContextFetching.value) return

	const projectId = projectIdInput.value.trim()
	if (submittedProjectId.value === projectId) {
		void refetchContext()
	} else {
		submittedProjectId.value = projectId
	}
}
</script>
