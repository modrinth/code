<template>
	<NewModal ref="ruleModal" :header="modalTitle">
		<form class="flex w-[36rem] max-w-full flex-col gap-3" @submit.prevent="saveRule">
			<label class="font-semibold text-contrast" for="rule-name">Name</label>
			<StyledInput
				id="rule-name"
				v-model="form.name"
				type="text"
				maxlength="128"
				placeholder="Known-safe obfuscated bootstrap"
			/>

			<label class="font-semibold text-contrast" for="rule-priority">Priority</label>
			<StyledInput id="rule-priority" v-model="form.priority" type="number" placeholder="0" />
			<p class="m-0 text-sm text-secondary">Higher-priority rules are evaluated first.</p>

			<label class="font-semibold text-contrast" for="rule-expression">CEL expression</label>
			<StyledInput
				id="rule-expression"
				v-model="form.expression"
				type="text"
				multiline
				resize="vertical"
				class="font-mono"
				input-class="min-h-64 font-mono"
			/>
			<p class="m-0 text-sm text-secondary">
				Return <code>null</code> when the rule does not match, or a map containing
				<code>severity</code> and/or <code>hidden</code> when it does.
			</p>

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
		description="This permanently deletes the rule, its revisions, and its materialized effects."
		:markdown="false"
		proceed-label="Delete rule"
		@proceed="deleteRule"
	/>

	<div class="flex flex-col gap-4">
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
						<p class="m-0 text-sm text-secondary">
							Priority {{ rule.priority }} · revision {{ rule.revision_id }}
						</p>
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
				><code>{{ rule.expression }}</code></pre>
			</article>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { ArrowLeftIcon, EditIcon, LoaderCircleIcon, PlusIcon, TrashIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	ConfirmModal,
	EmptyState,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	StyledInput,
} from '@modrinth/ui'

const DEFAULT_EXPRESSION = `input.trace.issue_type == "OBFUSCATED_NAMES"
	? {"severity": "low", "hidden": false}
	: null`

useHead({ title: 'Delphi rules - Modrinth' })

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const ruleModal = useTemplateRef<InstanceType<typeof NewModal>>('ruleModal')
const deleteModal = useTemplateRef<InstanceType<typeof ConfirmModal>>('deleteModal')

const rules = ref<Labrinth.TechReview.Internal.DelphiRule[]>([])
const isLoading = ref(true)
const isSaving = ref(false)
const loadFailed = ref(false)
const editingRuleId = ref<number | null>(null)
const ruleToDelete = ref<Labrinth.TechReview.Internal.DelphiRule | null>(null)
const form = reactive({
	name: '',
	priority: '0',
	expression: DEFAULT_EXPRESSION,
})

const modalTitle = computed(() => (editingRuleId.value === null ? 'Create rule' : 'Edit rule'))

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
	form.priority = '0'
	form.expression = DEFAULT_EXPRESSION
	ruleModal.value?.show()
}

function openEditModal(rule: Labrinth.TechReview.Internal.DelphiRule) {
	editingRuleId.value = rule.id
	form.name = rule.name
	form.priority = String(rule.priority)
	form.expression = rule.expression
	ruleModal.value?.show()
}

function closeRuleModal() {
	ruleModal.value?.hide()
}

async function saveRule() {
	if (isSaving.value) return

	const priority = Number(form.priority)
	if (!form.name.trim() || !form.expression.trim() || !Number.isInteger(priority)) {
		addNotification({
			type: 'error',
			title: 'Invalid rule',
			text: 'Enter a name, a CEL expression, and an integer priority.',
		})
		return
	}

	isSaving.value = true
	const payload = {
		name: form.name,
		priority,
		expression: form.expression,
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
			text: 'The Delphi rule revision was saved.',
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
			text: `${rule.name} was deleted.`,
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
