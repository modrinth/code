<template>
	<div>
		<!--
		<section
			class="universal-card mb-4 flex flex-wrap items-center justify-between gap-3"
			aria-label="Rules deployment"
		>
			<div class="min-w-0">
				<h2 class="m-0 text-base font-semibold text-contrast">Live ruleset</h2>
				<p v-if="deployment" class="m-0 break-all text-sm text-secondary">
					<code>{{ deployment.ruleset_id }}</code> · {{ formatDate(deployment.deployed_at, true) }}
					<NuxtLink
						v-if="deployment.deployed_by"
						:to="`/user/${deployment.deployed_by}`"
						class="text-link"
					>
						{{ deployment.deployed_by }}
					</NuxtLink>
				</p>
				<p v-else class="m-0 text-sm text-secondary">
					{{
						deploymentPending
							? 'Loading deployment…'
							: deploymentError
								? 'Deployment unavailable.'
								: 'No rules deployed.'
					}}
				</p>
			</div>
			<Button disabled>Deploy rules</Button>
		</section>
		-->
		<ProjectFlagRules :active="activeTab === 'rules'">
			<template #tabs>
				<Tabs
					:tabs="tabs"
					:value="activeTab"
					color="gray"
					wrap
					class="[&_[role=tab][aria-selected=true]>svg]:text-brand [&_[role=tab][aria-selected=true]]:border-brand [&_[role=tab][aria-selected=true]]:bg-brand-highlight [&_[role=tab][aria-selected=true]]:text-brand"
					@update:value="switchTab"
				/>
			</template>
			<ProjectFlagRunHistory v-if="activeTab === 'runs'" />
		</ProjectFlagRules>
		<ProjectFlagRuleEditor />
	</div>
</template>

<script setup lang="ts">
import { HistoryIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Tabs, type TabsValue } from '@modrinth/ui'

import ProjectFlagRules from '~/components/ui/moderation/project-flag/rules/index.vue'
import ProjectFlagRuleEditor from '~/components/ui/moderation/project-flag/rules/rule-editor/overlay.vue'
import ProjectFlagRunHistory from '~/components/ui/moderation/project-flag/run-history.vue'

useHead({ title: 'Project flag rules - Modrinth' })

const route = useRoute()
const router = useRouter()
const activeTab = computed(() => (route.query.tab === 'runs' ? 'runs' : 'rules'))
const tabs = [
	{ label: 'Flag rules', value: 'rules', icon: TagCategoryFlagIcon },
	{ label: 'Flag rule runs', value: 'runs', icon: HistoryIcon },
]

function switchTab(tab: TabsValue) {
	return router.push({
		query: { ...route.query, tab: tab === 'runs' ? tab : undefined },
	})
}
</script>
