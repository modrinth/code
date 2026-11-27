<template>
	<section
		class="flex h-[22rem] min-w-0 shrink-0 flex-col border-0 border-t border-solid border-surface-4 bg-surface-2"
		aria-label="Test projects"
	>
		<header class="flex items-center gap-2 px-3 pb-0 pt-0.5">
			<h2 class="m-0 text-sm font-semibold text-contrast">Test projects</h2>
			<span class="rounded-full bg-surface-4 px-2 text-xs text-secondary">{{
				projectIds.length
			}}</span>
			<span v-if="data" class="text-xs text-secondary">Last run: {{ data.elapsedMs }} ms</span>
			<span
				v-tooltip="runDisabledReason ?? 'Run all tests (Cmd/Ctrl+Enter)'"
				class="ml-auto inline-flex rounded-xl"
			>
				<Button
					type="colored"
					color="brand"
					size="sm"
					:disabled="Boolean(runDisabledReason)"
					:class="{ 'pointer-events-none': runDisabledReason }"
					:loading="isPending"
					@click="evaluate"
				>
					<PlayIcon aria-hidden="true" />
					{{ isPending ? 'Running…' : 'Run all' }}
				</Button>
			</span>
		</header>

		<div class="test-panes grid min-h-0 flex-1" :style="testPanesStyle">
			<div
				ref="projectListPane"
				class="flex min-h-0 min-w-0 flex-col gap-2 border-0 border-solid border-surface-4 p-2 max-[420px]:border-b"
			>
				<Combobox
					:options="addOptions"
					:search-value="searchInput"
					:no-options-message="
						searchPending
							? 'Searching…'
							: searchError
								? 'Could not search projects'
								: 'No projects found'
					"
					:sync-with-selection="false"
					:show-chevron="false"
					:disabled="disabled"
					:min-search-length-to-open="1"
					:search-input-attrs="{
						'aria-label': 'Add test projects',
						class: '[&::-webkit-search-cancel-button]:hidden',
					}"
					searchable
					disable-search-filter
					search-placeholder="Add name, slug or IDs…"
					search-type="search"
					search-name="flag-rule-add-test-projects"
					search-autocomplete="off"
					:search-spellcheck="false"
					dropdown-width="24rem"
					@search-input="searchInput = $event"
					@select="addFromOption"
					@paste.capture="pasteIds"
				/>
				<p v-if="addError" role="alert" class="m-0 text-xs text-red">{{ addError }}</p>
				<ul class="m-0 flex min-h-0 list-none flex-col gap-1 overflow-y-auto p-0">
					<li
						v-for="id in projectIds"
						:key="id"
						class="flex min-w-0 items-center rounded-lg pr-1 hover:bg-surface-3"
						:class="activeId === id ? 'bg-surface-3 text-contrast' : 'text-primary'"
					>
						<button
							type="button"
							class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-lg border-0 bg-transparent p-2 text-left text-sm text-inherit"
							:aria-pressed="activeId === id"
							:aria-label="`${projectName(id)}, ${status(id).label}`"
							@click="activeId = id"
						>
							<Avatar :src="projects.get(id)?.icon_url ?? undefined" size="1.5rem" no-shadow />
							<span class="min-w-0 flex-1 truncate">{{ projectName(id) }}</span>
							<span class="shrink-0 text-xs" :class="status(id).color">{{ status(id).label }}</span>
						</button>
						<IconButton
							v-tooltip="'Remove test project'"
							class="shrink-0 enabled:hover:text-red"
							type="quiet"
							size="xs"
							:disabled="disabled"
							:label="`Remove ${projectName(id)} from test projects`"
							@click="removeProject(id)"
						>
							<TrashIcon aria-hidden="true" />
						</IconButton>
					</li>
				</ul>
			</div>

			<PanelDivider
				v-model="savedProjectListWidth"
				class="max-[420px]:hidden"
				label="Resize test project list width"
				side="left"
				width-property="--project-list-width"
				responsive
				:min="144"
				:reserve="176"
			/>
			<section
				v-if="activeId"
				:aria-label="`Results for ${projectName(activeId)}`"
				class="flex min-h-0 min-w-0 flex-col"
			>
				<div class="flex flex-wrap items-center gap-2 px-3 pt-2">
					<Avatar :src="projects.get(activeId)?.icon_url ?? undefined" size="1.75rem" no-shadow />
					<h3 class="m-0 min-w-0 truncate text-sm font-semibold text-contrast">
						{{ projectName(activeId) }}
					</h3>
					<code v-if="projects.has(activeId)" class="text-xs text-secondary">{{ activeId }}</code>
				</div>
				<Tabs
					v-model:value="activeView"
					:tabs="outputTabs"
					wrap
					color="gray"
					aria-label="Test result views"
					class="!h-auto !w-full !rounded-none !border-x-0 !border-t-0 !border-b-surface-4 !px-2 !py-0 !shadow-none [&_[role=tab][aria-selected=true]]:!border-b-green [&_[role=tab]]:!rounded-none [&_[role=tab]]:!border-0 [&_[role=tab]]:!border-b-2 [&_[role=tab]]:!border-transparent [&_[role=tab]]:!bg-transparent [&_[role=tab]]:py-2"
				/>
				<div class="min-h-0 flex-1 overflow-auto p-3" :aria-busy="isProjectRunning(activeId)">
					<p v-if="isProjectRunning(activeId)" role="status" class="m-0 text-sm text-secondary">
						Running tests…
					</p>
					<p
						v-else-if="error && requestedProjects.has(activeId)"
						role="alert"
						class="m-0 whitespace-pre-wrap break-words text-sm text-red"
					>
						{{ error.message }}
					</p>
					<p v-else-if="!result" class="m-0 text-sm text-secondary">
						Run all to see this project’s results.
					</p>
					<pre
						v-else-if="activeView === 'raw'"
						class="m-0 whitespace-pre-wrap break-words font-mono text-xs"
						v-text="JSON.stringify(result, null, 2)"
					/>
					<pre
						v-else-if="activeView === 'log' && result.log.length"
						class="m-0 whitespace-pre-wrap break-words font-mono text-sm"
						v-text="result.log.join('\n')"
					/>
					<p v-else-if="activeView === 'log'" class="m-0 text-sm text-secondary">
						No print output.
					</p>
					<p
						v-else-if="result.error"
						role="alert"
						class="m-0 whitespace-pre-wrap break-words text-sm text-red"
					>
						{{ result.error }}
					</p>
					<p v-else-if="!result.flags.length" class="m-0 text-sm text-secondary">
						No issues found.
					</p>
					<ul v-else class="m-0 flex list-none flex-col gap-2 p-0">
						<li
							v-for="(flag, index) in result.flags"
							:key="index"
							class="flex min-w-0 flex-wrap items-baseline gap-x-2 gap-y-1 text-sm"
						>
							<code
								class="bg-red-highlight max-w-full break-all rounded-md px-2 py-0.5 font-mono text-red"
								>{{ flag.key }}</code
							>
							<span v-if="flag.reason" class="min-w-0 whitespace-pre-wrap break-words">{{
								flag.reason
							}}</span>
						</li>
					</ul>
				</div>
			</section>
			<p v-else class="m-0 p-3 text-sm text-secondary">Add projects to test this rule.</p>
		</div>
	</section>
