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
				class="flex min-h-0 min-w-0 flex-col gap-2 border-0 border-solid border-surface-4 p-2 max-[420px]:border-b"
			>
				<div class="flex items-center gap-1">
					<Combobox
						:model-value="selectedListName"
						:options="listOptions"
						:disabled="disabled"
						class="min-w-0 flex-1"
						placeholder="Project list"
						trigger-size="sm"
						dropdown-min-width="12rem"
						@update:model-value="selectList"
					/>
					<IconButton
						v-tooltip="'Edit project list'"
						class="shrink-0"
						label="Edit project list"
						aria-haspopup="dialog"
						size="sm"
						:circular="false"
						:disabled="disabled || !selectedListName"
						@click="openListEditor(selectedListName)"
					>
						<EditIcon aria-hidden="true" />
					</IconButton>
					<IconButton
						v-tooltip="'New project list'"
						class="shrink-0"
						label="New project list"
						aria-haspopup="dialog"
						size="sm"
						:circular="false"
						:disabled="disabled"
						@click="openListEditor()"
					>
						<PlusIcon aria-hidden="true" />
					</IconButton>
				</div>
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
						<NuxtLink
							:to="`/project/${encodeURIComponent(projects.get(activeId)?.slug ?? activeId)}`"
							target="_blank"
							rel="noopener"
							class="text-inherit hover:underline focus-visible:underline"
						>
							{{ projectName(activeId) }}
						</NuxtLink>
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
					<pre
						v-else-if="activeView === 'error' && activeError"
						role="alert"
						class="m-0 whitespace-pre-wrap break-words font-mono text-sm text-red"
						v-text="activeError"
					/>
					<p v-else-if="!result" class="m-0 text-sm text-secondary">
						{{ activeError ? 'Test failed.' : 'Run all to see this project’s results.' }}
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
					<p v-else-if="result.error" class="m-0 text-sm text-secondary">Test failed.</p>
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
		<NewModal
			ref="listModal"
			:header="editingListName ? 'Edit project list' : 'New project list'"
			width="100%"
			max-width="36rem"
			:initial-focus="() => listForm?.querySelector('input') ?? null"
		>
			<form
				:id="listFormId"
				ref="listForm"
				class="flex flex-col gap-4"
				@submit.prevent="saveList"
			>
				<div class="flex flex-col gap-2">
					<label :for="listNameId" class="text-sm font-medium text-contrast">List name</label>
					<Input
						:id="listNameId"
						v-model="draftListName"
						:disabled="disabled"
						autocomplete="off"
						@input="listError = ''"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<span class="text-sm font-medium text-contrast">
						Projects ({{ draftProjectIds.length }})
					</span>
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
						search-placeholder="Project name, id or slug"
						search-type="search"
						search-name="flag-rule-add-test-projects"
						search-autocomplete="off"
						:search-spellcheck="false"
						@search-input="searchInput = $event"
						@select="addFromOption"
					/>
					<ul
						v-if="draftProjectIds.length"
						class="m-0 max-h-40 list-none overflow-y-auto rounded-xl bg-surface-3 p-1"
					>
						<li
							v-for="id in draftProjectIds"
							:key="id"
							class="flex items-center gap-2 px-2 py-1 text-sm"
						>
							<Avatar :src="draftProjects.get(id)?.icon_url ?? undefined" size="1.5rem" no-shadow />
							<span class="min-w-0 flex-1 truncate">{{ draftProjects.get(id)?.title ?? id }}</span>
							<IconButton
								v-tooltip="'Remove project'"
								:label="`Remove ${draftProjects.get(id)?.title ?? id}`"
								:disabled="disabled"
								type="quiet"
								size="xs"
								@click="removeDraftProject(id)"
							>
								<TrashIcon aria-hidden="true" />
							</IconButton>
						</li>
					</ul>
					<div
						v-if="draftProjectIds.length"
						class="min-w-0 rounded-xl border border-solid border-surface-5 bg-surface-3 p-2"
					>
						<div class="flex items-center justify-between gap-2">
							<span class="text-sm text-secondary">Project IDs</span>
							<CopyCode type="button" :text="draftProjectIdsText" display-text="Copy IDs" />
						</div>
						<pre
							class="m-0 mt-2 max-h-24 overflow-auto whitespace-pre-wrap break-words font-mono text-xs"
							tabindex="0"
							aria-label="Project IDs"
							v-text="draftProjectIdsText"
						/>
					</div>
				</div>
				<div class="flex flex-col gap-2">
					<label :for="importInputId" class="text-sm text-secondary">
						Add comma-separated project IDs
					</label>
					<Textarea
						:id="importInputId"
						v-model="importInput"
						:rows="5"
						:spellcheck="false"
						:disabled="disabled"
						:error="Boolean(importError)"
						:aria-describedby="importError ? importErrorId : undefined"
						@input="importError = ''"
					/>
					<p v-if="importError" :id="importErrorId" role="alert" class="m-0 text-sm text-red">
						{{ importError }}
					</p>
					<Button
						class="self-start"
						:disabled="disabled || !importInput.trim()"
						@click="addImportedIds"
					>
						Add IDs
					</Button>
				</div>
				<p v-if="listError" role="alert" class="m-0 text-sm text-red">{{ listError }}</p>
			</form>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button @click="listModal?.hide()">Cancel</Button>
					<Button
						native-type="submit"
						:form="listFormId"
						type="colored"
						color="brand"
						:disabled="disabled || !draftListName.trim()"
					>
						Save list
					</Button>
				</div>
			</template>
		</NewModal>
	</section>
