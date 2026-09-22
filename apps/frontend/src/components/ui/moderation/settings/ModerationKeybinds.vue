<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import ModerationKeybind from '~/components/ui/moderation/settings/ModerationKeybind.vue'

defineProps<{
	embedded?: boolean
}>()

const keybinds = useModerationKeybinds()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	general: {
		id: 'moderation.keybinds.general',
		defaultMessage: 'General',
	},
	projectReview: {
		id: 'moderation.keybinds.project-review',
		defaultMessage: 'Project review page',
	},
})
const groups = computed(() => {
	const bindings = [...keybinds.value]
	const isProjectReview = (scope: string) =>
		scope === 'project-review' || scope === 'review-conversation'
	return [
		{
			id: 'project-review',
			title: formatMessage(messages.projectReview),
			bindings: bindings.filter(([, binding]) => isProjectReview(binding.scope)),
		},
		{
			id: 'general',
			title: formatMessage(messages.general),
			bindings: bindings.filter(([, binding]) => !isProjectReview(binding.scope)),
		},
	]
})
</script>

<template>
	<div :class="{ 'universal-card': !embedded }">
		<h2 class="text-2xl">Keybinds</h2>
		<section v-for="group in groups" :key="group.id" class="mt-6">
			<h3 class="mb-4 text-lg font-semibold">{{ group.title }}</h3>
			<div class="grid grid-cols-2 gap-x-12 gap-y-4">
				<ModerationKeybind
					v-for="[id, keybind] in group.bindings"
					:key="id"
					:title="keybind.description"
					:scope="keybind.scope"
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
		</section>
	</div>
</template>
