<template>
	<span ref="anchor" hidden />
	<Teleport v-if="target" :to="target">
		<div
			v-if="selection"
			ref="dialog"
			role="dialog"
			aria-modal="true"
			aria-label="Project flag rule editor"
			tabindex="-1"
			class="project-flag-editor-dialog fixed inset-0 z-[9000] h-dvh overflow-auto overscroll-none bg-surface-2 text-primary outline-none lg:[@media_(min-height:36rem)]:overflow-hidden"
		>
			<RuleEditor
				v-if="selection === 'new' || ruleReady || selection === createdRuleId"
				:key="editorSession"
				:initial-rule="rule"
				:initial-project-id="sampleProjectId"
				:back-label="backLabel"
				@close="closeEditor"
				@created="showSavedRule"
			/>
			<div v-else class="flex flex-col items-start gap-4 p-4">
				<Button type="quiet" @click="closeEditor">← {{ backLabel }}</Button>
				<p
					v-if="isFetching || !error"
					role="status"
					class="m-0 flex items-center gap-2 text-secondary"
				>
					<LoaderCircleIcon class="size-4 animate-spin" /> Loading rule…
				</p>
				<template v-else>
					<p role="alert" class="m-0 text-red">
						{{ notFound ? 'This rule could not be found.' : 'Could not load the rule.' }}
					</p>
					<Button v-if="!notFound" @click="loadRule">Try again</Button>
				</template>
			</div>
		</div>
	</Teleport>
</template>

<script setup lang="ts">
import { LoaderCircleIcon } from '@modrinth/assets'
import { Button, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'

import { projectFlagRuleQueryOptions } from '~/composables/queries/project-flag'

import RuleEditor from './index.vue'

withDefaults(defineProps<{ backLabel?: string }>(), { backLabel: 'Rules' })

const route = useRoute()
const router = useRouter()
const anchor = useTemplateRef<HTMLElement>('anchor')
const dialog = useTemplateRef<HTMLElement>('dialog')
const target = shallowRef<HTMLElement>()
const selection = computed(() => {
	const id = route.query.rule
	return typeof id === 'string' && /^[A-Za-z0-9]+$/.test(id) ? id : undefined
})
const ruleId = computed(() => (selection.value === 'new' ? undefined : selection.value))
const sampleProjectId = computed(() => {
	const id = route.query.ruleProject ?? route.query.project
	return typeof id === 'string' && /^[A-Za-z0-9]+$/.test(id) ? id : undefined
})
const createdRuleId = ref<string>()
const editorSession = ref(0)
const ruleReady = ref(false)
let loadAttempt = 0
const client = injectModrinthClient()
const {
	data: rule,
	isFetching,
	error,
	refetch,
} = useQuery(
	computed(() => ({
		...projectFlagRuleQueryOptions(client, ruleId.value ?? ''),
		enabled: false,
	})),
)
const notFound = computed(() => (error.value as { statusCode?: number } | null)?.statusCode === 404)

useHead(() =>
	selection.value
		? {
				title:
					(ruleReady.value || selection.value === createdRuleId.value) && rule.value
						? `${rule.value.name} - Project flag rules`
						: 'Project flag rule editor - Modrinth',
			}
		: {},
)

onMounted(() => {
	target.value = anchor.value?.ownerDocument.body
	watch(
		[selection, sampleProjectId],
		([current, project], [previous, previousProject]) => {
			if (
				previous === 'new' &&
				current &&
				current === createdRuleId.value &&
				project === previousProject
			) {
				return
			}

			createdRuleId.value = undefined
			editorSession.value++
			void loadRule()
		},
		{ immediate: true },
	)
})

watch(dialog, (element, _, onCleanup) => {
	if (!element) return
	const ownerDocument = element.ownerDocument
	const previousFocus = ownerDocument.activeElement as HTMLElement | null
	const background = Array.from(ownerDocument.body.children).filter(
		(child) => child !== element && !child.hasAttribute('inert'),
	)
	for (const child of background) child.setAttribute('inert', '')
	element.focus({ preventScroll: true })
	ownerDocument.addEventListener('keydown', onDialogKeydown)
	onCleanup(() => {
		ownerDocument.removeEventListener('keydown', onDialogKeydown)
		for (const child of background) child.removeAttribute('inert')
		if (previousFocus?.isConnected) previousFocus.focus?.({ preventScroll: true })
	})
})

function onDialogKeydown(event: KeyboardEvent) {
	if (event.key !== 'Escape' && event.key !== 'Tab') return
	const element = dialog.value
	if (
		!element ||
		event.defaultPrevented ||
		event.isComposing ||
		element.querySelector('dialog[open]')
	) {
		return
	}
	const { ownerDocument } = element
	const target = event.target as Node | null
	if (
		!element.contains(target) &&
		target !== ownerDocument.body &&
		target !== ownerDocument.documentElement &&
		target !== ownerDocument
	) {
		return
	}
	if (event.key === 'Escape') {
		if (event.repeat) return
		event.preventDefault()
		void closeEditor()
		return
	}

	const active = ownerDocument.activeElement
	const tabbable = Array.from(
		element.querySelectorAll<HTMLElement>('a[href], button, input, select, textarea, [tabindex]'),
	).filter(
		(entry) =>
			entry.tabIndex >= 0 &&
			!entry.matches(':disabled') &&
			!entry.closest('[inert]') &&
			entry.getClientRects().length > 0,
	)
	const edge = event.shiftKey ? tabbable.at(-1) : tabbable[0]
	if (
		active === element ||
		!element.contains(active) ||
		active === (event.shiftKey ? tabbable[0] : tabbable.at(-1)) ||
		!edge
	) {
		event.preventDefault()
		const next = edge ?? element
		next.focus()
	}
}

async function loadRule() {
	const attempt = ++loadAttempt
	ruleReady.value = false
	if (!ruleId.value) return
	const result = await refetch()
	if (attempt === loadAttempt) ruleReady.value = result.isSuccess
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

function showSavedRule(id: string) {
	createdRuleId.value = id
	return router.replace({ query: { ...route.query, rule: id }, hash: route.hash })
}
</script>

<style scoped>
:global(html:has(.project-flag-editor-dialog)) {
	overflow: hidden;
	overscroll-behavior: none;
}
</style>