</template>

<script setup lang="ts">
import type {
	EvalProjectFlagRuleRequest,
	Labrinth,
	ProjectFlagEvaluationResult,
	ProjectFlagRuleParams,
} from '@modrinth/api-client'
import { ModrinthServerError } from '@modrinth/api-client'
import { EditIcon, PlayIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	Avatar,
	Button,
	Combobox,
	type ComboboxOption,
	CopyCode,
	IconButton,
	Input,
	injectModrinthClient,
	NewModal,
	Tabs,
	type TabsValue,
	Textarea,
} from '@modrinth/ui'
import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { refDebounced, useLocalStorage } from '@vueuse/core'
import { computed, onMounted, ref, useId, useTemplateRef, watch } from 'vue'

import PanelDivider from '~/components/project-review/tech-review/panel-divider.vue'
import { projectFlagProjectContextQueryKey } from '~/composables/queries/project-flag'

const projectIdPattern = /^[A-Za-z0-9]{1,11}$/
const base62Digits = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz'
const defaultListName = 'Default'
const projectDetailsBatchSize = 100
const ruleEvaluationBatchSize = 20
const props = defineProps<{
	initialProjectId?: string
	params: ProjectFlagRuleParams
	hasUnsavedModules: boolean
	canTest: boolean
	disabled: boolean
}>()
const accountId = useAuthState().value.user?.id ?? ''
const legacyTestProjects = useLocalStorage<Partial<Record<string, string[]>>>(
	`moderation-project-flag-test-projects:${accountId}`,
	{},
	{ initOnMounted: true, writeDefaults: false },
)
const savedLists = useLocalStorage<{ lists: Record<string, string[]>; selectedName: string }>(
	`moderation-project-flag-test-project-lists:${accountId}`,
	{ lists: {}, selectedName: '' },
	{ initOnMounted: true },
)
const listNames = computed(() => Object.keys(savedLists.value.lists))
const selectedListName = computed(() => {
	const { lists, selectedName } = savedLists.value
	return Object.hasOwn(lists, selectedName) ? selectedName : (listNames.value[0] ?? '')
})
const listOptions = computed(() => listNames.value.map((name) => ({ label: name, value: name })))
const projectIds = computed(() => savedLists.value.lists[selectedListName.value] ?? [])

