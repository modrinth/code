<template>
	<div class="flex flex-col" :class="active ? 'gap-6' : 'gap-4'">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<slot name="tabs" />
			<ButtonLink
				v-if="active"
				type="colored"
				color="brand"
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
				class="universal-card m-0 flex h-[102px] items-center justify-center gap-2"
			>
				<LoaderCircleIcon class="animate-spin" />
				Loading project flag rules…
			</div>
			<div
				v-if="isError"
				role="alert"
				class="universal-card m-0 flex h-[102px] items-center justify-center"
			>
				<Button :disabled="isFetching" @click="refetch()">Try again</Button>
			</div>
			<template v-if="data">
				<div
					v-if="rules.length === 0"
					class="universal-card m-0 flex h-[102px] flex-col justify-center gap-2 text-center"
				>
					<h2 class="m-0 text-lg font-bold text-contrast">No project flag rules</h2>
					<p class="m-0 text-secondary">Create a rule to get started.</p>
				</div>
				<ProjectFlagRuleCard
					v-for="rule in rules"
					:key="rule.id"
					:id="`rule-${rule.id}`"
					:rule="rule"
					:delete-disabled="ruleToDelete?.id === rule.id"
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
							<p class="m-0 text-contrast">Delete “{{ rule.name }}”?</p>
							<p class="m-0 text-sm text-secondary">
								Existing flags will be preserved without a link to this rule after deletion.
							</p>
							<div class="flex justify-end gap-2">
								<Button @click="ruleToDelete = null">Cancel</Button>
								<Button type="colored" color="red" @click="removeRule(rule)">Delete rule</Button>
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
const { data, isPending, isError, error, isFetching, refetch, rules, deleteRule } = useRules(
	() => props.active,
)

watch(
	() => route.query.rule,
	(rule) => {
		if (rule) ruleToDelete.value = null
	},
)

function removeRule(rule: ProjectFlagRule) {
	if (ruleToDelete.value?.id !== rule.id) return
	ruleToDelete.value = null
	deleteRule({ id: rule.id, name: rule.name })
}

function confirmDelete(rule: ProjectFlagRule) {
	ruleToDelete.value = rule
}

useFlagErrorNotification(error, 'Could not load project flag rules')
</script>
