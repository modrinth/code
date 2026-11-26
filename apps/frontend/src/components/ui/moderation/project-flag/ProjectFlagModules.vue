<template>
	<div class="grid min-w-0 grid-cols-1 gap-4 lg:grid-cols-[minmax(0,3fr)_minmax(14rem,1fr)]">
		<RhaiExpressionEditor
			id="flag-module-script"
			v-model="form.script"
			class="universal-card !p-0"
			:disabled="busy || !editing"
			frameless
			height="max(36rem, calc(100vh - 18rem))"
			label="Module editor"
		/>
		<aside class="universal-card flex min-w-0 flex-col gap-4" aria-label="Module files">
			<div class="flex flex-wrap items-center justify-between gap-2">
				<h2 class="m-0 text-lg font-bold text-contrast">Modules</h2>
				<Button
					type="colored"
					color="brand"
					icon-only
					aria-label="Create module"
					title="Create module"
					:disabled="busy || isPending || !!error"
					@click="create"
				>
					<PlusIcon />
				</Button>
			</div>
			<template v-if="editing">
				<label for="flag-module-name" class="font-semibold text-contrast">Module name</label>
				<Input
					id="flag-module-name"
					v-model="form.name"
					:disabled="busy"
					placeholder="folder/module"
				/>
				<p class="m-0 text-sm text-secondary">{{ dirty ? 'Unsaved changes' : 'Saved' }}</p>
				<p v-if="validationError" role="alert" class="m-0 text-sm text-red">
					{{ validationError }}
				</p>
				<p v-if="saveError" role="alert" class="m-0 whitespace-pre-wrap text-red">
					Could not save module: {{ errorMessage(saveError) }}
				</p>
				<p v-if="deleteError" role="alert" class="m-0 whitespace-pre-wrap text-red">
					Could not delete module: {{ errorMessage(deleteError) }}
				</p>
				<div class="flex flex-wrap gap-2">
					<Button
						type="colored"
						color="brand"
						:disabled="busy || !dirty || !!validationError || isPending || !!error"
						@click="confirmSave"
					>
						{{
							isSaving
								? 'Saving…'
								: original && original.name !== form.name
									? 'Rename and save'
									: 'Save'
						}}
					</Button>
					<Button :disabled="busy" @click="close">Close</Button>
					<Button v-if="original" type="colored" color="red" :disabled="busy" @click="confirmDelete"
						>Delete</Button
					>
				</div>
			</template>
			<p v-else class="m-0 text-sm text-secondary">Select a module below, or create a new one.</p>
			<p v-if="isPending" role="status" class="m-0">Loading modules…</p>
			<div v-if="error" role="alert" class="flex flex-col gap-2">
				<p class="m-0 text-red">Could not load modules: {{ errorMessage(error) }}</p>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<p v-if="data && !data.length" class="m-0 text-secondary">No modules yet.</p>
			<ProjectFlagModuleTree
				:names="names"
				:selected="original?.name ?? null"
				:disabled="busy"
				@select="select"
			/>
		</aside>
	</div>
</template>

<script setup lang="ts">
import { PlusIcon } from '@modrinth/assets'
import { Button, injectModrinthClient, Input } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { onBeforeRouteLeave, onBeforeRouteUpdate } from 'vue-router'

import ProjectFlagModuleTree from './ProjectFlagModuleTree.vue'
import RhaiExpressionEditor from './rules/rule-editor-modal/expression-editor.vue'

