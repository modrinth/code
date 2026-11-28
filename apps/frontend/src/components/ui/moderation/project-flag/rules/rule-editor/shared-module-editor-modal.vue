<template>
	<NewModal
		ref="editorModal"
		:header="original ? 'Edit shared module' : 'New shared module'"
		width="52rem"
		scrollable
		:close-on-click-outside="!isDirty"
		:disable-close="isBusy"
		:before-hide="allowEditorClose"
		:on-hide="() => (isOpen = false)"
		@keydown.capture="onSaveKeydown"
	>
		<div class="flex min-w-0 flex-col gap-3">
			<div class="flex flex-col gap-2">
				<label :for="nameId" class="text-sm font-medium text-contrast">Module name</label>
				<Input
					:id="nameId"
					v-model="draft.name"
					:disabled="isBusy"
					:error="Boolean(isDirty && validationError)"
					placeholder="folder/module"
					autocomplete="off"
					:spellcheck="false"
				/>
				<p v-if="isRenaming" class="m-0 break-words text-sm text-secondary">
					Rules and modules that import “{{ original?.name }}” may fail in staging until their
					imports are updated.
				</p>
			</div>
			<RhaiExpressionEditor
				:id="scriptId"
				:runtime="runtime"
				v-model="draft.script"
				class="h-[clamp(12rem,48dvh,32rem)]"
				:disabled="isBusy"
				:word-wrap="wordWrap"
				label="Module script"
				variant="source"
			/>
			<p v-if="isDirty && validationError" role="alert" class="m-0 text-sm text-red">
				{{ validationError }}
			</p>
			<p v-if="saveError" role="alert" class="m-0 break-words text-sm text-red">
				Could not save module. {{ saveError.message }}
			</p>
		</div>
		<template #actions>
			<div class="flex flex-wrap items-center gap-2">
				<Button
					v-if="original"
					type="quiet"
					class="enabled:hover:text-red"
					:disabled="isBusy"
					@click="confirmDelete"
				>
					<TrashIcon aria-hidden="true" />
					Delete module
				</Button>
				<div class="ml-auto flex flex-wrap justify-end gap-2">
					<Button :disabled="isBusy" @click="editorModal?.hide()">Cancel</Button>
					<Button
						type="colored"
						color="brand"
						aria-keyshortcuts="Meta+S Control+S"
						:disabled="!canSave"
						:loading="isSaving"
						@click="saveModule"
					>
						{{ original ? 'Save changes' : 'Create module' }}
					</Button>
				</div>
			</div>
		</template>
	</NewModal>
	<ConfirmModal
		ref="discardModal"
		title="Discard changes?"
		description="Discard unsaved changes to this module?"
		:markdown="false"
		proceed-label="Discard"
		@proceed="discardChanges"
	/>
	<NewModal
		ref="deleteModal"
		header="Delete shared module?"
		max-width="32rem"
		fade="danger"
		:disable-close="isDeleting"
	>
		<div class="flex flex-col gap-3 break-words">
			<p class="m-0">Permanently delete “{{ original?.name }}” from staging?</p>
			<p class="m-0 text-secondary">
				Rules and modules that import it may fail in staging until their imports are updated.
			</p>
			<p v-if="deleteError" role="alert" class="m-0 text-sm text-red">
				Could not delete module. {{ deleteError.message }}
			</p>
		</div>
		<template #actions>
			<div class="flex flex-wrap justify-end gap-2">
				<Button :disabled="isDeleting" @click="deleteModal?.hide()">Cancel</Button>
				<Button
					type="colored"
					color="red"
					:disabled="isDeleting"
					:loading="isDeleting"
					@click="deleteModule"
				>
					Delete module
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { ProjectFlagModule } from '@modrinth/api-client'
import { TrashIcon } from '@modrinth/assets'
import {
	Button,
	ConfirmModal,
	injectModrinthClient,
	injectNotificationManager,
	Input,
	NewModal,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref, shallowRef, useId, useTemplateRef, watch } from 'vue'

import { projectFlagModulesQueryOptions } from '~/composables/queries/project-flag'

import RhaiExpressionEditor from './expression-editor.vue'
import type { RhaiEditorRuntime } from './rhai-editor-runtime'

const props = defineProps<{
	modules?: ProjectFlagModule[]
	runtime: RhaiEditorRuntime
	wordWrap: boolean
}>()

