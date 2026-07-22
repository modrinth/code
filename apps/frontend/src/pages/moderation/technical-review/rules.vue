<template>
	<NewModal ref="ruleModal" :header="modalTitle" :on-hide="handleRuleModalHide">
		<form class="flex w-[48rem] max-w-full flex-col gap-3" @submit.prevent="saveRule">
			<label class="font-semibold text-contrast" for="rule-name">Name</label>
			<StyledInput
				id="rule-name"
				v-model="form.name"
				type="text"
				maxlength="256"
				placeholder="Known-safe obfuscated bootstrap"
			/>

			<label class="font-semibold text-contrast" for="rule-expression">CEL expression</label>
			<div
				class="relative overflow-hidden rounded-[20px] border border-solid border-surface-4 shadow-sm"
			>
				<component
					:is="editorComponent"
					v-if="editorComponent"
					id="rule-expression"
					:value="form.rule"
					lang="javascript"
					theme="modrinth"
					:print-margin="false"
					:options="RULE_EDITOR_OPTIONS"
					:style="{ height: '16rem', fontSize: '0.875rem' }"
					class="ace-modrinth rounded-[20px]"
					@init="onRuleEditorInit"
					@update:value="handleRuleInput"
				/>
				<div v-else class="flex h-64 items-center justify-center bg-bg-raised">
					<LoaderCircleIcon class="size-8 animate-spin text-secondary" />
				</div>
			</div>
			<p class="m-0 text-sm text-secondary">
				Return <code>null</code> when the rule does not match, or a map containing
				<code>severity</code> and/or <code>hidden</code> when it does.
			</p>

			<section class="mt-2 flex flex-col gap-3">
				<div class="flex items-center justify-between gap-3">
					<div>
						<h3 class="m-0 text-base font-bold text-contrast">Test traces</h3>
						<p class="m-0 text-sm text-secondary">
							These results are evaluated from the current expression.
						</p>
					</div>
					<LoaderCircleIcon v-if="isTestingRule" class="size-5 animate-spin text-secondary" />
				</div>

				<div
					v-if="ruleTestError"
					class="border-red/40 rounded-lg border bg-highlight-red p-3 text-sm text-red"
				>
					{{ ruleTestError }}
				</div>

				<div
					v-for="example in previewExamples"
					:key="example.original.key"
					class="grid items-stretch gap-3 md:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]"
				>
					<article class="universal-card flex min-w-0 flex-col gap-3">
						<p class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">Original</p>
						<div class="flex flex-wrap items-center gap-2">
							<span
								class="rounded-full border px-2 py-0.5 text-xs font-semibold capitalize"
								:class="getSeverityBadgeColor(example.original.severity)"
							>
								{{ example.original.severity }}
							</span>
							<strong class="break-all text-contrast">{{ example.original.issue_type }}</strong>
						</div>
						<dl class="m-0 grid grid-cols-[auto_minmax(0,1fr)] gap-x-3 gap-y-1 text-sm">
							<dt class="text-secondary">Key</dt>
							<dd class="m-0 break-all font-mono text-contrast">{{ example.original.key }}</dd>
							<dt class="text-secondary">File</dt>
							<dd class="m-0 break-all font-mono text-contrast">
								{{ example.original.file_path }}
							</dd>
						</dl>
					</article>

					<div
						class="flex items-center justify-center text-2xl font-bold text-secondary max-md:rotate-90"
						aria-hidden="true"
					>
						→
					</div>

					<article class="universal-card flex min-w-0 flex-col gap-3">
						<p class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">
							New state
						</p>
						<div v-if="example.effect?.hidden" class="flex items-center gap-2 text-secondary">
							<EyeOffIcon class="size-5" />
							<strong class="text-contrast">Hidden from reports</strong>
						</div>
						<template v-else>
							<div class="flex flex-wrap items-center gap-2">
								<span
									class="rounded-full border px-2 py-0.5 text-xs font-semibold capitalize"
									:class="getSeverityBadgeColor(example.effectiveSeverity)"
								>
									{{ example.effectiveSeverity }}
								</span>
								<strong class="break-all text-contrast">{{ example.original.issue_type }}</strong>
							</div>
							<p class="m-0 text-sm text-secondary">{{ example.summary }}</p>
						</template>
					</article>
				</div>
			</section>

			<div class="flex justify-end gap-2">
				<ButtonStyled>
					<button type="button" @click="closeRuleModal">Cancel</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="submit" :disabled="isSaving">
						{{ isSaving ? 'Saving...' : 'Save rule' }}
					</button>
				</ButtonStyled>
			</div>
		</form>
	</NewModal>

	<ConfirmModal
		ref="deleteModal"
		:title="`Delete ${ruleToDelete?.name ?? 'rule'}?`"
		description="The rule will stop being included the next time the rules are scanned. Existing effects remain active until then."
		:markdown="false"
		proceed-label="Delete rule"
		@proceed="deleteRule"
	/>

	<div class="flex flex-col gap-6">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div class="flex items-center gap-3">
				<ButtonStyled circular type="transparent">
					<NuxtLink to="/moderation/technical-review" aria-label="Back to tech review queue">
						<ArrowLeftIcon />
					</NuxtLink>
				</ButtonStyled>
				<div>
					<h1 class="m-0 text-2xl font-bold text-contrast">Delphi rules</h1>
					<p class="m-0 text-secondary">Transform or hide Delphi issue traces.</p>
				</div>
			</div>

			<ButtonStyled color="brand">
				<button type="button" @click="openCreateModal">
					<PlusIcon />
					Create rule
				</button>
			</ButtonStyled>
		</div>

		<details class="universal-card text-sm">
			<summary class="cursor-pointer font-semibold text-contrast">CEL contract and input</summary>
			<div class="mt-3 flex flex-col gap-2 text-secondary">
				<p class="m-0">
					Effects are maps such as
					<code>{ "severity": "low", "hidden": false }</code>. Severity can be <code>low</code>,
					<code>medium</code>, <code>high</code>, or <code>severe</code>.
				</p>
				<p class="m-0">
					The <code>input</code> object contains <code>schema_version</code>,
					<code>trace</code> (<code>key</code>, <code>issue_type</code>, <code>severity</code>,
					<code>jar</code>, <code>file_path</code>, <code>data</code>),
					<code>scan.delphi_version</code>, <code>artifact</code> (<code>size</code>,
					<code>hashes</code>), and stable IDs under <code>scope</code> (<code>project_id</code>,
					<code>version_id</code>, <code>file_id</code>).
				</p>
			</div>
		</details>

		<div v-if="isLoading" class="universal-card flex h-32 items-center justify-center">
			<LoaderCircleIcon class="size-8 animate-spin text-secondary" />
		</div>
		<div v-else-if="loadFailed" class="universal-card flex flex-col items-center gap-3 py-8">
			<p class="m-0 text-secondary">Failed to load Delphi rules.</p>
			<ButtonStyled>
				<button type="button" @click="loadRules">Try again</button>
			</ButtonStyled>
		</div>
		<EmptyState
			v-else-if="rules.length === 0"
			type="no-search-result"
			heading="No Delphi rules"
			description="Create a rule to transform matching issue traces."
		/>
		<div v-else class="flex flex-col gap-3">
			<article v-for="rule in rules" :key="rule.id" class="universal-card flex flex-col gap-3">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="m-0 text-lg font-bold text-contrast">{{ rule.name }}</h2>
						<p class="m-0 text-sm text-secondary">Last applied in revision {{ rule.revision }}</p>
					</div>
					<div class="flex gap-2">
						<ButtonStyled>
							<button type="button" @click="openEditModal(rule)">
								<EditIcon />
								Edit
							</button>
						</ButtonStyled>
						<ButtonStyled color="red">
							<button type="button" @click="openDeleteModal(rule)">
								<TrashIcon />
								Delete
							</button>
						</ButtonStyled>
					</div>
				</div>
				<pre
					class="m-0 overflow-x-auto rounded-lg bg-bg-raised p-3 text-sm"
				><code>{{ rule.rule }}</code></pre>
			</article>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ArrowLeftIcon,
	EditIcon,
	EyeOffIcon,
	LoaderCircleIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	StyledInput,
} from '@modrinth/ui'
import { useDebounceFn } from '@vueuse/core'
import type { Ace } from 'ace-builds'
import type { Component } from 'vue'

