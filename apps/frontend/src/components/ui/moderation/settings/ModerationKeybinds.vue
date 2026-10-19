<script setup lang="ts">
import { SearchIcon, TriangleAlertIcon } from '@modrinth/assets'
import { formatKeybind } from '@modrinth/moderation'
import { defineMessages, Input, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ModerationKeybind from '~/components/ui/moderation/settings/ModerationKeybind.vue'
import { saveModerationOptions, useModerationKeybinds } from '~/composables/moderation'

defineProps<{
	embedded?: boolean
}>()

const search = ref('')
const keybinds = useModerationKeybinds()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	search: { id: 'moderation.keybinds.search', defaultMessage: 'Search keybinds…' },
	noResults: {
		id: 'moderation.keybinds.no-results',
		defaultMessage: 'No keybinds match your search.',
	},
	keybinds: { id: 'moderation.keybinds.title', defaultMessage: 'Keybinds' },
	navigation: {
		id: 'moderation.keybinds.navigation',
		defaultMessage: 'Review navigation and panels',
	},
	items: {
		id: 'moderation.keybinds.items',
		defaultMessage: 'Review items and checks',
	},
	decisions: {
		id: 'moderation.keybinds.decisions',
		defaultMessage: 'Review outcomes and queue',
	},
	composer: {
		id: 'moderation.keybinds.composer',
		defaultMessage: 'Thread replies and private notes',
	},
	project: {
		id: 'moderation.keybinds.project',
		defaultMessage: 'Copy from public project pages',
	},
	global: {
		id: 'moderation.keybinds.global',
		defaultMessage: 'Switch hosts and copy URLs',
	},
	technical: {
		id: 'moderation.keybinds.technical',
		defaultMessage: 'Technical review pages',
	},
	conflicts: {
		id: 'moderation.keybinds.conflicts',
		defaultMessage: 'Overlaps with: {actions}',
	},
	selfConflict: {
		id: 'moderation.keybinds.self-conflict',
		defaultMessage: 'another binding for this action',
	},
	conflictSummary: {
		id: 'moderation.keybinds.conflict-summary',
		defaultMessage:
			'{count, plural, one {# action has overlapping bindings.} other {# actions have overlapping bindings.}} These shortcuts may compete when available together. Change a binding, or press Escape while recording to remove it.',
	},
})
const navigationIds = [
	'review-toggle-icon',
	'review-cycle-conversation',
	'review-toggle-left',
	'review-toggle-right',
]
const decisionIds = [
	'approve-project',
	'withhold-project',
	'reject-project',
	'review-back',
	'skip-project',
	'review-exit',
]
const bindings = computed(() => [...keybinds.value])
const groups = computed(() => {
	const categories = [
		{
			id: 'navigation',
			title: messages.navigation,
			bindings: bindings.value.filter(
				([id, binding]) => binding.scope === 'project-review' || navigationIds.includes(id),
			),
		},
		{
			id: 'items',
			title: messages.items,
			bindings: bindings.value.filter(
				([id, binding]) =>
					binding.scope === 'review-actions' &&
					!navigationIds.includes(id) &&
					!decisionIds.includes(id),
			),
		},
		{
			id: 'decisions',
			title: messages.decisions,
			bindings: bindings.value.filter(([id]) => decisionIds.includes(id)),
		},
		{
			id: 'composer',
			title: messages.composer,
			bindings: bindings.value.filter(
				([, binding]) =>
					binding.scope === 'review-composer' || binding.scope === 'review-conversation',
			),
		},
		{
			id: 'project',
			title: messages.project,
			bindings: bindings.value.filter(([, binding]) => binding.scope === 'project'),
		},
		{
			id: 'global',
			title: messages.global,
			bindings: bindings.value.filter(([, binding]) => binding.scope === 'global'),
		},
		{
			id: 'technical',
			title: messages.technical,
			bindings: bindings.value.filter(([, binding]) => binding.scope === 'tech-review'),
		},
	]
	return categories.filter((group) => group.bindings.length)
})
const filteredGroups = computed(() => {
	const query = search.value.trim().toLocaleLowerCase()
	return groups.value
		.map((group) => ({
			...group,
			bindings: group.bindings.filter(([, binding]) =>
				[
					formatMessage(group.title),
					binding.description,
					...binding.keybind.flatMap((definition) => [
						formatKeybind(definition, false),
						formatKeybind(definition, true),
					]),
				].some((text) => text.toLocaleLowerCase().includes(query)),
			),
		}))
		.filter((group) => group.bindings.length)
})
const conflicts = computed(() => keybinds.value.getConflicts())
const conflictLabels = computed(() => {
	const descriptions = new Map(bindings.value.map(([id, binding]) => [id, binding.description]))
	return new Map(
		[...conflicts.value].map(([id, others]) => [
			id,
			[...others]
				.map((otherId) =>
					otherId === id ? formatMessage(messages.selfConflict) : descriptions.get(otherId),
				)
				.join(', '),
		]),
	)
})
</script>

<template>
	<div :class="{ 'universal-card': !embedded }">
		<h2 class="m-0 mb-2.5 text-2xl">{{ formatMessage(messages.keybinds) }}</h2>
		<Input
			v-model="search"
			type="search"
			:icon="SearchIcon"
			:placeholder="formatMessage(messages.search)"
			:aria-label="formatMessage(messages.search)"
			class="w-full"
		/>
		<p v-if="!filteredGroups.length" class="text-secondary" role="status">
			{{ formatMessage(messages.noResults) }}
		</p>
		<p v-if="conflicts.size" class="flex items-start gap-2 text-sm text-orange" role="status">
			<TriangleAlertIcon class="size-5 shrink-0" aria-hidden="true" />
			{{ formatMessage(messages.conflictSummary, { count: conflicts.size }) }}
		</p>
		<section v-for="group in filteredGroups" :key="group.id" class="mt-6">
			<h3 class="mb-4 text-lg font-semibold">
				{{ formatMessage(group.title) }}
			</h3>
			<div class="flex flex-col gap-4">
				<div v-for="[id, keybind] in group.bindings" :key="id" class="min-w-0">
					<ModerationKeybind
						:title="keybind.description"
						:scope="keybind.scope"
						:conflict-message="
							conflictLabels.has(id)
								? formatMessage(messages.conflicts, { actions: conflictLabels.get(id) })
								: undefined
						"
						:definitions="keybind.keybind"
						:default="keybind.defaultKeybind"
						:on-change="
							(definitions) => {
								keybinds.set(id, definitions)
								saveModerationOptions()
							}
						"
					/>
				</div>
			</div>
		</section>
	</div>
</template>
