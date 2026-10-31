<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<NavTabs :links="tabs" query="tab" />
			<Button
				v-if="!showRuns"
				type="colored"
				color="brand"
				:disabled="isBusy || isEditing"
				@click="openCreate"
			>
				<PlusIcon />
				Create rule
			</Button>
		</div>

		<div v-if="showRuns" class="universal-card text-secondary">
			Run history is not available yet.
		</div>

		<NewModal
			ref="ruleModal"
			:header="editingId === null ? 'Create rule' : 'Edit rule'"
			:disable-close="isSaving && isEditing"
			:on-hide="handleEditorHide"
		>
			<form
				v-if="isEditing"
				class="flex w-[48rem] max-w-full flex-col gap-3"
				@submit.prevent="saveRule"
			>
				<label for="project-flag-name" class="font-semibold text-contrast">Name</label>
				<Input
					id="project-flag-name"
					v-model="form.name"
					:maxlength="1023"
					:disabled="isSaving"
					placeholder="Rule name"
				/>
				<label for="project-flag-expression" class="font-semibold text-contrast"
					>Rhai expression</label
				>
				<RhaiExpressionEditor
					id="project-flag-expression"
					v-model="form.expression"
					:disabled="isSaving"
				/>
				<ProjectFlagInputSchema>
					<template #reference>
						<a
							href="https://rhai.rs/book/language/comments.html"
							target="_blank"
							rel="noopener noreferrer"
							class="w-fit text-sm text-link"
						>
							Rhai reference
						</a>
					</template>
				</ProjectFlagInputSchema>
				<ProjectFlagRuleTester
					v-model:patterns="patterns"
					:pattern-errors="patternErrors"
					:expression="form.expression"
					:disabled="isSaving"
				/>
				<p
					v-if="Array.from(form.expression).length > 65535"
					role="alert"
					class="m-0 text-sm text-red"
				>
					The expression must be at most 65,535 characters.
				</p>
				<p v-if="saveError" role="alert" class="m-0 text-sm text-red">
					Could not save the rule: {{ errorMessage(saveError) }}
				</p>
				<div class="flex justify-end gap-2">
					<Button :disabled="isSaving" @click="closeEditor">Cancel</Button>
					<Button
						type="colored"
						color="brand"
						native-type="submit"
						:disabled="isSaving || !isValid"
					>
						<LoaderCircleIcon v-if="isSaving" class="animate-spin" />
						{{ isSaving ? 'Saving…' : 'Save rule' }}
					</Button>
				</div>
			</form>
		</NewModal>

		<div
			v-if="!showRuns && isPending"
			role="status"
			class="universal-card flex items-center justify-center gap-2 py-8"
		>
			<LoaderCircleIcon class="animate-spin" />
			Loading project flag rules…
		</div>
		<div
			v-else-if="!showRuns && isError"
			role="alert"
			class="universal-card flex flex-col items-start gap-3"
		>
			<p class="m-0 text-red">Could not load project flag rules: {{ errorMessage(error) }}</p>
			<Button :disabled="isFetching" @click="refetch()">Try again</Button>
		</div>
		<template v-else-if="!showRuns">
			<div v-if="rules.length === 0" class="universal-card flex flex-col gap-2 py-8 text-center">
				<h2 class="m-0 text-lg font-bold text-contrast">
					{{ offset === 0 ? 'No project flag rules' : 'No rules on this page' }}
				</h2>
				<p class="m-0 text-secondary">
					{{
						offset === 0
							? 'Create a rule to get started.'
							: 'Go to the previous page to see more rules.'
					}}
				</p>
			</div>
			<article
				v-for="rule in rules"
				:key="rule.id"
				class="universal-card flex min-w-0 flex-col gap-3"
			>
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div class="min-w-0">
						<h2 class="m-0 break-words text-lg font-bold text-contrast">{{ rule.name }}</h2>
						<p class="m-0 mt-1 text-sm text-secondary">
							Created {{ formatDate(rule.created_at) }} · Updated {{ formatDate(rule.updated_at) }}
						</p>
					</div>
					<div class="flex gap-2">
						<Button :disabled="isBusy || isEditing" @click="openEdit(rule)">
							<EditIcon />
							Edit
						</Button>
						<Button
							type="colored"
							color="red"
							:disabled="isBusy || isEditing"
							@click="confirmDelete(rule)"
						>
							<TrashIcon />
							Delete
						</Button>
					</div>
				</div>
				<pre
					class="m-0 max-h-64 overflow-auto rounded-lg bg-bg-raised p-3 text-sm"
				><code>{{ rule.params.expression }}</code></pre>
				<div
					v-if="ruleToDelete?.id === rule.id"
					class="flex flex-col gap-3 rounded-lg border border-divider p-3"
					role="group"
					aria-label="Confirm rule deletion"
				>
					<p class="m-0 text-contrast">Delete “{{ rule.name }}”? This cannot be undone.</p>
					<p class="m-0 text-sm text-secondary">
						Existing flags will be preserved without a link to this rule.
					</p>
					<p v-if="deleteError" role="alert" class="m-0 text-sm text-red">
						Could not delete the rule: {{ errorMessage(deleteError) }}
					</p>
					<div class="flex justify-end gap-2">
						<Button :disabled="isDeleting" @click="ruleToDelete = null">Cancel</Button>
						<Button type="colored" color="red" :disabled="isDeleting" @click="deleteRule(rule.id)">
							<LoaderCircleIcon v-if="isDeleting" class="animate-spin" />
							{{ isDeleting ? 'Deleting…' : 'Delete rule' }}
						</Button>
					</div>
				</div>
			</article>

			<div
				v-if="offset > 0 || rules.length > 0"
				class="flex flex-wrap items-center justify-between gap-3"
			>
				<p class="m-0 text-sm text-secondary" role="status">
					<template v-if="rules.length"
						>Showing {{ offset + 1 }}–{{ offset + rules.length }}</template
					>
					<template v-else>No rules on this page</template>
				</p>
				<div class="flex gap-2">
					<Button
						:disabled="offset === 0 || isFetching || isBusy || isEditing"
						@click="changePage(-1)"
					>
						Previous
					</Button>
					<Button
						:disabled="!hasNextPage || isFetching || isBusy || isEditing"
						@click="changePage(1)"
					>
						Next
					</Button>
				</div>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import {
	EditIcon,
	HistoryIcon,
	LoaderCircleIcon,
	PlusIcon,
	TagCategoryFlagIcon,
	TrashIcon,
} from '@modrinth/assets'
import { Button, injectModrinthClient, Input, NavTabs, NewModal } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import type { FileScanPattern } from '~/components/ui/moderation/project-flag/FileScanPatternsEditor.vue'
import ProjectFlagInputSchema from '~/components/ui/moderation/project-flag/ProjectFlagInputSchema.vue'
import ProjectFlagRuleTester from '~/components/ui/moderation/project-flag/ProjectFlagRuleTester.vue'
import RhaiExpressionEditor from '~/components/ui/moderation/project-flag/RhaiExpressionEditor.vue'

