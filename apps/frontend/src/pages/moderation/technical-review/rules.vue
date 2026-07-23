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

			<details class="rounded-xl border border-divider bg-bg-raised p-3">
				<summary class="cursor-pointer font-semibold text-contrast">
					Input and output schema
				</summary>
				<div v-if="isLoadingRuleSchema" class="mt-3 flex items-center gap-2 text-secondary">
					<LoaderCircleIcon class="size-4 animate-spin" />
					Loading schema…
				</div>
				<p v-else-if="ruleSchemaError" class="m-0 mt-3 text-sm text-red">
					{{ ruleSchemaError }}
				</p>
				<div v-else class="mt-3 grid gap-3 md:grid-cols-2">
					<div class="min-w-0">
						<p class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
							Input (<code>input</code>)
						</p>
						<pre
							class="m-0 overflow-x-auto rounded-lg bg-surface-1 p-3 text-xs leading-relaxed text-contrast"
						><code>{{ ruleInputSchemaText }}</code></pre>
					</div>
					<div class="min-w-0">
						<p class="m-0 mb-2 text-xs font-semibold uppercase tracking-wide text-secondary">
							Output
						</p>
						<pre
							class="m-0 overflow-x-auto rounded-lg bg-surface-1 p-3 text-xs leading-relaxed text-contrast"
						><code>{{ ruleOutputSchemaText }}</code></pre>
					</div>
				</div>
			</details>

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
	<ConfirmModal
		ref="scanModal"
		title="Run a full Delphi rule scan?"
		description="Every stored issue detail will be evaluated against the current rules. Existing effects remain active unless the entire scan succeeds."
		:markdown="false"
		proceed-label="Run full scan"
		@proceed="runFullScan"
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

			<div class="flex flex-wrap gap-2">
				<ButtonStyled>
					<button type="button" :disabled="isScanning" @click="scanModal?.show()">
						<PlayIcon />
						{{ isScanning ? 'Scanning...' : 'Run full scan' }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" :disabled="isScanning" @click="openCreateModal">
						<PlusIcon />
						Create rule
					</button>
				</ButtonStyled>
			</div>
		</div>

		<section v-if="isScanning && scanProgress" class="universal-card flex flex-col gap-3">
			<div class="flex flex-wrap items-center justify-between gap-2">
				<div>
					<h2 class="m-0 text-base font-bold text-contrast">Scanning Delphi rule effects</h2>
					<p class="m-0 text-sm text-secondary">
						{{ scanProgress.scanned.toLocaleString() }} of
						{{ scanProgress.total.toLocaleString() }} details scanned ·
						{{ scanProgress.effects.toLocaleString() }} effects
					</p>
				</div>
				<span class="text-sm font-semibold capitalize text-secondary">
					{{ scanProgress.phase }} revision {{ scanProgress.revision }}
				</span>
			</div>
			<ProgressBar
				:progress="scanProgress.scanned"
				:max="Math.max(scanProgress.total, 1)"
				:waiting="scanProgress.total === 0 && scanProgress.phase !== 'complete'"
				full-width
				show-progress
			/>
		</section>

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
			<article
				v-for="rule in rules"
				:key="rule.id"
				class="universal-card relative flex flex-col gap-3 overflow-hidden"
			>
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="m-0 text-lg font-bold text-contrast">{{ rule.name }}</h2>
						<p class="m-0 text-sm text-secondary">Revision {{ rule.revision }}</p>
					</div>
					<div class="flex gap-2">
						<ButtonStyled>
							<button type="button" :disabled="isScanning" @click="openEditModal(rule)">
								<EditIcon />
								Edit
							</button>
						</ButtonStyled>
						<ButtonStyled color="red">
							<button type="button" :disabled="isScanning" @click="openDeleteModal(rule)">
								<TrashIcon />
								Delete
							</button>
						</ButtonStyled>
					</div>
				</div>
				<pre
					class="m-0 overflow-x-auto rounded-lg bg-bg-raised p-3 text-sm"
				><code>{{ rule.rule }}</code></pre>

				<section class="flex flex-col gap-2">
					<h3 class="m-0 text-sm font-semibold text-contrast">
						Affected details ({{ rule.affected_details_count.toLocaleString() }})
					</h3>
					<p v-if="rule.affected_details_count === 0" class="m-0 text-sm text-secondary">
						No details are affected in the current revision.
					</p>
					<div v-else class="flex flex-col gap-2">
						<div
							v-for="detail in getVisibleRuleDetails(rule)"
							:key="detail.detail_id"
							class="flex min-w-0 items-center justify-between gap-3 rounded-lg border border-divider bg-bg-raised px-3 py-2"
						>
							<div class="min-w-0">
								<div class="mb-1 flex min-w-0 items-center gap-1.5 text-sm">
									<NuxtLink
										v-if="detail.project_id"
										:to="getProjectLink(detail)"
										class="flex min-w-0 items-center gap-1.5 font-semibold text-contrast hover:underline"
									>
										<Avatar
											:src="detail.project_icon_url"
											:alt="detail.project_name ?? ''"
											size="xs"
											no-shadow
										/>
										<span class="truncate">{{ detail.project_name ?? detail.project_id }}</span>
									</NuxtLink>
									<span v-else class="text-secondary">Unattached trace</span>
									<template v-if="detail.project_id && detail.version_id">
										<span class="shrink-0 text-secondary" aria-hidden="true">·</span>
										<NuxtLink
											:to="getVersionLink(detail)"
											class="truncate text-secondary hover:underline"
										>
											{{ detail.version_name ?? detail.version_number ?? detail.version_id }}
										</NuxtLink>
									</template>
								</div>
								<div class="flex min-w-0 flex-wrap items-center gap-2">
									<span
										v-if="!detail.hidden"
										class="rounded-full border px-2 py-0.5 text-xs font-semibold capitalize"
										:class="getSeverityBadgeColor(detail.severity ?? detail.original_severity)"
									>
										{{ detail.severity ?? detail.original_severity }}
									</span>
									<span v-else class="flex items-center gap-1 text-xs font-semibold text-secondary">
										<EyeOffIcon class="size-4" />
										Hidden
									</span>
									<strong class="truncate text-sm text-contrast">{{ detail.issue_type }}</strong>
								</div>
								<p
									class="m-0 mt-0.5 flex min-w-0 items-center gap-1 font-mono text-xs text-secondary"
								>
									<template v-if="detail.jar">
										<span class="truncate">{{ detail.jar }}</span>
										<ChevronRightIcon class="size-3.5 shrink-0" aria-hidden="true" />
									</template>
									<span class="truncate">{{ detail.file_path }}</span>
								</p>
							</div>
							<ButtonStyled>
								<NuxtLink v-if="detail.project_id" :to="getAffectedDetailLink(detail)">
									<ExternalIcon />
									View
								</NuxtLink>
								<button
									v-else
									type="button"
									disabled
									title="This trace is not attached to a project"
								>
									<ExternalIcon />
									View
								</button>
							</ButtonStyled>
						</div>

						<div
							v-if="rule.affected_details_count > 3"
							class="relative z-20 mt-1 flex justify-center"
						>
							<ButtonStyled circular type="transparent">
								<button
									type="button"
									:disabled="loadingAffectedRuleIds.has(rule.id)"
									@click="toggleAffectedDetails(rule)"
								>
									<LoaderCircleIcon
										v-if="loadingAffectedRuleIds.has(rule.id)"
										class="animate-spin"
									/>
									{{ expandedAffectedDetails.has(rule.id) ? 'Show less' : 'Show more' }}
								</button>
							</ButtonStyled>
						</div>
					</div>
				</section>
				<div
					v-if="rule.affected_details_count > 3 && !expandedAffectedDetails.has(rule.id)"
					class="pointer-events-none absolute inset-0 z-10 bg-gradient-to-b from-transparent to-surface-3"
					aria-hidden="true"
				/>
			</article>
		</div>
	</div>
</template>

<script setup lang="ts">
import { type Labrinth, SseParser } from '@modrinth/api-client'
import {
	ArrowLeftIcon,
	ChevronRightIcon,
	EditIcon,
	EyeOffIcon,
	ExternalIcon,
	LoaderCircleIcon,
	PlayIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	ButtonStyled,
	Avatar,
	ConfirmModal,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	ProgressBar,
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
const scanModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('scanModal')
const editorComponent = shallowRef<Component | null>(null)
const ruleEditorInstance = shallowRef<Ace.Editor | null>(null)

const rules = ref<Labrinth.TechReview.Internal.DelphiRule[]>([])
const isLoading = ref(true)
const isSaving = ref(false)
const isScanning = ref(false)
const isTestingRule = ref(false)
const isLoadingRuleSchema = ref(false)
const isRuleModalOpen = ref(false)
const loadFailed = ref(false)
const ruleSchemaError = ref<string | null>(null)
const ruleSchema = ref<Labrinth.TechReview.Internal.DelphiRuleSchemaResponse | null>(null)
const editingRuleId = ref<number | null>(null)
const ruleToDelete = ref<Labrinth.TechReview.Internal.DelphiRule | null>(null)
const ruleTestEffects = ref<Array<Labrinth.TechReview.Internal.DelphiRuleEffect | null>>([])
const ruleTestError = ref<string | null>(null)
const scanProgress = ref<Labrinth.TechReview.Internal.DelphiRuleScanEvent | null>(null)
const expandedAffectedDetails = reactive(
	new Map<number, Labrinth.TechReview.Internal.DelphiRuleAffectedDetail[]>(),
)
const loadingAffectedRuleIds = reactive(new Set<number>())
const form = reactive({
	name: '',
	rule: DEFAULT_RULE,
})
let ruleTestRequestId = 0
let scanAbortController: AbortController | null = null

onMounted(async () => {
	const [{ VAceEditor }] = await Promise.all([
		import('vue3-ace-editor'),
		import('@modrinth/ui/src/utils/ace-theme'),
	])
	editorComponent.value = VAceEditor
})

const modalTitle = computed(() => (editingRuleId.value === null ? 'Create rule' : 'Edit rule'))
const ruleInputSchemaText = computed(() =>
	ruleSchema.value ? formatRuleSchema(ruleSchema.value.input, ruleSchema.value.components) : '',
)
const ruleOutputSchemaText = computed(() =>
	ruleSchema.value ? formatRuleSchema(ruleSchema.value.output, ruleSchema.value.components) : '',
)
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

function isSchema(value: unknown): value is Labrinth.TechReview.Internal.DelphiRuleSchema {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function formatRuleSchema(
	schema: Labrinth.TechReview.Internal.DelphiRuleSchema,
	components: Record<string, Labrinth.TechReview.Internal.DelphiRuleSchema>,
	depth = 0,
	visitedReferences = new Set<string>(),
): string {
	if (typeof schema.$ref === 'string') {
		const name = decodeURIComponent(schema.$ref.split('/').at(-1) ?? '')
		if (visitedReferences.has(name)) return name || 'unknown'

		const referencedSchema = components[name]
		if (!referencedSchema) return name || 'unknown'

		const visited = new Set(visitedReferences)
		visited.add(name)
		return formatRuleSchema(referencedSchema, components, depth, visited)
	}

	const resolved = schema
	const alternatives = [resolved.oneOf, resolved.anyOf].find(Array.isArray)
	if (alternatives) {
		return alternatives
			.filter(isSchema)
			.map((alternative) => formatRuleSchema(alternative, components, depth, visitedReferences))
			.join(' | ')
	}

	if (Array.isArray(resolved.enum)) {
		return resolved.enum.map((value) => JSON.stringify(value)).join(' | ')
	}

	const declaredTypes = Array.isArray(resolved.type)
		? resolved.type.filter((type): type is string => typeof type === 'string')
		: typeof resolved.type === 'string'
			? [resolved.type]
			: []
	const nullable = resolved.nullable === true || declaredTypes.includes('null')
	const type = declaredTypes.find((value) => value !== 'null')
	let formatted: string

	if (type === 'object' || isSchema(resolved.properties) || resolved.additionalProperties) {
		const properties = isSchema(resolved.properties) ? resolved.properties : {}
		const required = new Set(
			Array.isArray(resolved.required)
				? resolved.required.filter((name): name is string => typeof name === 'string')
				: [],
		)
		const indentation = '  '.repeat(depth)
		const childIndentation = '  '.repeat(depth + 1)
		const lines = Object.entries(properties)
			.filter((entry): entry is [string, Labrinth.TechReview.Internal.DelphiRuleSchema] =>
				isSchema(entry[1]),
			)
			.map(
				([name, property]) =>
					`${childIndentation}${JSON.stringify(name)}${required.has(name) ? '' : '?'}: ${formatRuleSchema(property, components, depth + 1, visitedReferences)}`,
			)

		if (isSchema(resolved.additionalProperties)) {
			lines.push(
				`${childIndentation}[key: string]: ${formatRuleSchema(resolved.additionalProperties, components, depth + 1, visitedReferences)}`,
			)
		} else if (resolved.additionalProperties === true) {
			lines.push(`${childIndentation}[key: string]: unknown`)
		}

		formatted = lines.length === 0 ? '{}' : `{\n${lines.join(',\n')}\n${indentation}}`
	} else if (type === 'array') {
		formatted = isSchema(resolved.items)
			? `Array<${formatRuleSchema(resolved.items, components, depth, visitedReferences)}>`
			: 'unknown[]'
	} else if (type === 'integer' || type === 'number') {
		formatted = 'number'
	} else if (type === 'boolean' || type === 'string' || type === 'null') {
		formatted = type
	} else {
		formatted = 'unknown'
	}

	return nullable && formatted !== 'null' ? `${formatted} | null` : formatted
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
		expandedAffectedDetails.clear()
	} catch (error) {
		console.error('Failed to load Delphi rules', error)
		loadFailed.value = true
	} finally {
		isLoading.value = false
	}
}

async function loadRuleSchema() {
	if (ruleSchema.value || isLoadingRuleSchema.value) return

	isLoadingRuleSchema.value = true
	ruleSchemaError.value = null
	try {
		ruleSchema.value = await client.labrinth.tech_review_internal.getRuleSchema()
	} catch (error) {
		console.error('Failed to load Delphi rule schema', error)
		ruleSchemaError.value = 'The rule input and output schema could not be loaded.'
	} finally {
		isLoadingRuleSchema.value = false
	}
}

function getVisibleRuleDetails(
	rule: Labrinth.TechReview.Internal.DelphiRule,
): Labrinth.TechReview.Internal.DelphiRuleAffectedDetail[] {
	return expandedAffectedDetails.get(rule.id) ?? rule.affected_details
}

function getAffectedDetailLink(
	detail: Labrinth.TechReview.Internal.DelphiRuleAffectedDetail,
): string {
	return `/moderation/technical-review/${detail.project_id}?detail=${encodeURIComponent(detail.detail_id)}`
}

function getProjectLink(detail: Labrinth.TechReview.Internal.DelphiRuleAffectedDetail): string {
	return `/project/${detail.project_id}`
}

function getVersionLink(detail: Labrinth.TechReview.Internal.DelphiRuleAffectedDetail): string {
	return `/project/${detail.project_id}/version/${detail.version_id}`
}

async function toggleAffectedDetails(rule: Labrinth.TechReview.Internal.DelphiRule) {
	if (expandedAffectedDetails.has(rule.id)) {
		expandedAffectedDetails.delete(rule.id)
		return
	}
	if (loadingAffectedRuleIds.has(rule.id)) return

	loadingAffectedRuleIds.add(rule.id)
	try {
		const details = await client.labrinth.tech_review_internal.getRuleAffectedDetails(rule.id)
		expandedAffectedDetails.set(rule.id, details)
	} catch (error) {
		console.error('Failed to load details affected by Delphi rule', error)
		addNotification({
			type: 'error',
			title: 'Failed to load affected details',
			text: 'The complete list of affected details could not be loaded.',
		})
	} finally {
		loadingAffectedRuleIds.delete(rule.id)
	}
}

function openCreateModal() {
	if (isScanning.value) return
	editingRuleId.value = null
	form.name = ''
	form.rule = DEFAULT_RULE
	isRuleModalOpen.value = true
	ruleModal.value?.show()
	nextTick(() => ruleEditorInstance.value?.resize(true))
	void loadRuleSchema()
	void testRule()
}

function openEditModal(rule: Labrinth.TechReview.Internal.DelphiRule) {
	if (isScanning.value) return
	editingRuleId.value = rule.id
	form.name = rule.name
	form.rule = rule.rule
	isRuleModalOpen.value = true
	ruleModal.value?.show()
	nextTick(() => ruleEditorInstance.value?.resize(true))
	void loadRuleSchema()
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
	if (isSaving.value || isScanning.value) return

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
	if (isScanning.value) return
	ruleToDelete.value = rule
	deleteModal.value?.show()
}

async function deleteRule() {
	if (isScanning.value) return
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

async function runFullScan() {
	if (isScanning.value) return

	isScanning.value = true
	scanProgress.value = null
	scanAbortController = new AbortController()
	let completed = false

	try {
		const stream = await client.labrinth.tech_review_internal.scanRules(scanAbortController.signal)
		const reader = stream.getReader()
		const decoder = new TextDecoder()
		const parser = new SseParser()

		const processItems = (items: ReturnType<SseParser['feed']>) => {
			for (const item of items) {
				if (item.kind !== 'event') continue

				if (item.event === 'failed') {
					const error = JSON.parse(
						item.data,
					) as Labrinth.TechReview.Internal.DelphiRuleScanErrorEvent
					throw new Error(error.message)
				}

				if (item.event === 'progress' || item.event === 'complete') {
					scanProgress.value = JSON.parse(
						item.data,
					) as Labrinth.TechReview.Internal.DelphiRuleScanEvent
					completed ||= item.event === 'complete'
				}
			}
		}

		while (true) {
			const { done, value } = await reader.read()
			if (done) break
			processItems(parser.feed(decoder.decode(value, { stream: true })))
		}

		const finalChunk = decoder.decode()
		if (finalChunk) processItems(parser.feed(finalChunk))
		processItems(parser.end())

		if (!completed || !scanProgress.value) {
			throw new Error('The scan stream ended before the new revision was published.')
		}

		addNotification({
			type: 'success',
			title: 'Rule scan complete',
			text: `${scanProgress.value.scanned.toLocaleString()} details were scanned for revision ${scanProgress.value.revision}.`,
		})
		await loadRules()
	} catch (error) {
		console.error('Failed to scan Delphi rules', error)
		addNotification({
			type: 'error',
			title: 'Rule scan failed',
			text: error instanceof Error ? error.message : 'The previous rule revision remains active.',
		})
	} finally {
		isScanning.value = false
		scanAbortController = null
	}
}

onMounted(loadRules)
onUnmounted(() => scanAbortController?.abort())
</script>
