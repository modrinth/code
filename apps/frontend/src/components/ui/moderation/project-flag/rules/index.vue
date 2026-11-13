<template>
	<div class="flex flex-col" :class="active ? 'gap-6' : 'gap-4'">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<slot name="tabs" />
			<Button
				v-if="active"
				type="colored"
				color="brand"
				:disabled="showDraft || isDeleting || (offset > 0 && dirtyRules.size > 0)"
				@click="createDraft"
			>
				<PlusIcon />
				Create rule
			</Button>
		</div>

		<slot />
		<div v-show="active" class="flex min-w-0 flex-col gap-6">
			<ProjectFlagRuleCard
				v-if="showDraft"
				@saved="showDraft = false"
				@cancel-create="showDraft = false"
			/>

			<div
				v-if="isPending"
				role="status"
				class="universal-card flex items-center justify-center gap-2 py-8"
			>
				<LoaderCircleIcon class="animate-spin" />
				Loading project flag rules…
			</div>
			<div v-if="isError" role="alert" class="universal-card flex flex-col items-start gap-3">
				<p class="m-0 text-red">Could not load project flag rules: {{ errorMessage(error) }}</p>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<template v-if="data">
				<div
					v-if="displayedRules.length === 0 && !showDraft"
					class="universal-card flex flex-col gap-2 py-8 text-center"
				>
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
				<ProjectFlagRuleCard
					v-for="rule in displayedRules"
					:key="rule.id"
					:rule="rule"
					:delete-disabled="isDeleting"
					:deleting="ruleToDelete?.id === rule.id"
					@dirty-change="setDirty(rule, $event)"
					@request-delete="confirmDelete(rule)"
				>
					<template #delete-confirmation>
						<div
							v-if="ruleToDelete?.id === rule.id"
							class="mt-5 flex flex-col gap-3 rounded-xl border border-solid border-surface-4 p-3"
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
								<Button
									type="colored"
									color="red"
									:disabled="isDeleting"
									@click="deleteRule(rule.id)"
								>
									<LoaderCircleIcon v-if="isDeleting" class="animate-spin" />
									{{ isDeleting ? 'Deleting…' : 'Delete rule' }}
								</Button>
							</div>
						</div>
					</template>
				</ProjectFlagRuleCard>

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
							:disabled="
								offset === 0 || isFetching || isDeleting || showDraft || dirtyRules.size > 0
							"
							@click="changePage(-1)"
						>
							Previous
						</Button>
						<Button
							:disabled="
								!hasNextPage || isFetching || isDeleting || showDraft || dirtyRules.size > 0
							"
							@click="changePage(1)"
						>
							Next
						</Button>
					</div>
				</div>
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRule } from '@modrinth/api-client'
import { LoaderCircleIcon, PlusIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'

import ProjectFlagRuleCard from './rule-card.vue'
import { RULES_PAGE_SIZE, useRules } from './use-rules'

const props = defineProps<{ active: boolean }>()
const offset = ref(0)
const showDraft = ref(false)
const dirtyRules = reactive(new Map<number, ProjectFlagRule>())
const ruleToDelete = ref<ProjectFlagRule | null>(null)
const {
	data,
	isPending,
	isError,
	error,
	isFetching,
	refetch,
	rules,
	hasNextPage,
	deleteRule,
	isDeleting,
	deleteError,
	resetDelete,
} = useRules(offset, () => props.active, handleDeleted)

const displayedRules = computed(() => {
	const visibleIds = new Set(rules.value.map((rule) => rule.id))
	return [...rules.value, ...[...dirtyRules.values()].filter((rule) => !visibleIds.has(rule.id))]
})
function handleDeleted() {
	ruleToDelete.value = null
	if (rules.value.length === 1 && offset.value > 0 && dirtyRules.size === 0) {
		offset.value = Math.max(0, offset.value - RULES_PAGE_SIZE)
	}
}

function createDraft() {
	ruleToDelete.value = null
	offset.value = 0
	showDraft.value = true
}

function setDirty(rule: ProjectFlagRule, dirty: boolean) {
	if (dirty) dirtyRules.set(rule.id, rule)
	else dirtyRules.delete(rule.id)
}

function confirmDelete(rule: ProjectFlagRule) {
	resetDelete()
	ruleToDelete.value = rule
}

function changePage(direction: number) {
	ruleToDelete.value = null
	offset.value = Math.max(0, offset.value + direction * RULES_PAGE_SIZE)
}

function errorMessage(value: unknown) {
	return value instanceof Error ? value.message : 'Please try again.'
}
</script>
