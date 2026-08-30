<template>
	<NewModal ref="ruleModal" :header="modalTitle" :on-hide="handleRuleModalHide">
		<form class="flex w-[48rem] max-w-full flex-col gap-3" @submit.prevent="saveRule">
			<label class="font-semibold text-contrast" for="rule-name">Name</label>
			<Input
				id="rule-name"
				v-model="form.name"
				type="text"
				:maxlength="256"
				placeholder="Known-safe obfuscated bootstrap"
			/>

			<label class="font-semibold text-contrast" for="rule-priority">Priority</label>
			<Input
				id="rule-priority"
				v-model="form.priority"
				type="number"
				:min="-2147483648"
				:max="2147483647"
				:step="1"
			/>
			<p class="m-0 text-sm text-secondary">
				Higher-priority rules run first. Rules with the same priority run in creation order.
			</p>

			<div class="flex flex-col gap-1">
				<label class="font-semibold text-contrast">Issue types</label>
				<p class="m-0 text-sm text-secondary">
					Choose which issue types this rule evaluates. Leave empty to run it against every issue
					type.
				</p>
				<MultiSelect
					v-model="form.onIssueTypes"
					:options="issueTypeOptions"
					:disabled="isLoadingIssueTypes"
					:placeholder="isLoadingIssueTypes ? 'Loading issue types…' : 'All issue types'"
					:no-options-message="
						hasIssueTypeSchemaError ? 'Issue types could not be loaded' : 'No issue types available'
					"
					search-placeholder="Search issue types…"
					searchable
					fuzzy-search
					clearable
					:max-tag-rows="2"
				/>
				<div v-if="hasIssueTypeSchemaError" class="flex items-center justify-between gap-2">
					<p class="m-0 text-sm text-red">The available issue types could not be loaded.</p>
					<Button size="sm" :disabled="isFetchingIssueTypes" @click="refetchIssueTypes()">
						<LoaderCircleIcon v-if="isFetchingIssueTypes" class="animate-spin" />
						Try again
					</Button>
				</div>
			</div>

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
			<div
				v-if="ruleTestError"
				role="alert"
				class="border-red/40 max-h-64 overflow-auto rounded-lg border bg-highlight-red p-3 text-red"
			>
				<code class="rule-test-error">
					<span>{{ ruleTestError.summary }}</span>
					<span
						v-for="(detail, index) in ruleTestError.details"
						:key="index"
						class="rule-test-error-detail"
					>
						{{ detail }}
					</span>
				</code>
			</div>
			<a
				class="flex w-fit items-center gap-1 text-sm text-link"
				href="https://cel.dev/reference/api-reference"
				target="_blank"
				rel="noopener noreferrer"
			>
				<BookOpenIcon class="size-4" />
				CEL API reference
			</a>

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
							Context
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
						<p class="m-0 mb-2 mt-3 text-xs font-semibold uppercase tracking-wide text-secondary">
							Extensions
						</p>
						<ul class="m-0 mb-3 list-disc pl-5 text-sm text-secondary">
							<li>
								<a
									class="text-link"
									href="https://cel.dev/reference/api-reference#general_string_functions"
									target="_blank"
									rel="noopener noreferrer"
								>
									Regex
								</a>
							</li>
							<li>
								<a
									class="text-link"
									href="https://docs.rs/url/latest/url/struct.Url.html"
									target="_blank"
									rel="noopener noreferrer"
								>
									URL parsing
								</a>
								with <code>url.parse(string)</code> and
								<code>url.is_valid(string)</code>
								<div class="mt-1 text-xs">
									Parsed URLs expose <code>href</code>, <code>scheme</code>, <code>host</code>,
									<code>domain</code>, <code>port</code>, <code>path</code>,
									<code>path_segments</code>, <code>query</code>, <code>query_pairs</code>,
									<code>fragment</code>, credentials, and origin fields.
								</div>
							</li>
						</ul>
						<pre
							class="m-0 overflow-x-auto rounded-lg bg-surface-1 p-3 text-xs leading-relaxed text-contrast"
						><code>#define ISSUE_TYPE "OBFUSCATED_NAMES"
#bind IS_MATCH trace.issue_type == ISSUE_TYPE

