<template>
	<ConfirmModal
		ref="unlockModal"
		title="Unlock server?"
		:description="`The owner and members of ${unlockingServerId} will regain write access.`"
		:proceed-icon="LockOpenIcon"
		proceed-label="Unlock server"
		@proceed="confirmUnlock"
	/>
	<div>
		<h2 class="m-0 mb-4 text-2xl font-semibold">Server locks</h2>
		<div v-if="isLoading" class="py-8 text-center text-secondary">Loading locks...</div>
		<Admonition v-else-if="error" type="critical" header="Failed to load server locks">
			{{ error.message }}
		</Admonition>
		<div v-else-if="!locks || locks.length === 0" class="py-8 text-center text-secondary">
			No servers are locked.
		</div>
		<div v-else class="flex flex-col gap-3">
			<div
				v-for="lock in locks"
				:key="lock.server_id"
				class="relative overflow-clip rounded-xl bg-bg-raised p-4"
			>
				<div class="absolute bottom-0 left-0 top-0 w-1 bg-red" />
				<div class="ml-2 flex flex-col gap-2">
					<div class="flex items-center justify-between gap-4">
						<div class="flex items-center gap-3">
							<CopyCode :text="lock.server_id" />
							<nuxt-link
								:to="`/hosting/manage/${lock.server_id}`"
								class="flex items-center gap-1 text-sm text-link hover:underline"
							>
								Open panel
								<ExternalIcon />
							</nuxt-link>
						</div>
						<Button
							type="quiet"
							color="red"
							class="!text-red [&>svg]:!text-red"
							@click="showUnlockModal(lock.server_id)"
						>
							<LockOpenIcon />
							Unlock
						</Button>
					</div>
					<div class="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-secondary">
						<span v-tooltip="formatDateTime(lock.created)">
							Locked {{ formatRelativeTime(lock.created) }}
						</span>
						<template v-if="lock.locked_by">
							<span>by</span>
							<nuxt-link
								:to="`/user/${userMap.get(lock.locked_by)?.username ?? lock.locked_by}`"
								class="flex items-center gap-1 font-semibold text-contrast hover:underline"
							>
								<Avatar
									:src="userMap.get(lock.locked_by)?.avatar_url"
									:alt="userMap.get(lock.locked_by)?.username"
									size="20px"
									circle
								/>
								{{ userMap.get(lock.locked_by)?.username ?? lock.locked_by }}
							</nuxt-link>
						</template>
					</div>
					<div class="text-sm">
						<span class="text-secondary">Reason:</span>
						<span class="ml-1 whitespace-pre-wrap text-contrast">{{ lock.reason }}</span>
					</div>
				</div>
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
import { ExternalIcon, LockOpenIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	ConfirmModal,
	CopyCode,
	injectModrinthClient,
	injectNotificationManager,
	useFormatDateTime,
	useRelativeTime,
} from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})

const unlockModal = ref<InstanceType<typeof ConfirmModal>>()
const unlockingServerId = ref<string | null>(null)

const {
	data: locks,
	error,
	isLoading,
} = useQuery({
	queryKey: ['servers', 'locks'],
	queryFn: () => client.archon.servers_internal.getLocks(),
})

const lockerIds = computed(() => [
	...new Set((locks.value ?? []).flatMap((lock) => (lock.locked_by ? [lock.locked_by] : []))),
])

const { data: lockers } = useQuery({
	queryKey: computed(() => ['users', 'multiple', lockerIds.value]),
	queryFn: () => client.labrinth.users_v2.getMultiple(lockerIds.value),
	enabled: computed(() => lockerIds.value.length > 0),
})

const userMap = computed(() => new Map((lockers.value ?? []).map((user) => [user.id, user])))

function showUnlockModal(serverId: string) {
	unlockingServerId.value = serverId
	unlockModal.value?.show()
}

async function confirmUnlock() {
	const serverId = unlockingServerId.value
	if (!serverId) return

	try {
		await client.archon.servers_internal.unlock(serverId)
		addNotification({
			title: 'Server unlocked',
			text: `${serverId} has been unlocked.`,
			type: 'success',
		})
		unlockingServerId.value = null
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'locks'] }),
			queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] }),
		])
	} catch (err) {
		addNotification({
			title: 'Error unlocking server',
			text: err instanceof Error ? err.message : String(err),
			type: 'error',
		})
	}
}
</script>