onMounted(() => {
	if (!listNames.value.length) {
		const legacyIds =
			legacyTestProjects.value.shared ??
			Object.values(legacyTestProjects.value).flatMap((ids) => ids ?? [])
		savedLists.value = {
			lists: { [defaultListName]: [...new Set(legacyIds)] },
			selectedName: defaultListName,
		}
	}

	const id = props.initialProjectId
	if (id && !projectIds.value.includes(id)) {
		const selectedName = selectedListName.value
		savedLists.value = {
			lists: { ...savedLists.value.lists, [selectedName]: [...projectIds.value, id] },
			selectedName,
		}
	}
})

function selectList(name: string) {
	savedLists.value = { ...savedLists.value, selectedName: name }
	activeProjectId.value = undefined
}

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
const searchInput = ref('')
const query = refDebounced(
	computed(() => searchInput.value.trim()),
	250,
)
const listModal = useTemplateRef<InstanceType<typeof NewModal>>('listModal')
const listForm = useTemplateRef<HTMLFormElement>('listForm')
const listFormId = useId()
const listNameId = `${listFormId}-name`
const importInputId = `${listFormId}-ids`
const importErrorId = `${listFormId}-error`
const editingListName = ref('')
const draftListName = ref('')
const draftProjectIds = ref<string[]>([])
const draftProjectIdsText = computed(() => draftProjectIds.value.join(', '))
const listError = ref('')
const importInput = ref('')
const importError = ref('')

function* projectBatches(ids: string[], batchSize: number) {
	for (let offset = 0; offset < ids.length; offset += batchSize) {
		yield ids.slice(offset, offset + batchSize)
	}
}

function projectDetailsOptions(ids: string[]) {
	return queryOptions({
		queryKey: ['moderation', 'project-flag', 'test-projects', ids] as const,
		queryFn: async ({ signal }) => {
			const projects: Labrinth.Projects.v2.Project[] = []
			for (const batch of projectBatches(ids, projectDetailsBatchSize)) {
				signal.throwIfAborted()
				projects.push(...(await client.labrinth.projects_v2.getMultiple(batch)))
			}
			return projects
		},
		enabled: ids.length > 0,
		staleTime: 60_000,
	})
}

