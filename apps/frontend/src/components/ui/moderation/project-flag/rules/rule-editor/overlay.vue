<template>
	<NewModal
		ref="createModal"
		header="Create rule"
		width="100%"
		max-width="28rem"
		:initial-focus="() => createForm?.querySelector('input') ?? null"
		:disable-close="isCreatingRule && selection === 'new'"
		:on-hide="onCreateModalHide"
	>
		<form
			:id="createFormId"
			ref="createForm"
			class="flex flex-col gap-2 pb-2"
			@submit.prevent="createRule"
		>
			<label :for="ruleNameId" class="text-sm font-medium text-contrast">Rule name</label>
			<Input
				:id="ruleNameId"
				v-model="ruleName"
				placeholder="Enter a rule name"
				autocomplete="off"
				:disabled="isCreatingRule"
				:error="Boolean(ruleName && ruleNameError)"
				:aria-describedby="ruleName && ruleNameError ? ruleNameErrorId : undefined"
			/>
			<p v-if="ruleName && ruleNameError" :id="ruleNameErrorId" class="m-0 text-sm text-red">
				{{ ruleNameError }}
			</p>
			<p v-if="createError" role="alert" class="m-0 break-words text-sm text-red">
				Could not create the rule. {{ createError.message }}
			</p>
		</form>
		<template #actions>
			<div class="flex justify-end gap-2">
				<Button :disabled="isCreatingRule" @click="createModal?.hide()">Cancel</Button>
				<Button
					native-type="submit"
					:form="createFormId"
					type="colored"
					color="brand"
					:disabled="Boolean(ruleNameError)"
					:loading="isCreatingRule"
				>
					Create rule
				</Button>
			</div>
		</template>
	</NewModal>
	<NewModal
		ref="modal"
		header="Project flag rule editor"
		:close-on-click-outside="false"
		:animate-resize="false"
		:before-hide="() => !editorOpen"
		:on-escape="closeEditor"
		:on-after-hide="clearClosedEditor"
		hide-header
		no-padding
		noblur
		class="project-flag-editor-dialog !inset-0 !h-dvh !max-h-none !w-full !max-w-none !rounded-none !border-0 !bg-surface-2 !shadow-none ![scale:1]"
		@keydown.capture="onSaveKeydown"
	>
		<div
			v-if="editor"
			class="h-dvh overflow-auto overscroll-none lg:[@media_(min-height:36rem)]:overflow-hidden"
		>
			<RuleEditor
				:key="editor.key"
				ref="ruleEditor"
				:initial-rule="editor.rule"
				:initial-project-id="editor.project"
				:rhai-editor="editor.rhaiEditor"
				:back-label="backLabel"
				@close="closeEditor"
				@saved="onRuleSaved"
			/>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import type { ProjectFlagRule } from '@modrinth/api-client'
import { Button, Input, injectModrinthClient, injectNotificationManager, NewModal } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import {
	projectFlagInputSchemaQueryOptions,
	projectFlagModulesQueryOptions,
	projectFlagRuleQueryOptions,
} from '~/composables/queries/project-flag'

import { getRuleNameError, useSaveRule } from '../use-rules'
import RuleEditor from './index.vue'
import { loadRhaiEditor, type RhaiEditorRuntime } from './rhai-editor-runtime'

withDefaults(defineProps<{ backLabel?: string }>(), { backLabel: 'Rules' })

