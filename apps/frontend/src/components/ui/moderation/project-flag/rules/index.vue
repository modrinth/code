<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<slot name="tabs" />
		</div>
		<div v-if="active" class="flex flex-wrap items-center gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="search"
				input-class="[&::-webkit-search-cancel-button]:hidden"
				aria-label="Search flag rules"
				placeholder="Search rules by name or ID…"
				autocomplete="off"
				wrapper-class="min-w-0 grow basis-full sm:max-w-md sm:basis-48"
				clearable
			/>
			<ButtonLink
				type="colored"
				color="brand"
				class="ml-auto max-w-full shrink-0"
				:to="{ query: { ...$route.query, rule: 'new' }, hash: $route.hash }"
			>
				<PlusIcon />
				Create rule
			</ButtonLink>
		</div>

		<slot />
		<div v-show="active" class="flex min-w-0 flex-col gap-2.5">
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
				<p
					v-else-if="filteredRules.length === 0"
					role="status"
					class="universal-card m-0 flex h-[102px] items-center justify-center text-center text-secondary"
				>
					No rules match your search.
				</p>
				<ProjectFlagRuleCard
					v-for="rule in filteredRules"
					:key="rule.id"
					:id="`rule-${rule.id}`"
					:rule="rule"
					:delete-disabled="ruleToDelete?.id === rule.id"
					@request-delete="confirmDelete(rule)"
				>
					<template #delete-confirmation>
						<div
							v-if="ruleToDelete?.id === rule.id"
							class="mx-4 mb-4 flex cursor-auto flex-col gap-3 rounded-xl border border-solid border-surface-4 p-3"
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
import { LoaderCircleIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import { Button, ButtonLink, Input } from '@modrinth/ui'
import Fuse from 'fuse.js'

import ProjectFlagRuleCard from './rule-card.vue'
import { useRules } from './use-rules'

const props = defineProps<{ active: boolean }>()
const route = useRoute()
const ruleToDelete = ref<ProjectFlagRule | null>(null)
const { data, isPending, isError, error, isFetching, refetch, rules, deleteRule } = useRules(
	() => props.active,
)
const search = ref<string>()
const ruleSearch = computed(
	() => new Fuse(rules.value, { keys: ['name', 'id'], threshold: 0.3, ignoreLocation: true }),
)
const filteredRules = computed(() => {
	const term = search.value?.trim()
	return term ? ruleSearch.value.search(term).map(({ item }) => item) : rules.value
})

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