const nameId = useId()
const scriptId = useId()
const editorModal = useTemplateRef<InstanceType<typeof NewModal>>('editorModal')
const deleteModal = useTemplateRef<InstanceType<typeof NewModal>>('deleteModal')
const discardModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('discardModal')
const isOpen = ref(false)
let discardConfirmed = false
const original = shallowRef<ProjectFlagModule>()
const draft = ref<ProjectFlagModule>({ name: '', script: '' })
const isDirty = computed(
	() =>
		isOpen.value &&
		(draft.value.name !== (original.value?.name ?? '') ||
			draft.value.script !== (original.value?.script ?? '')),
)
const isRenaming = computed(() => original.value && draft.value.name !== original.value.name)
const validationError = computed(() => {
	const { name, script } = draft.value
	if (!name || Array.from(name).length > 1024)
		return 'Enter a module name of at most 1,024 characters.'
	if (/[\\\0]/.test(name) || name.split('/').some((part) => !part || part === '.' || part === '..'))
		return 'Enter a valid slash-separated module name.'
	if (Array.from(script).length > 65535 || script.includes('\0'))
		return 'Enter a script of at most 65,535 characters without null characters.'
	if (name !== original.value?.name && props.modules?.some((module) => module.name === name))
		return 'A module with this name already exists.'
})

const client = injectModrinthClient()
const api = client.labrinth.project_flags_internal
const { addNotification } = injectNotificationManager()
const queryClient = useQueryClient()
const { queryKey } = projectFlagModulesQueryOptions(client)
const {
	mutate: save,
	isPending: isSaving,
	isSuccess: saved,
	error: saveError,
	reset: resetSave,
} = useMutation({
	mutationFn: ({ oldName, module }: { oldName?: string; module: ProjectFlagModule }) =>
		oldName ? api.updateModule(oldName, module) : api.createModule(module),
	retry: false,
	onSuccess: async (module, { oldName }) => {
		await queryClient.cancelQueries({ queryKey })
		queryClient.setQueryData<ProjectFlagModule[]>(
			queryKey,
			(modules) =>
				modules &&
				[...modules.filter(({ name }) => name !== oldName && name !== module.name), module].sort(
					(a, b) => a.name.localeCompare(b.name),
				),
		)
		void queryClient.invalidateQueries({ queryKey })
		addNotification({
			type: 'success',
			title: oldName ? 'Shared module saved' : 'Shared module created',
		})
	},
})
const {
	mutate: remove,
	isPending: isDeleting,
	isSuccess: deleted,
	error: deleteError,
	reset: resetDelete,
} = useMutation({
	mutationFn: (name: string) => api.deleteModule(name),
	retry: false,
	onSuccess: async (_, name) => {
		await queryClient.cancelQueries({ queryKey })
		queryClient.setQueryData<ProjectFlagModule[]>(queryKey, (modules) =>
			modules?.filter((module) => module.name !== name),
		)
		void queryClient.invalidateQueries({ queryKey })
		addNotification({ type: 'success', title: 'Shared module deleted' })
	},
})
const isBusy = computed(() => isSaving.value || isDeleting.value)
const canSave = computed(() => isDirty.value && !isBusy.value && !validationError.value)

watch(
	saved,
	(success) => {
		if (success) editorModal.value?.hide()
	},
	{ flush: 'post' },
)
watch(
	deleted,
	(success) => {
		if (!success) return
		deleteModal.value?.hide()
		editorModal.value?.hide()
	},
	{ flush: 'post' },
)

function show(module?: ProjectFlagModule) {
	if (isOpen.value || !editorModal.value) return
	resetSave()
	resetDelete()
	original.value = module
	draft.value = module ? { ...module } : { name: '', script: '' }
	isOpen.value = true
	editorModal.value.show()
}

function allowEditorClose() {
	if (!isDirty.value || discardConfirmed || saved.value || deleted.value) return true
	discardModal.value?.show()
	return false
}

function discardChanges() {
	discardConfirmed = true
	editorModal.value?.hide()
	discardConfirmed = false
}

function saveModule() {
	if (canSave.value) save({ oldName: original.value?.name, module: { ...draft.value } })
}

function onSaveKeydown(event: KeyboardEvent) {
	if (
		event.key.toLowerCase() !== 's' ||
		!(event.metaKey || event.ctrlKey) ||
		event.altKey ||
		event.shiftKey ||
		event.isComposing
	) {
		return
	}
	event.preventDefault()
	event.stopPropagation()
	if (!event.repeat) saveModule()
}

function confirmDelete() {
	if (!original.value || isBusy.value) return
	resetDelete()
	deleteModal.value?.show()
}

function deleteModule() {
	if (original.value && !isBusy.value) remove(original.value.name)
}

defineExpose({ show, isBusy, isDirty })
</script>