IS_MATCH ? "low" : null</code></pre>
					</div>
				</div>
			</details>

			<section class="mt-2 flex flex-col gap-3">
				<div class="flex items-center justify-between gap-3">
					<div>
						<h3 class="m-0 text-base font-bold text-contrast">Test trace</h3>
						<p class="m-0 text-sm text-secondary">
							Change the trace details to evaluate the current expression.
						</p>
					</div>
					<LoaderCircleIcon v-if="isTestingRule" class="size-5 animate-spin text-secondary" />
				</div>

				<div class="grid items-stretch gap-3 md:grid-cols-[minmax(0,2fr)_auto_minmax(0,1fr)]">
					<article class="universal-card flex min-w-0 flex-col gap-3">
						<p class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">
							Trace details
						</p>
						<div class="grid gap-3 sm:grid-cols-2">
							<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
								Key
								<Input v-model="testTraceForm.key" placeholder="unique-trace-key" />
							</label>
							<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
								Issue type
								<Input v-model="testTraceForm.issueType" placeholder="OBFUSCATED_NAMES" />
							</label>
							<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
								Severity
								<select
									v-model="testTraceForm.severity"
									class="h-9 w-full rounded-xl border-none bg-surface-4 px-3 font-medium capitalize text-primary outline-none focus:ring-4 focus:ring-brand-shadow"
								>
									<option v-for="severity in TRACE_SEVERITIES" :key="severity" :value="severity">
										{{ severity }}
									</option>
								</select>
							</label>
							<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
								JAR
								<Input v-model="testTraceForm.jar" placeholder="META-INF/jars/embedded.jar" />
							</label>
						</div>
						<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
							File path
							<Input v-model="testTraceForm.filePath" placeholder="com/example/Bootstrap.class" />
						</label>
						<label class="flex min-w-0 flex-col gap-1 text-sm font-semibold text-contrast">
							Data (JSON)
							<Textarea
								v-model="testTraceForm.data"
								:rows="4"
								resize="vertical"
								input-class="font-mono text-sm"
								:error="Boolean(traceDataError)"
							/>
						</label>
						<p v-if="traceDataError" class="m-0 text-sm text-red">
							{{ traceDataError }}
						</p>
					</article>

					<div
						class="flex items-center justify-center text-2xl font-bold text-secondary max-md:rotate-90"
						aria-hidden="true"
					>
						→
					</div>

					<article class="universal-card flex min-w-0 flex-col gap-3">
						<p class="m-0 text-xs font-semibold uppercase tracking-wide text-secondary">
							Rule result
						</p>
						<div v-if="isTestingRule" class="flex items-center gap-2 text-secondary">
							<LoaderCircleIcon class="size-5 animate-spin" />
							<span>Evaluating…</span>
						</div>
						<p v-else-if="traceDataError || ruleTestError" class="m-0 text-sm text-secondary">
							Preview unavailable.
						</p>
						<div
							v-else-if="testTracePreview.effectiveSeverity === 'hidden'"
							class="flex items-center gap-2 text-secondary"
						>
							<EyeOffIcon class="size-5" />
							<strong class="text-contrast">Hidden from reports</strong>
						</div>
						<div v-else-if="!testTracePreview.effect" class="flex flex-col gap-2">
							<strong class="text-contrast">No match</strong>
							<p class="m-0 text-sm text-secondary">{{ testTracePreview.summary }}</p>
						</div>
						<template v-else>
							<div class="flex flex-wrap items-center gap-2">
								<span
									class="rounded-full border px-2 py-0.5 text-xs font-semibold capitalize"
									:class="getSeverityBadgeColor(testTracePreview.effectiveSeverity)"
								>
									{{ testTracePreview.effectiveSeverity }}
								</span>
								<strong class="break-all text-contrast">{{ testTraceForm.issueType }}</strong>
							</div>
							<p class="m-0 text-sm text-secondary">{{ testTracePreview.summary }}</p>
						</template>
					</article>
				</div>
			</section>

			<div class="flex justify-end gap-2">
				<Button @click="closeRuleModal">Cancel</Button>
				<Button type="colored" color="brand" native-type="submit" :disabled="isSaving">
					{{ isSaving ? 'Saving...' : 'Save rule' }}
				</Button>
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
				<ButtonLink
					to="/moderation/technical-review"
					type="quiet"
					class="!size-10 !rounded-full !p-0"
					aria-label="Back to tech review queue"
				>
					<ArrowLeftIcon />
				</ButtonLink>
				<div>
					<h1 class="m-0 text-2xl font-bold text-contrast">Delphi rules</h1>
					<p class="m-0 text-secondary">Transform or hide Delphi issue traces.</p>
				</div>
			</div>

			<div class="flex flex-wrap gap-2">
				<Button :disabled="isScanning" @click="scanModal?.show()">
					<PlayIcon />
					{{ isScanning ? 'Scanning...' : 'Run full scan' }}
				</Button>
				<Button type="colored" color="brand" :disabled="isScanning" @click="openCreateModal">
					<PlusIcon />
					Create rule
				</Button>
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
			<Button @click="loadRules">Try again</Button>
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
						<div class="mt-1 flex items-center gap-2 text-sm text-secondary">
							<span>Priority {{ rule.priority }}</span>
							<span aria-hidden="true">·</span>
							<span
								class="rounded-full border px-2 py-0.5 text-xs font-semibold"
								:class="
									isRuleLive(rule)
										? 'border-green/60 bg-highlight-green text-green'
										: 'border-orange/60 bg-highlight-orange text-orange'
								"
							>
								{{ isRuleLive(rule) ? 'Live' : 'Outdated' }}
							</span>
						</div>
						<div class="mt-2 flex flex-wrap items-center gap-1.5">
							<span class="text-xs font-semibold text-secondary">Applies to</span>
							<TagItem v-if="rule.on_issue_types.length === 0">All issue types</TagItem>
							<template v-else>
								<TagItem v-for="issueType in rule.on_issue_types" :key="issueType">
									{{ issueType }}
								</TagItem>
							</template>
						</div>
					</div>
					<div class="flex gap-2">
						<Button :disabled="isScanning" @click="openEditModal(rule)">
							<EditIcon />
							Edit
						</Button>
						<Button
							type="colored"
							color="red"
							:disabled="isScanning"
							@click="openDeleteModal(rule)"
						>
							<TrashIcon />
							Delete
						</Button>
					</div>
				</div>
				<pre
					class="m-0 overflow-x-auto rounded-lg bg-bg-raised p-3 text-sm"
				><code>{{ rule.rule }}</code></pre>

				<section class="flex flex-col gap-2">
					<h3 class="m-0 text-sm font-semibold text-contrast">
						Affected details ({{ getAffectedDetailsTotal(rule).toLocaleString() }})
					</h3>
					<p v-if="getAffectedDetailsTotal(rule) === 0" class="m-0 text-sm text-secondary">
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
										v-if="detail.severity !== 'hidden'"
										class="rounded-full border px-2 py-0.5 text-xs font-semibold capitalize"
										:class="getSeverityBadgeColor(detail.severity)"
									>
										{{ detail.severity }}
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
									<IssueDetailPath
										:segments="[detail.jar, detail.file_path]"
										hide-base-mrpack
										truncate
									/>
								</p>
							</div>
							<ButtonLink v-if="detail.project_id" :to="getAffectedDetailLink(detail)">
								<ExternalIcon />
								View
							</ButtonLink>
							<Button v-else disabled title="This trace is not attached to a project">
								<ExternalIcon />
								View
							</Button>
						</div>

						<div
							v-if="getAffectedDetailsPageCount(rule) > 1"
							class="mt-1 flex flex-wrap items-center justify-between gap-2"
						>
							<p class="m-0 flex items-center gap-2 text-sm text-secondary">
								<LoaderCircleIcon
									v-if="loadingAffectedRuleIds.has(rule.id)"
									class="size-4 animate-spin"
								/>
								Showing {{ getAffectedDetailsPageStart(rule).toLocaleString() }}–{{
									getAffectedDetailsPageEnd(rule).toLocaleString()
								}}
								of {{ getAffectedDetailsTotal(rule).toLocaleString() }}
							</p>
							<Pagination
								:page="getAffectedDetailsPage(rule)"
								:count="getAffectedDetailsPageCount(rule)"
								@switch-page="switchAffectedDetailsPage(rule, $event)"
							/>
						</div>
					</div>
				</section>
			</article>
		</div>
	</div>
