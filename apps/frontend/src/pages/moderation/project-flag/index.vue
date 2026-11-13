<template>
	<ProjectFlagRules :active="!showRuns">
		<template #tabs>
			<Tabs
				:tabs="tabs"
				:value="showRuns ? 'runs' : 'rules'"
				color="gray"
				wrap
				class="[&>button[aria-selected=true]>svg]:text-brand [&>button[aria-selected=true]]:border-brand [&>button[aria-selected=true]]:bg-brand-highlight [&>button[aria-selected=true]]:text-brand"
				@update:value="switchTab"
			/>
		</template>
		<ProjectFlagRunHistory v-if="showRuns" />
	</ProjectFlagRules>
</template>

<script setup lang="ts">
import { HistoryIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Tabs, type TabsValue } from '@modrinth/ui'

import ProjectFlagRules from '~/components/ui/moderation/project-flag/rules/index.vue'
import ProjectFlagRunHistory from '~/components/ui/moderation/project-flag/run-history.vue'

useHead({ title: 'Project flag rules - Modrinth' })

const route = useRoute()
const router = useRouter()
const showRuns = computed(() => route.query.tab === 'runs')
const tabs = [
	{ label: 'Flag rules', value: 'rules', icon: TagCategoryFlagIcon },
	{ label: 'Flag rule runs', value: 'runs', icon: HistoryIcon },
]

function switchTab(tab: TabsValue) {
	return router.push({ query: { ...route.query, tab: tab === 'runs' ? 'runs' : undefined } })
}
</script>