const route = useRoute()
const router = useRouter()
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const ruleEditor = useTemplateRef<InstanceType<typeof RuleEditor>>('ruleEditor')
const createModal = useTemplateRef<InstanceType<typeof NewModal>>('createModal')
const createForm = useTemplateRef<HTMLFormElement>('createForm')
const createFormId = useId()
const ruleNameId = `${createFormId}-name`
const ruleNameErrorId = `${ruleNameId}-error`
const ruleName = ref('')
const ruleNameError = computed(() => getRuleNameError(ruleName.value.trim()))
const {
	mutateAsync: create,
	isPending: isCreatingRule,
	error: createError,
	reset: resetCreate,
} = useSaveRule()
let createModalOpen = false
let editorOpen = false
let nextEditorKey = 0
const selection = computed(() => {
	const id = route.query.rule
	return typeof id === 'string' && /^[A-Za-z0-9]+$/.test(id) ? id : undefined
})
const sampleProjectId = computed(() => {
	const id = route.query.ruleProject ?? route.query.project
	return typeof id === 'string' && /^[A-Za-z0-9]+$/.test(id) ? id : undefined
})
const editor = shallowRef<{
	key: number
	rule: ProjectFlagRule
	project?: string
	rhaiEditor: RhaiEditorRuntime
}>()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

useHead(() =>
	editor.value
		? {
					title: `${editor.value.rule.name} - Project flag rules`,
			}
		: {},
)

onMounted(() => {
	watch(
		[selection, sampleProjectId],
		async ([current, project], [previous], onCleanup) => {
			if (editorOpen) {
				editorOpen = false
				modal.value?.hide()
			}
			if (current === 'new') {
				if (previous === 'new') return
				if (!isCreatingRule.value) {
					ruleName.value = ''
					resetCreate()
				}
				createModalOpen = true
				createModal.value?.show()
				return
			}
			if (createModalOpen) createModal.value?.hide()
			if (!current) return

			let cancelled = false
			onCleanup(() => {
				cancelled = true
			})
			try {
				const [rule, rhaiEditor] = await Promise.all([
					queryClient.fetchQuery(projectFlagRuleQueryOptions(client, current)),
					loadRhaiEditor(),
					queryClient.prefetchQuery(projectFlagInputSchemaQueryOptions(client, project)),
					queryClient.prefetchQuery(projectFlagModulesQueryOptions(client)),
				])
				if (cancelled) return
				editor.value = { key: ++nextEditorKey, rule, project, rhaiEditor }
				editorOpen = true
				modal.value?.show()
			} catch (error) {
				if (cancelled) return
				addNotification({
					type: 'error',
					title: 'Could not open the rule editor',
					text: error instanceof Error ? error.message : String(error),
				})
				void closeEditor()
			}
		},
		{ immediate: true, flush: 'post' },
	)
})

function clearClosedEditor() {
	if (!editorOpen) editor.value = undefined
}

function closeEditor() {
	if (!selection.value) return
	const { rule: _rule, ruleProject: _ruleProject, ...query } = route.query
	const destination = { path: route.path, query, hash: route.hash }
	if (router.options.history.state.back === router.resolve(destination).fullPath) {
		router.back()
		return
	}
	return router.replace(destination)
}

function onRuleSaved(rule: ProjectFlagRule) {
	if (editor.value) editor.value = { ...editor.value, rule }
}

function onSaveKeydown(event: KeyboardEvent) {
	if (
		!editorOpen ||
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
	if (!event.repeat) void ruleEditor.value?.saveRule()
}

function onCreateModalHide() {
	createModalOpen = false
	if (selection.value === 'new') void closeEditor()
}

async function createRule() {
	if (selection.value !== 'new' || ruleNameError.value || isCreatingRule.value) return
	const path = router.currentRoute.value.path
	const rule = await create({
		id: undefined,
		body: { name: ruleName.value.trim(), params: { expression: '[]' } },
	}).catch(() => undefined)
	if (!rule || !createModal.value) return

	ruleName.value = ''
	const current = router.currentRoute.value
	if (current.path === path && current.query.rule === 'new') {
		await router.replace({ path, query: { ...current.query, rule: rule.id }, hash: current.hash })
	}
}
</script>

<style scoped>
:global(.modal-container:has(> .project-flag-editor-dialog)) {
	--modal-mouse-x-offset: 0px;
	--modal-mouse-y-offset: 0px;
}

:global(html:has(.project-flag-editor-dialog)) {
	overflow: hidden;
	overscroll-behavior: none;
}
</style>