</template>

<script setup lang="ts">
import type { EvalProjectFlagRuleRequest, ProjectFlagRuleParams } from '@modrinth/api-client'
import { PlayIcon, TrashIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	Combobox,
	type ComboboxOption,
	IconButton,
	injectModrinthClient,
	Tabs,
	type TabsValue,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { refDebounced, useLocalStorage } from '@vueuse/core'
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue'

import PanelDivider from '~/components/project-review/tech-review/panel-divider.vue'
import { projectFlagProjectContextQueryKey } from '~/composables/queries/project-flag'

const projectIdPattern = /^[A-Za-z0-9]{1,11}$/
const base62Digits = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
const props = defineProps<{
	params: ProjectFlagRuleParams
	maxProjects: number
	hasUnsavedModules: boolean
	canTest: boolean
	disabled: boolean
}>()
const projectIds = defineModel<string[]>('projectIds', { required: true })
const projectListPane = useTemplateRef<HTMLElement>('projectListPane')
const savedProjectListWidth = useLocalStorage('moderation-flag-rule-test-list-width', 0)
const testPanesStyle = computed(() => ({
	'--project-list-width': savedProjectListWidth.value
		? `${savedProjectListWidth.value}px`
		: undefined,
}))
const client = injectModrinthClient()
const queryClient = useQueryClient()
const activeProjectId = ref<string>()
const activeId = computed({
	get: () =>
		activeProjectId.value && projectIds.value.includes(activeProjectId.value)
			? activeProjectId.value
			: projectIds.value[0],
	set: (id) => {
		activeProjectId.value = id
	},
})
const activeView = ref<TabsValue>('issues')
watch(activeId, () => {
	activeView.value = 'issues'
})
const searchInput = ref('')
const query = refDebounced(
	computed(() => searchInput.value.trim()),
	250,
)
const addError = ref('')

const { data: projectDetails } = useQuery(
	computed(() => ({
		queryKey: ['moderation', 'project-flag', 'test-projects', projectIds.value] as const,
		queryFn: ({ queryKey: [, , , ids] }) => client.labrinth.projects_v2.getMultiple([...ids]),
		enabled: projectIds.value.length > 0,
		placeholderData: (previous) => previous,
		staleTime: 60_000,
	})),
)
const projects = computed(
	() => new Map(projectDetails.value?.map((project) => [project.id, project])),
)
function projectName(id: string) {
	return projects.value.get(id)?.title ?? id
}

const {
	data: search,
	isPending: searchPending,
	isError: searchError,
} = useQuery(
	computed(() => ({
		queryKey: ['moderation', 'project-flag', 'test-project-search', query.value] as const,
		queryFn: async ({ queryKey: [, , , term] }) => {
			const [search, exact] = await Promise.all([
				client.labrinth.projects_v2.search({ query: term, limit: 10 }),
				/^[\w-]+$/.test(term)
					? client.labrinth.projects_v2.get(term).catch(() => undefined)
					: undefined,
			])
			return { hits: search.hits, exact }
		},
		enabled: Boolean(query.value),
		staleTime: 30_000,
	})),
)

function parseIds(input: string) {
	const ids = input
		.trim()
		.split(/[,\s]+/)
		.filter(Boolean)
		.map((id) => id.replace(/^0+(?=.)/, ''))
	if (!ids.length || !ids.every(isProjectId)) return undefined
	return ids
}

function isProjectId(id: string) {
	if (!projectIdPattern.test(id)) return false
	const value = Array.from(id).reduce(
		(acc, digit) => acc * 62n + BigInt(base62Digits.indexOf(digit)),
		0n,
	)
	return value < 1n << 64n
}

const addOptions = computed<ComboboxOption<string[]>[]>(() => {
	const term = searchInput.value.trim()
	if (!term) return []
	const ids = parseIds(term)
	if (term !== query.value || searchPending.value) return []
	const options = new Map<string, ComboboxOption<string[]>>()
	const exact = search.value?.exact
	if (exact) options.set(exact.id, { label: exact.title, subLabel: exact.slug, value: [exact.id] })
	for (const hit of search.value?.hits ?? []) {
		if (!options.has(hit.project_id))
			options.set(hit.project_id, { label: hit.title, subLabel: hit.slug, value: [hit.project_id] })
	}
	for (const id of projectIds.value) options.delete(id)
	const [id] = ids ?? []
	if (id && ids?.length === 1 && !exact && !options.has(id) && !projectIds.value.includes(id)) {
		options.set(id, { label: `Add ID ${id}`, value: ids })
	} else if (ids && ids.length > 1) {
		options.set(term, { label: `Add ${ids.length} project IDs`, value: ids })
	}
	return [...options.values()]
})

function addFromOption(option: ComboboxOption<string[]>) {
	if (!option.value) return
	addProjects(option.value)
	searchInput.value = ''
}

function pasteIds(event: ClipboardEvent) {
	const text = event.clipboardData?.getData('text').trim() ?? ''
	const ids = parseIds(text)
	if (!ids || ids.length < 2) return
	if (!/[,\r\n\t]/.test(text) && !ids.every((id) => id.length === 8)) return
	event.preventDefault()
	searchInput.value = ''
	addProjects(ids)
}

function addProjects(ids: string[]) {
	addError.value = ''
	const next = [...new Set([...projectIds.value, ...ids])]
	if (next.length > props.maxProjects) {
		addError.value = `Add up to ${props.maxProjects} unique project IDs.`
		return
	}
	projectIds.value = next
}

async function removeProject(id: string) {
	const index = projectIds.value.indexOf(id)
	const remaining = projectIds.value.filter((entry) => entry !== id)
	const nextIndex = Math.min(index, remaining.length - 1)
	if (activeId.value === id) activeId.value = remaining[nextIndex]
	projectIds.value = remaining
	addError.value = ''
	await nextTick()
	const pane = projectListPane.value
	const nextProject = pane?.querySelectorAll<HTMLButtonElement>('li > button[aria-pressed]')[
		nextIndex
	]
	const focusTarget = nextProject ?? pane?.querySelector('input')
	focusTarget?.focus()
}

const {
	mutate: run,
	data,
	isPending,
	error,
	variables,
} = useMutation({
	mutationFn: async (request: EvalProjectFlagRuleRequest) => {
		const start = performance.now()
		const results = await client.labrinth.project_flags_internal.evalRule(request)
		return { results, elapsedMs: Math.round(performance.now() - start) }
	},
	retry: false,
	onSuccess: (_, { project_ids }) => {
		const testedProjects = new Set(project_ids)
		void queryClient.invalidateQueries({
			queryKey: projectFlagProjectContextQueryKey,
			predicate: ({ queryKey }) => {
				const projectId = queryKey[projectFlagProjectContextQueryKey.length]
				return typeof projectId === 'string' && testedProjects.has(projectId)
			},
		})
	},
})
const requestedProjects = computed(() => new Set(variables.value?.project_ids))
const results = computed(
	() => new Map(data.value?.results.map((result) => [result.project_id, result])),
)
const result = computed(() => (activeId.value ? results.value.get(activeId.value) : undefined))
const outputTabs = computed(() => [
	{ value: 'issues', label: result.value ? `Issues (${result.value.flags.length})` : 'Issues' },
	{ value: 'log', label: result.value ? `Log (${result.value.log.length})` : 'Log' },
	{ value: 'raw', label: 'Raw JSON' },
])

function isProjectRunning(id: string) {
	return isPending.value && requestedProjects.value.has(id)
}

function status(id: string) {
	if (isProjectRunning(id)) return { label: 'Running…', color: 'text-secondary' }
	if (error.value && requestedProjects.value.has(id)) return { label: 'Error', color: 'text-red' }
	const result = results.value.get(id)
	if (!result) return { label: 'Not run', color: 'text-secondary' }
	if (result.error === 'project not found') return { label: 'Not found', color: 'text-orange' }
	if (result.error) return { label: 'Error', color: 'text-red' }
	if (!result.flags.length) return { label: 'Pass', color: 'text-green' }
	return {
		label: `${result.flags.length} ${result.flags.length === 1 ? 'flag' : 'flags'}`,
		color: 'text-red',
	}
}

const runDisabledReason = computed(() => {
	if (isPending.value) return 'A test is running.'
	if (!projectIds.value.length) return 'Add a project to test.'
	if (props.disabled) return 'Wait for the save to finish.'
	if (props.hasUnsavedModules) return 'Save or discard module changes before testing.'
	if (!props.canTest) return 'Fix rule errors before testing.'
	return undefined
})

function evaluate() {
	if (runDisabledReason.value) return
	run({ project_ids: [...projectIds.value], params: props.params })
}

defineExpose({ evaluate })
</script>

<style scoped>
.test-panes {
	grid-template-columns:
		clamp(
			144px,
			var(--project-list-width-drag, var(--project-list-width, 33.333%)),
			calc(100% - 184px)
		)
		8px minmax(0, 1fr);
}

@media (max-width: 420px) {
	.test-panes {
		grid-template-columns: minmax(0, 1fr);
		grid-template-rows: minmax(0, 2fr) minmax(0, 3fr);
	}
}
</style>