const { data: projectDetails } = useQuery(computed(() => projectDetailsOptions(projectIds.value)))
const { data: draftProjectDetails } = useQuery(
	computed(() => ({
		...projectDetailsOptions(draftProjectIds.value),
		placeholderData: (previous) => previous,
	})),
)
const projects = computed(
	() => new Map(projectDetails.value?.map((project) => [project.id, project])),
)
const draftProjects = computed(
	() => new Map(draftProjectDetails.value?.map((project) => [project.id, project])),
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

function normalizeProjectId(input: string) {
	return input.trim().replace(/^0+(?=.)/, '')
}

function isProjectId(id: string) {
	if (!projectIdPattern.test(id)) return false
	const value = Array.from(id).reduce(
		(acc, digit) => acc * 62n + BigInt(base62Digits.indexOf(digit)),
		0n,
	)
	return value < 1n << 64n
}

const addOptions = computed<ComboboxOption<string>[]>(() => {
	const term = searchInput.value.trim()
	if (!term) return []
	if (term !== query.value || searchPending.value) return []
	const options = new Map<string, ComboboxOption<string>>()
	const exact = search.value?.exact
	if (exact) options.set(exact.id, { label: exact.title, subLabel: exact.slug, value: exact.id })
	for (const hit of search.value?.hits ?? []) {
		if (!options.has(hit.project_id))
			options.set(hit.project_id, { label: hit.title, subLabel: hit.slug, value: hit.project_id })
	}
	for (const id of draftProjectIds.value) options.delete(id)
	const id = normalizeProjectId(term)
	if (isProjectId(id) && !exact && !options.has(id) && !draftProjectIds.value.includes(id)) {
		options.set(id, { label: `Add ID ${id}`, value: id })
	}
	return [...options.values()]
})

function openListEditor(name = '') {
	editingListName.value = name
	draftListName.value = name
	draftProjectIds.value = name ? [...(savedLists.value.lists[name] ?? [])] : []
	listError.value = ''
	importInput.value = ''
	importError.value = ''
	searchInput.value = ''
	listModal.value?.show()
}

function addDraftProjects(ids: string[]) {
	draftProjectIds.value = [...new Set([...draftProjectIds.value, ...ids])]
}

function addFromOption(option: ComboboxOption<string>) {
	if (props.disabled || !option.value) return
	addDraftProjects([option.value])
	searchInput.value = ''
}

function removeDraftProject(id: string) {
	draftProjectIds.value = draftProjectIds.value.filter((entry) => entry !== id)
	listError.value = ''
	importError.value = ''
}

function addImportedIds() {
	if (props.disabled) return false
	if (!importInput.value.trim()) return true
	const ids = importInput.value.split(',').map(normalizeProjectId)
	if (!ids.every(isProjectId)) {
		importError.value = 'Enter valid project IDs separated by commas.'
		return false
	}
	addDraftProjects(ids)
	importError.value = ''
	importInput.value = ''
	return true
}

function saveList() {
	const name = draftListName.value.trim()
	if (props.disabled || !name) return
	if (name !== editingListName.value && Object.hasOwn(savedLists.value.lists, name)) {
		listError.value = 'A project list with this name already exists.'
		return
	}
	if (!addImportedIds()) return

	const lists = { ...savedLists.value.lists, [name]: [...draftProjectIds.value] }
	if (editingListName.value && editingListName.value !== name) delete lists[editingListName.value]
	savedLists.value = { lists, selectedName: name }
	listModal.value?.hide()
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
		const results: ProjectFlagEvaluationResult[] = []
		const requestErrors = new Map<string, string>()
		for (const batch of projectBatches(request.project_ids, ruleEvaluationBatchSize)) {
			try {
				const evaluated = await client.labrinth.project_flags_internal.evalRule({
					...request,
					project_ids: batch,
				})
				results.push(...evaluated)
			} catch (failure) {
				const message = formatRequestError(failure)
				for (const id of batch) requestErrors.set(id, message)
			}
		}
		return { results, requestErrors, elapsedMs: Math.round(performance.now() - start) }
	},
	retry: false,
	onSuccess: ({ results }) => {
		if (!results.length) return
		const testedProjects = new Set(results.map((result) => result.project_id))
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
function formatRequestError(failure: unknown) {
	if (!(failure instanceof ModrinthServerError) || !failure.v1Error) {
		return failure instanceof Error && failure.message ? failure.message : 'Test request failed.'
	}

	const { description, details } = failure.v1Error
	if (details == null) return description
	const causes = Array.isArray(details) ? details : [details]
	return [
		description,
		...causes.map((cause) => (typeof cause === 'string' ? cause : JSON.stringify(cause, null, 2))),
	].join('\n')
}

function projectRequestError(id: string) {
	if (error.value && requestedProjects.value.has(id)) return formatRequestError(error.value)
	return data.value?.requestErrors.get(id)
}

const activeError = computed(() => {
	if (!activeId.value) return undefined
	return projectRequestError(activeId.value) ?? result.value?.error
})
watch([activeId, activeError], ([id, failure], [previousId]) => {
	if (failure) {
		activeView.value = 'error'
	} else if (id !== previousId || activeView.value === 'error') {
		activeView.value = 'issues'
	}
})
const outputTabs = computed(() => [
	{ value: 'issues', label: result.value ? `Issues (${result.value.flags.length})` : 'Issues' },
	{ value: 'log', label: result.value ? `Log (${result.value.log.length})` : 'Log' },
	...(activeError.value ? [{ value: 'error', label: 'Error' }] : []),
	{ value: 'raw', label: 'Raw JSON' },
])

function isProjectRunning(id: string) {
	return isPending.value && requestedProjects.value.has(id)
}

function status(id: string) {
	if (isProjectRunning(id)) return { label: 'Running…', color: 'text-secondary' }
	if (projectRequestError(id)) return { label: 'Error', color: 'text-red' }
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