const DEFAULT_RULE = `input.trace.issue_type == "OBFUSCATED_NAMES"
	? {"severity": "low", "hidden": false}
	: null`
const RULE_EDITOR_OPTIONS: Partial<Ace.EditorOptions> = {
	useWorker: false,
	tabSize: 2,
	useSoftTabs: true,
}

const TEST_TRACES: Labrinth.TechReview.Internal.TestDelphiRuleTrace[] = [
	{
		key: 'known-safe:obfuscated-bootstrap',
		issue_type: 'OBFUSCATED_NAMES',
		severity: 'high',
		jar: 'META-INF/jars/embedded.jar',
		file_path: 'com/example/Bootstrap.class',
		data: {
			confidence: 0.97,
			symbol_count: 42,
		},
	},
	{
		key: 'network/known-telemetry-host',
		issue_type: 'SUSPICIOUS_NETWORK_ACCESS',
		severity: 'medium',
		jar: null,
		file_path: 'com/example/Telemetry.class',
		data: {
			host: 'telemetry.example.com',
		},
	},
]

useHead({ title: 'Delphi rules - Modrinth' })

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const ruleModal = useTemplateRef<InstanceType<typeof NewModal>>('ruleModal')
const deleteModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('deleteModal')
const editorComponent = shallowRef<Component | null>(null)
const ruleEditorInstance = shallowRef<Ace.Editor | null>(null)