</template>

<script setup lang="ts">
import { type Labrinth, ModrinthServerError, SseParser } from '@modrinth/api-client'
import {
	ArrowLeftIcon,
	BookOpenIcon,
	EditIcon,
	ExternalIcon,
	EyeOffIcon,
	LoaderCircleIcon,
	PlayIcon,
	PlusIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Avatar,
	Button,
	ButtonLink,
	ConfirmModal,
	EmptyState,
	Input,
	injectModrinthClient,
	injectNotificationManager,
	MultiSelect,
	NewModal,
	Pagination,
	ProgressBar,
	TagItem,
	Textarea,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import type { Ace } from 'ace-builds'
import type { Component } from 'vue'

import IssueDetailPath from '~/components/ui/moderation/IssueDetailPath.vue'

const DEFAULT_RULE = `#define ISSUE_TYPE "OBFUSCATED_NAMES"

trace.issue_type == ISSUE_TYPE
	? "low"
	: null`

const CEL_LANGUAGE_COMPLETIONS: Ace.Completion[] = [
	...['true', 'false', 'null', 'in'].map((value) => ({
		value,
		score: 900,
		meta: 'keyword',
	})),
	{ caption: 'has()', snippet: 'has(${1:field})', score: 850, meta: 'macro' },
	{ caption: 'size()', snippet: 'size(${1:value})', score: 850, meta: 'function' },
	{ caption: 'string()', snippet: 'string(${1:value})', score: 850, meta: 'function' },
	{ caption: 'int()', snippet: 'int(${1:value})', score: 850, meta: 'function' },
	{ caption: 'url.parse()', snippet: 'url.parse("${1:url}")', score: 850, meta: 'URL' },
	{ caption: 'url.is_valid()', snippet: 'url.is_valid("${1:url}")', score: 850, meta: 'URL' },
	{ caption: '#define', snippet: '#define ${1:NAME} ${2:value}', score: 800, meta: 'preprocessor' },
	{
		caption: '#bind',
		snippet: '#bind ${1:NAME} ${2:expression}',
		score: 800,
		meta: 'preprocessor',
	},
]
const CEL_MEMBER_COMPLETIONS: Ace.Completion[] = [
	{ caption: 'contains()', snippet: 'contains("${1:value}")', score: 800, meta: 'string' },
	{ caption: 'startsWith()', snippet: 'startsWith("${1:prefix}")', score: 800, meta: 'string' },
	{ caption: 'endsWith()', snippet: 'endsWith("${1:suffix}")', score: 800, meta: 'string' },
	{ caption: 'matches()', snippet: 'matches("${1:pattern}")', score: 800, meta: 'regex' },
	{ caption: 'all()', snippet: 'all(${1:item}, ${2:predicate})', score: 750, meta: 'list macro' },
	{
		caption: 'exists()',
		snippet: 'exists(${1:item}, ${2:predicate})',
		score: 750,
		meta: 'list macro',
	},
	{
		caption: 'exists_one()',
		snippet: 'exists_one(${1:item}, ${2:predicate})',
		score: 750,
		meta: 'list macro',
	},
	{
		caption: 'filter()',
		snippet: 'filter(${1:item}, ${2:predicate})',
		score: 750,
		meta: 'list macro',
	},
	{ caption: 'map()', snippet: 'map(${1:item}, ${2:expression})', score: 750, meta: 'list macro' },
]
const CEL_COMPLETER: Ace.Completer = {
	id: 'delphi-cel',
	triggerCharacters: ['.'],
	getCompletions(_editor, session, position, prefix, callback) {
		const beforePrefix = session
			.getLine(position.row)
			.slice(0, Math.max(0, position.column - prefix.length))
		const receiver = beforePrefix.match(/([A-Za-z_][\w.]*)\.$/)?.[1]
		if (!receiver) {
			callback(null, [...getCelSchemaCompletions(), ...CEL_LANGUAGE_COMPLETIONS])
			return
		}

		const schemaCompletions = getCelSchemaCompletions(receiver)
		if (schemaCompletions.length > 0) {
			callback(null, schemaCompletions)
			return
		}

		callback(null, CEL_MEMBER_COMPLETIONS)
	},
}
const RULE_EDITOR_OPTIONS: Partial<Ace.EditorOptions> = {
	useWorker: false,
	tabSize: 2,
	useSoftTabs: true,
	enableBasicAutocompletion: [CEL_COMPLETER],
	enableLiveAutocompletion: [CEL_COMPLETER],
	liveAutocompletionDelay: 150,
	liveAutocompletionThreshold: 1,
	enableSnippets: true,
}

type RuleTestError = {
	summary: string
	details: string[]
}

type TestTraceForm = {
	key: string
	issueType: string
	severity: Labrinth.TechReview.Internal.DelphiSeverity
	jar: string
	filePath: string
	data: string
}

const TRACE_SEVERITIES: Labrinth.TechReview.Internal.DelphiSeverity[] = [
	'low',
	'medium',
	'high',
	'severe',
	'malware',
	'hidden',
]

const TEST_INPUT_METADATA: Omit<Labrinth.TechReview.Internal.RuleInput, 'trace' | 'file_traces'> = {
	schema_version: 1,
	scan: {
		delphi_version: 17,
	},
	artifact: {
		size: 412_892,
		hashes: {
			sha1: '0123456789abcdef',
			sha512: 'fedcba9876543210',
		},
	},
	project: {
		id: 'example-project',
		types: ['mod'],
	},
	version: {
		id: 'example-version',
		loaders: ['fabric'],
	},
	file: {
		id: 'example-file',
	},
}

function createTestTraceForm(): TestTraceForm {
	return {
		key: 'known-safe:obfuscated-bootstrap',
		issueType: 'OBFUSCATED_NAMES',
		severity: 'high',
		jar: 'META-INF/jars/embedded.jar',
		filePath: 'com/example/Bootstrap.class',
		data: JSON.stringify(
			{
				confidence: 0.97,
				symbol_count: 42,
			},
			null,
			2,
		),
	}
}

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
const ruleTestError = ref<RuleTestError | null>(null)
const traceDataError = ref<string | null>(null)
const scanProgress = ref<Labrinth.TechReview.Internal.DelphiRuleScanEvent | null>(null)
const affectedDetailsPages = reactive(
	new Map<
		number,
		{
			page: number
			total: number
			details: Labrinth.TechReview.Internal.DelphiRuleAffectedDetail[]
		}
	>(),
)
const loadingAffectedRuleIds = reactive(new Set<number>())
const AFFECTED_DETAILS_PAGE_SIZE = 3
const form = reactive({
	name: '',
	priority: 0 as number | undefined,
	rule: DEFAULT_RULE,
	onIssueTypes: [] as string[],
})
const testTraceForm = reactive(createTestTraceForm())
let ruleTestRequestId = 0
let scanAbortController: AbortController | null = null

onMounted(async () => {
	const [{ VAceEditor }] = await Promise.all([
		import('vue3-ace-editor'),
		import('@modrinth/ui/src/utils/ace-theme'),
	])
	await import('ace-builds/src-noconflict/ext-language_tools')
	editorComponent.value = VAceEditor
})

const modalTitle = computed(() => (editingRuleId.value === null ? 'Create rule' : 'Edit rule'))
const {
	data: issueTypeSchema,
	isPending: isLoadingIssueTypes,
	isError: hasIssueTypeSchemaError,
	isFetching: isFetchingIssueTypes,
	refetch: refetchIssueTypes,
} = useQuery({
	queryKey: ['delphi', 'issue-types', 'schema'] as const,
	queryFn: () => client.labrinth.tech_review_internal.getIssueTypeSchema(),
	staleTime: 60_000,
})
const issueTypeOptions = computed(() =>
	[...new Set([...Object.keys(issueTypeSchema.value ?? {}), ...form.onIssueTypes])]
		.sort((first, second) => first.localeCompare(second))
		.map((issueType) => ({
			value: issueType,
			label: issueType,
		})),
)
const ruleInputSchemaText = computed(() =>
	ruleSchema.value ? formatRuleSchema(ruleSchema.value.input, ruleSchema.value.components) : '',
)
const ruleOutputSchemaText = computed(() =>
	ruleSchema.value ? formatRuleSchema(ruleSchema.value.output, ruleSchema.value.components) : '',
)
const testTracePreview = computed(() => {
	const effect = ruleTestEffects.value[0] ?? null
	const effectiveSeverity = effect?.severity ?? testTraceForm.severity
	let summary: string

	if (!effect) {
		summary = 'This rule would not change the trace.'
	} else if (effect.severity !== testTraceForm.severity) {
		summary = `Severity changed from ${testTraceForm.severity} to ${effect.severity}.`
	} else {
		summary = 'The rule matched without changing the severity.'
	}

	return {
		effect,
		effectiveSeverity,
		summary,
	}
})

function isRuleLive(rule: Labrinth.TechReview.Internal.DelphiRule): boolean {
	return rule.current_revision === undefined || rule.revision === rule.current_revision
}

function getSeverityBadgeColor(severity: Labrinth.TechReview.Internal.DelphiSeverity): string {
	switch (severity) {
		case 'malware':
		case 'severe':
			return 'border-red/60 bg-highlight-red text-red'
		case 'high':
			return 'border-orange/60 bg-highlight-orange text-orange'
		case 'medium':
			return 'border-green/60 bg-highlight-green text-green'
		case 'hidden':
			return 'border-divider bg-surface-2 text-secondary'
		case 'low':
		default:
			return 'border-blue/60 bg-highlight-blue text-blue'
	}
}

function isSchema(value: unknown): value is Labrinth.TechReview.Internal.DelphiRuleSchema {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function resolveRuleSchema(
	schema: Labrinth.TechReview.Internal.DelphiRuleSchema,
	components: Record<string, Labrinth.TechReview.Internal.DelphiRuleSchema>,
): Labrinth.TechReview.Internal.DelphiRuleSchema {
	let resolved = schema
	const visited = new Set<string>()

	while (typeof resolved.$ref === 'string') {
		const name = decodeURIComponent(resolved.$ref.split('/').at(-1) ?? '')
		if (!name || visited.has(name) || !components[name]) break

		visited.add(name)
		resolved = components[name]
	}

	return resolved
}

function getRuleSchemaLabel(
	schema: Labrinth.TechReview.Internal.DelphiRuleSchema,
	components: Record<string, Labrinth.TechReview.Internal.DelphiRuleSchema>,
): string {
	if (typeof schema.$ref === 'string') {
		return decodeURIComponent(schema.$ref.split('/').at(-1) ?? '') || 'context'
	}

	const resolved = resolveRuleSchema(schema, components)
	const types = Array.isArray(resolved.type)
		? resolved.type.filter((type): type is string => typeof type === 'string')
		: typeof resolved.type === 'string'
			? [resolved.type]
			: []
	return types.join(' | ') || 'context'
}

function getCelSchemaCompletions(receiver?: string): Ace.Completion[] {
	const response = ruleSchema.value
	if (!response) return []

	let schema = response.input
	for (const segment of receiver?.split('.') ?? []) {
		const resolved = resolveRuleSchema(schema, response.components)
		if (!isSchema(resolved.properties)) return []

		const property = resolved.properties[segment]
		if (!isSchema(property)) return []
		schema = property
	}

	const resolved = resolveRuleSchema(schema, response.components)
	if (!isSchema(resolved.properties)) return []

	return Object.entries(resolved.properties)
		.filter((entry): entry is [string, Labrinth.TechReview.Internal.DelphiRuleSchema] =>
			isSchema(entry[1]),
		)
		.map(([value, property]) => ({
			value,
			score: 1_000,
			meta: getRuleSchemaLabel(property, response.components),
		}))
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

function getTestRuleInput(): Labrinth.TechReview.Internal.RuleInput | null {
	let data: unknown
	try {
		data = JSON.parse(testTraceForm.data)
	} catch {
		traceDataError.value = 'Enter valid JSON data for the trace.'
		return null
	}

	if (typeof data !== 'object' || data === null || Array.isArray(data)) {
		traceDataError.value = 'Trace data must be a JSON object.'
		return null
	}

	traceDataError.value = null
	const trace: Labrinth.TechReview.Internal.RuleTrace = {
		key: testTraceForm.key,
		issue_type: testTraceForm.issueType,
		severity: testTraceForm.severity,
		jar: testTraceForm.jar.trim() || null,
		file_path: testTraceForm.filePath,
		data: data as Record<string, unknown>,
	}

	return {
		...TEST_INPUT_METADATA,
		trace,
		file_traces: [trace],
	}
}

async function testRule() {
	if (!isRuleModalOpen.value) return

	const requestId = ++ruleTestRequestId
	const rule = form.rule.trim()
	ruleTestEffects.value = []
	ruleTestError.value = null
	traceDataError.value = null

	if (!rule) {
		isTestingRule.value = false
		ruleTestError.value = {
			summary: 'Enter a CEL expression to test it.',
			details: [],
		}
		return
	}

	const input = getTestRuleInput()
	if (!input) {
		isTestingRule.value = false
		return
	}

	isTestingRule.value = true
	try {
		const response = await client.labrinth.tech_review_internal.testRule({
			rule,
			inputs: [input],
		})
		if (requestId !== ruleTestRequestId) return

		ruleTestEffects.value = response.effects
	} catch (error) {
		if (requestId !== ruleTestRequestId) return

		const details =
			error instanceof ModrinthServerError && Array.isArray(error.v1Error?.details)
				? error.v1Error.details.filter((detail): detail is string => typeof detail === 'string')
				: []
		ruleTestError.value = {
			summary: error instanceof Error ? error.message : 'The rule could not be tested.',
			details,
		}
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

watch(testTraceForm, queueRuleTest, { flush: 'sync' })

async function loadRules() {
	isLoading.value = true
	loadFailed.value = false
	try {
		rules.value = await client.labrinth.tech_review_internal.getRules()
		affectedDetailsPages.clear()
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
	return affectedDetailsPages.get(rule.id)?.details ?? rule.affected_details
}

function getAffectedDetailsPage(rule: Labrinth.TechReview.Internal.DelphiRule): number {
	return affectedDetailsPages.get(rule.id)?.page ?? 1
}

function getAffectedDetailsTotal(rule: Labrinth.TechReview.Internal.DelphiRule): number {
	return affectedDetailsPages.get(rule.id)?.total ?? rule.affected_details_count
}

function getAffectedDetailsPageCount(rule: Labrinth.TechReview.Internal.DelphiRule): number {
	return Math.max(Math.ceil(getAffectedDetailsTotal(rule) / AFFECTED_DETAILS_PAGE_SIZE), 1)
}

function getAffectedDetailsPageStart(rule: Labrinth.TechReview.Internal.DelphiRule): number {
	const total = getAffectedDetailsTotal(rule)
	if (total === 0) return 0
	return (getAffectedDetailsPage(rule) - 1) * AFFECTED_DETAILS_PAGE_SIZE + 1
}

function getAffectedDetailsPageEnd(rule: Labrinth.TechReview.Internal.DelphiRule): number {
	return Math.min(
		getAffectedDetailsPage(rule) * AFFECTED_DETAILS_PAGE_SIZE,
		getAffectedDetailsTotal(rule),
	)
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

async function switchAffectedDetailsPage(
	rule: Labrinth.TechReview.Internal.DelphiRule,
	page: number,
) {
	if (loadingAffectedRuleIds.has(rule.id)) return
	if (page === 1) {
		affectedDetailsPages.delete(rule.id)
		return
	}

	loadingAffectedRuleIds.add(rule.id)
	try {
		const response = await client.labrinth.tech_review_internal.getRuleAffectedDetails(rule.id, {
			limit: AFFECTED_DETAILS_PAGE_SIZE,
			page: page - 1,
		})
		affectedDetailsPages.set(rule.id, {
			page,
			total: response.total,
			details: response.details,
		})
	} catch (error) {
		console.error('Failed to load details affected by Delphi rule', error)
		addNotification({
			type: 'error',
			title: 'Failed to load affected details',
			text: 'The requested page of affected details could not be loaded.',
		})
	} finally {
		loadingAffectedRuleIds.delete(rule.id)
	}
}

function openCreateModal() {
	if (isScanning.value) return
	editingRuleId.value = null
	form.name = ''
	form.priority = 0
	form.rule = DEFAULT_RULE
	form.onIssueTypes = []
	Object.assign(testTraceForm, createTestTraceForm())
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
	form.priority = rule.priority
	form.rule = rule.rule
	form.onIssueTypes = [...rule.on_issue_types]
	Object.assign(testTraceForm, createTestTraceForm())
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

	const priority = form.priority
	if (
		!form.name.trim() ||
		!form.rule.trim() ||
		typeof priority !== 'number' ||
		!Number.isInteger(priority) ||
		priority < -2147483648 ||
		priority > 2147483647
	) {
		addNotification({
			type: 'error',
			title: 'Invalid rule',
			text: 'Enter a name, an integer priority, and a CEL expression.',
		})
		return
	}

	isSaving.value = true
	const payload = {
		name: form.name,
		priority,
		rule: form.rule,
		on_issue_types: form.onIssueTypes,
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

<style scoped>
.rule-test-error {
	display: block;
	font-family: monospace;
	font-size: 0.75rem;
	line-height: 1.625;
	overflow-wrap: anywhere;
	white-space: pre-wrap;
}

.rule-test-error > span {
	display: block;
}

.rule-test-error-detail {
	margin-inline-start: 1rem;
}
</style>
