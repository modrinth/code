<template>
	<div class="flex flex-col" :class="active ? 'gap-6' : 'gap-4'">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<slot name="tabs" />
			<ButtonLink
				v-if="active"
				type="colored"
				color="brand"
				:disabled="isDeleting"
				:to="{ query: { ...$route.query, rule: 'new' }, hash: $route.hash }"
			>
				<PlusIcon />
				Create rule
			</ButtonLink>
		</div>

		<slot />
		<div v-show="active" class="flex min-w-0 flex-col gap-6">
			<div
				v-if="isPending"
				role="status"
				class="universal-card flex items-center justify-center gap-2 py-8"
			>
				<LoaderCircleIcon class="animate-spin" />
				Loading project flag rules…
			</div>
			<div v-if="isError" role="alert" class="universal-card flex flex-col items-start gap-3">
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<template v-if="data">
				<div v-if="rules.length === 0" class="universal-card flex flex-col gap-2 py-8 text-center">
					<h2 class="m-0 text-lg font-bold text-contrast">No project flag rules</h2>
					<p class="m-0 text-secondary">Create a rule to get started.</p>
				</div>
				<ProjectFlagRuleCard
					v-for="rule in rules"
					:key="rule.id"
					:id="`rule-${rule.id}`"
					:rule="rule"
					:edit-disabled="isDeleting"
					:delete-disabled="isDeleting || ruleToDelete?.id === rule.id"
					@request-delete="confirmDelete(rule)"
				>
					<template #delete-confirmation>
						<div
							v-if="ruleToDelete?.id === rule.id"
							class="flex cursor-auto flex-col gap-3 rounded-xl border border-solid border-surface-4 p-3"
							role="group"
							aria-label="Confirm rule deletion"
							@click.stop
						>
							<p class="m-0 text-contrast">Delete “{{ rule.name }}”? This cannot be undone.</p>
							<p class="m-0 text-sm text-secondary">
								Existing flags will be preserved without a link to this rule.
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
			</template>
		</div>
	</div>
</template>

<script setup lang="ts">
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'

import type { ProjectFlagRule } from '@modrinth/api-client'
import { LoaderCircleIcon, PlusIcon } from '@modrinth/assets'
import { Button, ButtonLink } from '@modrinth/ui'

import ProjectFlagRuleCard from './rule-card.vue'
import { useRules } from './use-rules'

const props = defineProps<{ active: boolean }>()
const route = useRoute()
const ruleToDelete = ref<ProjectFlagRule | null>(null)
const {
	data,
	isPending,
	isError,
	error,
	isFetching,
	refetch,
	rules,
	deleteRule,
	isDeleting,
	deleteError,
	resetDelete,
} = useRules(() => props.active, handleDeleted)

watch(
	() => route.query.rule,
	(rule) => {
		if (rule && !isDeleting.value) ruleToDelete.value = null
	},
)

function handleDeleted() {
	ruleToDelete.value = null
}

function confirmDelete(rule: ProjectFlagRule) {
	resetDelete()
	ruleToDelete.value = rule
}

useFlagErrorNotification(error, 'Could not load project flag rules')
useFlagErrorNotification(deleteError, 'Could not delete the rule')
</script>