const rules = ref<Labrinth.TechReview.Internal.DelphiRule[]>([])
const isLoading = ref(true)
const isSaving = ref(false)
const isTestingRule = ref(false)
const isRuleModalOpen = ref(false)
const loadFailed = ref(false)
const editingRuleId = ref<number | null>(null)
const ruleToDelete = ref<Labrinth.TechReview.Internal.DelphiRule | null>(null)
const ruleTestEffects = ref<Array<Labrinth.TechReview.Internal.DelphiRuleEffect | null>>([])
const ruleTestError = ref<string | null>(null)
const form = reactive({
	name: '',
	rule: DEFAULT_RULE,
})
let ruleTestRequestId = 0

onMounted(async () => {
	const [{ VAceEditor }] = await Promise.all([
		import('vue3-ace-editor'),
		import('@modrinth/ui/src/utils/ace-theme'),
	])
	editorComponent.value = VAceEditor
})

const modalTitle = computed(() => (editingRuleId.value === null ? 'Create rule' : 'Edit rule'))
const previewExamples = computed(() =>
	TEST_TRACES.map((original, index) => {
		const effect = ruleTestEffects.value[index] ?? null
		const effectiveSeverity = effect?.severity ?? original.severity
		let summary: string

		if (isTestingRule.value) {
			summary = 'Evaluating the current expression...'
		} else if (ruleTestError.value) {
			summary = 'Preview unavailable.'
		} else if (!effect) {
			summary = 'Rule does not match; no change.'
		} else if (effect.severity && effect.severity !== original.severity) {
			summary = `Severity changed from ${original.severity} to ${effect.severity}.`
		} else {
			summary = 'Rule matched; no visible change.'
		}

		return {
			original,
			effect,
			effectiveSeverity,
			summary,
		}
	}),
)

function getSeverityBadgeColor(severity: Labrinth.TechReview.Internal.DelphiSeverity): string {
	switch (severity) {
		case 'severe':
			return 'border-red/60 bg-highlight-red text-red'
		case 'high':
			return 'border-orange/60 bg-highlight-orange text-orange'
		case 'medium':
			return 'border-green/60 bg-highlight-green text-green'
		case 'low':
		default:
			return 'border-blue/60 bg-highlight-blue text-blue'
	}
}

function onRuleEditorInit(editor: Ace.Editor) {
	ruleEditorInstance.value = editor
	editor.session.setUseWrapMode(true)
}

function handleRuleInput(rule: string) {
	form.rule = rule
	queueRuleTest()
}

