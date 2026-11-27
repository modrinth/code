<template>
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
	>
		<div
			v-if="editor"
			class="h-dvh overflow-auto overscroll-none lg:[@media_(min-height:36rem)]:overflow-hidden"
		>
			<RuleEditor
				:key="editor.key"
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
import { injectModrinthClient, injectNotificationManager, NewModal } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'

import {
	projectFlagInputSchemaQueryOptions,
	projectFlagModulesQueryOptions,
	projectFlagRuleQueryOptions,
} from '~/composables/queries/project-flag'

import RuleEditor from './index.vue'
import { loadRhaiEditor, type RhaiEditorRuntime } from './rhai-editor-runtime'

withDefaults(defineProps<{ backLabel?: string }>(), { backLabel: 'Rules' })

const route = useRoute()
const router = useRouter()
const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
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
	rule?: ProjectFlagRule
	project?: string
	rhaiEditor: RhaiEditorRuntime
}>()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()

useHead(() =>
	editor.value
		? {
				title: editor.value.rule
					? `${editor.value.rule.name} - Project flag rules`
					: 'Project flag rule editor - Modrinth',
			}
		: {},
)

onMounted(() => {
	watch(
		[selection, sampleProjectId],
		async ([current, project], [previous, previousProject], onCleanup) => {
			if (
				previous === 'new' &&
				current &&
				current === editor.value?.rule?.id &&
				project === previousProject
			) {
				return
			}

			if (editorOpen) {
				editorOpen = false
				modal.value?.hide()
			}
			if (!current) return

			let cancelled = false
			onCleanup(() => {
				cancelled = true
			})
			try {
				const [rule, rhaiEditor] = await Promise.all([
					current === 'new'
						? undefined
						: queryClient.fetchQuery(projectFlagRuleQueryOptions(client, current)),
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
		{ immediate: true },
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
	if (selection.value === 'new') {
		return router.replace({ query: { ...route.query, rule: rule.id }, hash: route.hash })
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
