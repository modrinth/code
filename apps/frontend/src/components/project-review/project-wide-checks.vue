<template>
	<div v-if="visibleTargets.length" class="flex min-h-0 min-w-0 flex-1 flex-col gap-4 p-0">
		<template v-if="stacked">
			<ReviewPanel
				v-for="target in visibleTargets"
				:key="target"
				mode="inline"
				inline-title
				class="min-w-0 shrink-0"
				:target="{ kind: 'panel', key: target }"
			/>
		</template>
		<template v-else>
			<Tabs
				:value="selectedTarget"
				:tabs="tabs"
				wrap
				class="max-w-full shrink-0"
				@update:value="selectedTarget = $event as ActionTarget"
			/>
			<ReviewPanel
				v-if="selectedTarget !== undefined && visibleTargets.includes(selectedTarget)"
				:key="selectedTarget"
				mode="inline"
				inline-title
				class="min-w-0 shrink-0"
				:target="{ kind: 'panel', key: selectedTarget }"
			/>
		</template>
	</div>
	<div v-else class="min-h-0 flex-1" />
</template>

<script setup lang="ts">
import { moderationSettings } from '@modrinth/moderation'
import { type MiscReviewPanelKey, reviewPanels } from '@modrinth/moderation/src/data/issues'
import { Tabs } from '@modrinth/ui'
import { computed, onScopeDispose, ref, watch } from 'vue'

import { useModerationSettings } from '~/composables/moderation'
import { injectReviewPanels } from '~/providers/project-review/review-panels'

import { injectProjectReviewContext } from './layout/context'
import { injectReviewContext } from './review-panel/context'
import ReviewPanel from './review-panel/index.vue'
import { useReviewShortcut } from './shortcuts'

/**
 * The rulings, i.e. the project-wide checks (re-review, post-approval, reupload, rule following):
 * one at a time by tabs, or all of them at once as the settings choose.
 */
const { resolve } = injectReviewPanels()
const settings = useModerationSettings()
const stacked = computed(() => settings.value.get(moderationSettings.General.MergeRulings))
type ActionTarget = MiscReviewPanelKey
const selectedTarget = ref<ActionTarget | undefined>(reviewPanels.misc[0]?.key)
const { revealSlot } = injectProjectReviewContext()
const { registerRoute } = injectReviewContext()
for (const { key: target } of reviewPanels.misc) {
	useReviewShortcut(
		target,
		() => {
			selectedTarget.value = target
			revealSlot('rulings')
		},
		() => visibleTargets.value.includes(target),
	)
	onScopeDispose(
		registerRoute(target, () => {
			selectedTarget.value = target
		}),
	)
}
const visiblePanels = computed(() =>
	reviewPanels.misc.filter(({ key }) => resolve({ kind: 'panel', key })),
)
const visibleTargets = computed(() => visiblePanels.value.map(({ key }) => key))
const tabs = computed(() => visiblePanels.value.map(({ key, label }) => ({ value: key, label })))

useReviewShortcut(
	'cycle-misc',
	() => {
		const targets = visibleTargets.value
		const currentIndex =
			selectedTarget.value === undefined ? -1 : targets.indexOf(selectedTarget.value)
		selectedTarget.value = targets[(currentIndex + 1) % targets.length]
		revealSlot('rulings')
	},
	() => visibleTargets.value.length > 0,
)

watch(
	visibleTargets,
	(visible) => {
		if (selectedTarget.value === undefined || !visible.includes(selectedTarget.value)) {
			selectedTarget.value = visible[0]
		}
	},
	{ immediate: true },
)
</script>