async function testRule() {
	if (!isRuleModalOpen.value) return

	const requestId = ++ruleTestRequestId
	const rule = form.rule.trim()
	ruleTestEffects.value = []
	ruleTestError.value = null

	if (!rule) {
		isTestingRule.value = false
		ruleTestError.value = 'Enter a CEL expression to test it.'
		return
	}

	isTestingRule.value = true
	try {
		const response = await client.labrinth.tech_review_internal.testRule({
			rule,
			traces: TEST_TRACES,
		})
		if (requestId !== ruleTestRequestId) return

		ruleTestEffects.value = response.effects
	} catch (error) {
		if (requestId !== ruleTestRequestId) return

		ruleTestError.value = error instanceof Error ? error.message : 'The rule could not be tested.'
	} finally {
		if (requestId === ruleTestRequestId) {
			isTestingRule.value = false
		}
	}
}

const testRuleDebounced = useDebounceFn(testRule, 350)

function queueRuleTest() {
	if (!isRuleModalOpen.value) return

	ruleTestRequestId += 1
	ruleTestEffects.value = []
	ruleTestError.value = null
	isTestingRule.value = true
	void testRuleDebounced()
}

async function loadRules() {
	isLoading.value = true
	loadFailed.value = false
	try {
		rules.value = await client.labrinth.tech_review_internal.getRules()
	} catch (error) {
		console.error('Failed to load Delphi rules', error)
		loadFailed.value = true
	} finally {
		isLoading.value = false
	}
}

function openCreateModal() {
	editingRuleId.value = null
	form.name = ''
	form.rule = DEFAULT_RULE
	isRuleModalOpen.value = true
	ruleModal.value?.show()
	nextTick(() => ruleEditorInstance.value?.resize(true))
	void testRule()
}

function openEditModal(rule: Labrinth.TechReview.Internal.DelphiRule) {
	editingRuleId.value = rule.id
	form.name = rule.name
	form.rule = rule.rule
	isRuleModalOpen.value = true
	ruleModal.value?.show()
	nextTick(() => ruleEditorInstance.value?.resize(true))
	void testRule()
}

function closeRuleModal() {
	ruleModal.value?.hide()
}

function handleRuleModalHide() {
	isRuleModalOpen.value = false
	ruleEditorInstance.value = null
	ruleTestRequestId += 1
	isTestingRule.value = false
}

async function saveRule() {
	if (isSaving.value) return

	if (!form.name.trim() || !form.rule.trim()) {
		addNotification({
			type: 'error',
			title: 'Invalid rule',
			text: 'Enter a name and a CEL expression.',
		})
		return
	}

	isSaving.value = true
	const payload = {
		name: form.name,
		rule: form.rule,
	}

	try {
		if (editingRuleId.value === null) {
			await client.labrinth.tech_review_internal.createRule(payload)
		} else {
			await client.labrinth.tech_review_internal.updateRule(editingRuleId.value, payload)
		}
		closeRuleModal()
		addNotification({
			type: 'success',
			title: 'Rule saved',
			text: 'The rule will take effect after the next manual scan.',
		})
		await loadRules()
	} catch (error) {
		console.error('Failed to save Delphi rule', error)
		addNotification({
			type: 'error',
			title: 'Failed to save rule',
			text: 'Check the CEL expression and try again.',
		})
	} finally {
		isSaving.value = false
	}
}

function openDeleteModal(rule: Labrinth.TechReview.Internal.DelphiRule) {
	ruleToDelete.value = rule
	deleteModal.value?.show()
}

async function deleteRule() {
	const rule = ruleToDelete.value
	if (!rule) return

	try {
		await client.labrinth.tech_review_internal.deleteRule(rule.id)
		addNotification({
			type: 'success',
			title: 'Rule deleted',
			text: `${rule.name} will be removed by the next manual scan.`,
		})
		await loadRules()
	} catch (error) {
		console.error('Failed to delete Delphi rule', error)
		addNotification({
			type: 'error',
			title: 'Failed to delete rule',
			text: 'The Delphi rule could not be deleted.',
		})
	} finally {
		ruleToDelete.value = null
	}
}

onMounted(loadRules)
</script>