useHead({ title: 'Project flag rules - Modrinth' })

const route = useRoute()
const showRuns = computed(() => route.query.tab === 'runs')
const tabs = [
	{ label: 'Rules', href: '', icon: TagCategoryFlagIcon },
	{ label: 'Runs', href: 'runs', icon: HistoryIcon },
]

const client = injectModrinthClient()
const api = client.labrinth.project_flags_internal
const queryClient = useQueryClient()
type Rule = Awaited<ReturnType<typeof api.get>>

const PAGE_SIZE = 25
const queryKey = ['moderation', 'project-flag', 'rules'] as const
const offset = ref(0)
const { data, isPending, isError, error, isFetching, refetch } = useQuery({
	queryKey: computed(() => [...queryKey, { limit: PAGE_SIZE + 1, offset: offset.value }]),
	queryFn: () => api.list({ limit: PAGE_SIZE + 1, offset: offset.value }),
	enabled: computed(() => !showRuns.value),
})
const rules = computed(() => data.value?.slice(0, PAGE_SIZE) ?? [])
const hasNextPage = computed(() => (data.value?.length ?? 0) > PAGE_SIZE)

const DEFAULT_RULE = `print(\`\${project.name}: \${project.summary}\`);
if project.slug == "foo" {
	[#{ reason: "Project has illegal slug" }]
} else {
	[]
}`

