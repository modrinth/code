<template>
	<div>
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
			<Button :disabled="isDeploying" @click="deployModal?.show()">Deploy current staging</Button>
		</section>
		<NewModal ref="deployModal" header="Deploy current staging" :disable-close="isDeploying">
			<p class="m-0 max-w-lg">
				Deploy all saved staging rules and shared modules to live? This includes additions, edits,
				and deletions. No test scan will run.
			</p>
			<template #actions>
				<Button :disabled="isDeploying" @click="deployModal?.hide()">Cancel</Button>
				<Button
					type="colored"
					color="brand"
					:loading="isDeploying"
					:disabled="isDeploying"
					@click="deployStaging()"
				>
					Deploy
				</Button>
			</template>
		</NewModal>
		<ProjectFlagRules :active="activeTab === 'rules'">
			<template #tabs>
				<Tabs
					:tabs="tabs"
					:value="activeTab"
					color="gray"
					wrap
					class="[&>button[aria-selected=true]>svg]:text-brand [&>button[aria-selected=true]]:border-brand [&>button[aria-selected=true]]:bg-brand-highlight [&>button[aria-selected=true]]:text-brand"
					@update:value="switchTab"
				/>
			</template>
			<ProjectFlagRunHistory v-if="activeTab === 'runs'" />
			<ProjectFlagModules v-else-if="activeTab === 'modules'" />
		</ProjectFlagRules>
	</div>
</template>

<script setup lang="ts">
import { BoxIcon, HistoryIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import {
	Button,
	injectModrinthClient,
	injectNotificationManager,
	NewModal,
	Tabs,
	type TabsValue,
} from '@modrinth/ui'

import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import ProjectFlagModules from '~/components/ui/moderation/project-flag/ProjectFlagModules.vue'
import { formatDate } from '~/components/ui/moderation/project-flag/run-format'
import { useFlagErrorNotification } from '~/components/ui/moderation/project-flag/use-flag-error-notification'
import { projectFlagDeploymentQueryOptions } from '~/composables/queries/project-flag'
import ProjectFlagRules from '~/components/ui/moderation/project-flag/rules/index.vue'
import ProjectFlagRunHistory from '~/components/ui/moderation/project-flag/run-history.vue'

useHead({ title: 'Project flag rules - Modrinth' })

const route = useRoute()
const router = useRouter()
const activeTab = computed(() =>
	route.query.tab === 'runs' || route.query.tab === 'modules' ? route.query.tab : 'rules',
)
const tabs = [
	{ label: 'Flag rules', value: 'rules', icon: TagCategoryFlagIcon },
	{ label: 'Flag rule runs', value: 'runs', icon: HistoryIcon },
	{ label: 'Modules', value: 'modules', icon: BoxIcon },
]

const client = injectModrinthClient()
const queryClient = useQueryClient()
const { addNotification } = injectNotificationManager()
const deployModal = useTemplateRef<InstanceType<typeof NewModal>>('deployModal')
const deploymentOptions = projectFlagDeploymentQueryOptions(client)
const {
	data: deployment,
	isPending: deploymentPending,
	error: deploymentError,
} = useQuery(deploymentOptions)
const {
	mutate: deployStaging,
	isPending: isDeploying,
	isSuccess: deployed,
	error: deployError,
} = useMutation({
	mutationFn: () => client.labrinth.project_flags_internal.deployStaging(),
	retry: false,
	onSuccess: async (live) => {
		queryClient.setQueryData(deploymentOptions.queryKey, live)
		addNotification({ type: 'success', title: 'Staging rules deployed' })
		await queryClient.invalidateQueries({ queryKey: ['moderation', 'project-flag'] })
	},
})
useFlagErrorNotification(deploymentError, 'Could not load deployment status')
watch(
	deployed,
	(success) => {
		if (success) deployModal.value?.hide()
	},
	{ flush: 'post' },
)
useFlagErrorNotification(deployError, 'Could not deploy staging')

function switchTab(tab: TabsValue) {
	return router.push({
		query: { ...route.query, tab: tab === 'runs' || tab === 'modules' ? tab : undefined },
	})
}
</script>