const api = injectModrinthClient().labrinth.project_flags_internal
const queryClient = useQueryClient()
type Module = Awaited<ReturnType<typeof api.getModule>>
const queryKey = ['moderation', 'project-flag', 'modules'] as const
const { data, isPending, error, isFetching, refetch } = useQuery({
	queryKey,
	queryFn: async () => {
		const modules: Module[] = []
		for (let offset = 0; ; offset += 200) {
			const page = await api.listModules({ limit: 200, offset })
			modules.push(...page)
			if (page.length < 200) return modules
		}
	},
})
const names = computed(() => data.value?.map((module) => module.name) ?? [])
const original = ref<Module | null>(null)
const editing = ref(false)
const form = reactive({ name: '', script: '' })
const dirty = computed(
	() =>
		editing.value &&
		(original.value
			? form.name !== original.value.name || form.script !== original.value.script
			: true),
)
const validationError = computed(() => {
	if (!form.name || Array.from(form.name).length > 1024)
		return 'Enter a module name of at most 1,024 characters.'
	if (
		/[\\\0]/.test(form.name) ||
		form.name.split('/').some((part) => !part || part === '.' || part === '..')
	) {
		return 'Use canonical slash-separated segments without empty, . or .. segments, backslashes, or null characters.'
	}
	if (form.name !== original.value?.name && names.value.includes(form.name))
		return 'A module with this name already exists.'
	if (Array.from(form.script).length > 65535) return 'The script must be at most 65,535 characters.'
	return ''
})
const {
	mutate: save,
	isPending: isSaving,
	error: saveError,
	reset: resetSave,
} = useMutation({
	mutationFn: ({ name, module }: { name: string | null; module: Module }) =>
		name === null ? api.createModule(module) : api.updateModule(name, module),
	retry: false,
	onSuccess: async (_, { module }) => {
		original.value = { ...module }
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})
const {
	mutate: remove,
	isPending: isDeleting,
	error: deleteError,
	reset: resetDelete,
} = useMutation({
	mutationFn: (name: string) => api.deleteModule(name),
	retry: false,
	onSuccess: async () => {
		editing.value = false
		original.value = null
		Object.assign(form, { name: '', script: '' })
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})
const busy = computed(() => isSaving.value || isDeleting.value)
function canLeave() {
	return !busy.value && (!dirty.value || window.confirm('Discard unsaved module changes?'))
}
function resetErrors() {
	resetSave()
	resetDelete()
}
function select(name: string) {
	if (original.value?.name === name || !canLeave()) return
	const module = data.value?.find((entry) => entry.name === name)
	if (!module) return
	resetErrors()
	original.value = { ...module }
	Object.assign(form, module)
	editing.value = true
}
function create() {
	if (!canLeave()) return
	resetErrors()
	original.value = null
	Object.assign(form, { name: '', script: '' })
	editing.value = true
}
function close() {
	if (!canLeave()) return
	editing.value = false
	original.value = null
	Object.assign(form, { name: '', script: '' })
	resetErrors()
}
function confirmSave() {
	if (busy.value || !dirty.value || validationError.value) return
	if (!original.value) {
		save({ name: null, module: { ...form } })
		return
	}
	const action =
		original.value.name !== form.name
			? `Rename “${original.value.name}” to “${form.name}” and save changes?`
			: `Save changes to “${form.name}”?`
	if (window.confirm(action)) save({ name: original.value?.name ?? null, module: { ...form } })
}
function confirmDelete() {
	if (busy.value || !original.value) return
	if (
		window.confirm(
			`Delete module “${original.value.name}”? This cannot be undone.${dirty.value ? ' Unsaved changes will also be discarded.' : ''}`,
		)
	)
		remove(original.value.name)
}
function beforeUnload(event: BeforeUnloadEvent) {
	if (!dirty.value && !busy.value) return
	event.preventDefault()
	event.returnValue = ''
}
onMounted(() => window.addEventListener('beforeunload', beforeUnload))
onBeforeUnmount(() => window.removeEventListener('beforeunload', beforeUnload))
onBeforeRouteLeave(() => canLeave())
onBeforeRouteUpdate(() => canLeave())
function errorMessage(value: unknown) {
	if (value && typeof value === 'object' && 'data' in value) {
		const data = value.data
		if (
			data &&
			typeof data === 'object' &&
			'description' in data &&
			typeof data.description === 'string'
		) {
			return data.description
		}
	}
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