const ruleModal = useTemplateRef<InstanceType<typeof NewModal>>('ruleModal')
const isEditing = ref(false)
const editingId = ref<number | null>(null)
const form = reactive({ name: '', expression: '' })
const patterns = ref<FileScanPattern[]>([])
const patternErrors = computed(() => {
	const names = patterns.value.map((entry) => entry.name.trim())
	return patterns.value.map((entry) => {
		const name = entry.name.trim()
		if (!name) return 'Enter a pattern name.'
		if (names.indexOf(name) !== names.lastIndexOf(name)) {
			return 'Each pattern must have a unique name.'
		}
		if (!entry.pattern.trim()) return 'Enter a regex pattern.'
		return ''
	})
})
const ruleToDelete = ref<Rule | null>(null)
const isValid = computed(
	() =>
		form.name.trim().length > 0 &&
		Array.from(form.name).length <= 1023 &&
		form.expression.length > 0 &&
		Array.from(form.expression).length <= 65535 &&
		patternErrors.value.every((error) => !error),
)

const {
	mutate: save,
	isPending: isSaving,
	error: saveError,
	reset: resetSave,
} = useMutation({
	mutationFn: ({
		id,
		name,
		params,
	}: {
		id: number | null
		name: string
		params: Rule['params']
	}) => (id === null ? api.create({ name, params }) : api.update(id, { name, params })),
	onSuccess: async (_, { id }) => {
		isEditing.value = false
		await nextTick()
		closeEditor()
		if (id === null) offset.value = 0
		await queryClient.invalidateQueries({ queryKey })
	},
})

const {
	mutate: deleteRule,
	isPending: isDeleting,
	error: deleteError,
	reset: resetDelete,
} = useMutation({
	mutationFn: (id: number) => api.delete(id),
	onSuccess: async () => {
		ruleToDelete.value = null
		if (rules.value.length === 1 && offset.value > 0) {
			offset.value = Math.max(0, offset.value - PAGE_SIZE)
		}
		await queryClient.invalidateQueries({ queryKey })
	},
})
const isBusy = computed(() => isSaving.value || isDeleting.value)

function openCreate() {
	resetSave()
	ruleToDelete.value = null
	editingId.value = null
	form.name = ''
	form.expression = DEFAULT_RULE
	patterns.value = []
	isEditing.value = true
	ruleModal.value?.show()
}

function openEdit(rule: Rule) {
	resetSave()
	ruleToDelete.value = null
	editingId.value = rule.id
	form.name = rule.name
	form.expression = rule.params.expression
	patterns.value = Object.entries(rule.params.scan_files_for).map(([name, pattern]) => ({
		name,
		pattern,
	}))
	isEditing.value = true
	ruleModal.value?.show()
}

function closeEditor() {
	ruleModal.value?.hide()
}

function handleEditorHide() {
	isEditing.value = false
	resetSave()
}

function saveRule() {
	if (!isValid.value || isBusy.value) return
	save({
		id: editingId.value,
		name: form.name,
		params: {
			expression: form.expression,
			scan_files_for: Object.fromEntries(
				patterns.value.map((entry) => [entry.name.trim(), entry.pattern]),
			),
		},
	})
}

function confirmDelete(rule: Rule) {
	resetDelete()
	ruleToDelete.value = rule
}

function changePage(direction: number) {
	ruleToDelete.value = null
	offset.value = Math.max(0, offset.value + direction * PAGE_SIZE)
}

function formatDate(value: string) {
	return new Date(value).toLocaleString('en-US', { timeZone: 'UTC', timeZoneName: 'short' })
}

function errorMessage(value: unknown) {
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
