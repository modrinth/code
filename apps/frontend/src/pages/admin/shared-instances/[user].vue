<template>
	<SharedInstanceInspectorModal
		ref="inspectorModal"
		@hide="clearInspectorQuery"
		@instance-change="setInspectorQuery"
	/>
	<SharedInstanceBanModal ref="banModal" :username="username" subject="user" />

	<div class="normal-page no-sidebar !pb-6">
		<div class="normal-page__content flex flex-col gap-4">
			<header
				class="flex flex-col gap-3 border-0 border-b border-solid border-divider pb-4 sm:flex-row sm:items-center sm:justify-between"
			>
				<div class="flex min-w-0 items-center gap-3">
					<Avatar
						:src="user?.avatar_url"
						:alt="username"
						:tint-by="user?.id ?? userId"
						size="48px"
						circle
					/>
					<h1 class="m-0 min-w-0 truncate text-2xl font-extrabold text-contrast">
						{{ username }}'s shared instances
					</h1>
				</div>

				<div class="flex flex-wrap gap-2">
					<ButtonLink :to="`/user/${user?.id ?? userId}`" target="_blank">
						<UserIcon />
						User profile
						<ExternalIcon class="size-4" />
					</ButtonLink>
					<Button
						v-if="blacklist?.blacklisted"
						type="outlined"
						:loading="actionPending"
						@click="unbanUser(userSummary)"
					>
						<UndoIcon />
						Unban from shared instances
					</Button>
					<Button
						v-else
						type="outlined"
						color="red"
						:disabled="!blacklist"
						:loading="actionPending"
						@click="promptBan"
					>
						<BanIcon />
						Ban from shared instances
					</Button>
				</div>
			</header>

			<Admonition
				v-if="userError"
				type="critical"
				header="Failed to load user"
				:body="getRequestErrorMessage(userError, 'The user could not be loaded.')"
				show-actions-underneath
			>
				<template #actions>
					<Button :disabled="userFetching" @click="refetchUser()">
						<LoaderCircleIcon v-if="userFetching" class="animate-spin" />
						Try again
					</Button>
				</template>
			</Admonition>

			<SharedInstanceUserInstancesPanel v-else :user-id="userId" @inspect="openInspector" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { BanIcon, ExternalIcon, LoaderCircleIcon, UndoIcon, UserIcon } from '@modrinth/assets'
import { Admonition, Avatar, Button, ButtonLink, injectModrinthClient } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, onMounted, onServerPrefetch, ref } from 'vue'

import SharedInstanceInspectorModal from '~/components/ui/moderation/shared-instance-inspector/index.vue'
import SharedInstanceBanModal from '~/components/ui/moderation/SharedInstanceBanModal.vue'
import SharedInstanceUserInstancesPanel from '~/components/ui/moderation/SharedInstanceUserInstancesPanel.vue'
import { sharedInstanceQueryOptions } from '~/composables/queries/shared-instance'
import { useSharedInstanceModeration } from '~/composables/shared-instance-moderation'
import { getRequestErrorMessage } from '~/helpers/shared-instance-moderation'

const route = useRoute()
const router = useRouter()
const client = injectModrinthClient()
const { pending: actionPending, banUser, unbanUser } = useSharedInstanceModeration()

const userId = computed(() => String(route.params.user ?? ''))
const canLoadUser = computed(() => userId.value.length > 0)
const inspectorModal = ref<InstanceType<typeof SharedInstanceInspectorModal> | null>(null)
const banModal = ref<InstanceType<typeof SharedInstanceBanModal> | null>(null)

const {
	data: user,
	error: userError,
	isFetching: userFetching,
	refetch: refetchUser,
	suspense: userSuspense,
} = useQuery({
	queryKey: computed(() => ['user', 'v3', userId.value] as const),
	queryFn: () => client.labrinth.users_v3.get(userId.value),
	enabled: canLoadUser,
})

const { data: blacklist } = useQuery(
	computed(() => ({
		...sharedInstanceQueryOptions.blacklist(userId.value, client),
		enabled: canLoadUser.value,
	})),
)

const username = computed(() => user.value?.username ?? userId.value)
const userSummary = computed(() => ({
	id: user.value?.id ?? userId.value,
	username: username.value,
}))

useHead({
	title: computed(() => `${username.value}'s shared instances - Modrinth`),
})

onServerPrefetch(userSuspense)

onMounted(() => {
	const instanceId = route.query.instance
	if (typeof instanceId !== 'string' || !instanceId) return
	const version = Number(route.query.version)
	inspectorModal.value?.show(instanceId, {
		version: Number.isInteger(version) ? version : undefined,
	})
})

function openInspector(instanceId: string, reportedVersions: number[]) {
	inspectorModal.value?.show(instanceId, { reportedVersions })
	setInspectorQuery(instanceId)
}

function setInspectorQuery(instanceId: string) {
	void router.replace({ query: { ...route.query, instance: instanceId, version: undefined } })
}

function clearInspectorQuery() {
	if (route.query.instance === undefined) return
	void router.replace({ query: { ...route.query, instance: undefined, version: undefined } })
}

async function promptBan() {
	if (!(await banModal.value?.prompt())) return
	await banUser(userSummary.value)
}
</script>
