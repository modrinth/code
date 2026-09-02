<template>
	<div class="flex flex-col gap-8">
		<section class="flex flex-col gap-4">
			<div class="flex flex-col gap-1">
				<h3 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.activeInvitesTitle) }}
				</h3>
				<p class="m-0 text-secondary">
					{{ formatMessage(messages.activeInvitesDescription) }}
				</p>
			</div>

			<Table :columns="inviteColumns" :data="activeInvites" row-key="id" table-min-width="36rem">
				<template #empty-state>
					<div class="flex h-40 items-center justify-center text-secondary">
						<SpinnerIcon
							v-if="activeInvitesQuery.isLoading.value"
							class="animate-spin"
							aria-hidden="true"
						/>
						<template v-else>
							{{ formatMessage(messages.noActiveInvites) }}
						</template>
					</div>
				</template>
				<template #cell-id="{ row }">
					<CopyCode
						:text="`${config.siteUrl}/share/${encodeURIComponent(row.id)}`"
						:display-text="`/${row.id}`"
					/>
				</template>
				<template #cell-uses="{ row }">
					<span class="font-medium text-primary">{{ row.uses }}</span>
					<span> / {{ row.maxUses }}</span>
				</template>
				<template #cell-expiration="{ row }">
					<span v-tooltip="formatDateTime(row.expiration)" class="whitespace-nowrap">
						{{ formatRelativeTime(row.expiration) }}
					</span>
				</template>
				<template #cell-actions="{ row }">
					<div class="flex justify-end">
						<IconButton
							v-tooltip="formatMessage(messages.revokeInvite)"
							type="quiet"
							:label="
								formatMessage(messages.revokeInviteWithCode, {
									code: row.id,
								})
							"
							:disabled="revokeInviteMutation.isPending.value || isBusy"
							class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
							@click="revokeInviteModal?.show(row.id)"
						>
							<SpinnerIcon
								v-if="
									revokeInviteMutation.isPending.value &&
									revokeInviteMutation.variables.value?.inviteId === row.id
								"
								class="animate-spin"
								aria-hidden="true"
							/>
							<XIcon v-else aria-hidden="true" />
						</IconButton>
					</div>
				</template>
			</Table>
		</section>

		<SharedInstanceInstallationSettingsControls
			can-unpublish
			:busy="isBusy"
			:unpublishing="unpublishing"
			:unpublish="unpublishSharedInstance"
		/>

		<ConfirmRevokeSharedInstanceInviteModal ref="revokeInviteModal" @revoke="revokeInvite" />
	</div>
</template>

<script setup lang="ts">
import { SpinnerIcon, XIcon } from '@modrinth/assets'
import { IconButton } from '@modrinth/ui'
import {
	CopyCode,
	defineMessages,
	Table,
	type TableColumn,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { config } from '@/config'
import {
	get_shared_instance_invites,
	revoke_shared_instance_invite,
	type SharedInstanceInvite,
	unpublish_shared_instance,
} from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'

import { instanceKeys } from '../../query-options.ts'
import ConfirmRevokeSharedInstanceInviteModal from './confirm-revoke-shared-instance-invite-modal.vue'
import { injectInstanceSettings } from './instance-settings-context.ts'
import SharedInstanceInstallationSettingsControls from './shared-instance-installation-settings-controls.vue'

const { instance, offline, onUnlinked } = injectInstanceSettings()
const { notifySharedInstanceError } = useSharedInstanceErrors()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const unpublishing = ref(false)
const revokeInviteModal = ref<InstanceType<typeof ConfirmRevokeSharedInstanceInviteModal>>()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

type InviteTableColumn = 'id' | 'uses' | 'expiration' | 'actions'
type ActiveInvitesQueryKey = readonly ['sharedInstanceInvites', string]

const activeInvitesQueryKey = computed(
	() => ['sharedInstanceInvites', instance.value.id] as const satisfies ActiveInvitesQueryKey,
)
const activeInvitesQuery = useQuery({
	queryKey: activeInvitesQueryKey,
	queryFn: async ({ queryKey }) => {
		try {
			return await get_shared_instance_invites(queryKey[1])
		} catch (error) {
			notifySharedInstanceError(error)
			throw error
		}
	},
	enabled: () => !!instance.value.id && !offline,
	retry: false,
	staleTime: Infinity,
	refetchOnMount: 'always',
	refetchOnReconnect: false,
	refetchOnWindowFocus: false,
})
const activeInvites = computed(() => activeInvitesQuery.data.value ?? [])

const revokeInviteMutation = useMutation({
	mutationFn: ({ instanceId, inviteId }: { instanceId: string; inviteId: string }) =>
		revoke_shared_instance_invite(instanceId, inviteId),
	onSuccess: (_data, { instanceId, inviteId }) => {
		queryClient.setQueryData<SharedInstanceInvite[]>(
			['sharedInstanceInvites', instanceId],
			(invites = []) => invites.filter((invite) => invite.id !== inviteId),
		)
	},
	onError: notifySharedInstanceError,
})

const isBusy = computed(
	() =>
		instance.value.install_stage !== 'installed' ||
		unpublishing.value ||
		revokeInviteMutation.isPending.value ||
		!!offline,
)

const inviteColumns = computed<TableColumn<InviteTableColumn>[]>(() => [
	{
		key: 'id',
		label: formatMessage(messages.inviteCodeLabel),
		width: 'clamp(11rem, 34%, 19rem)',
	},
	{
		key: 'uses',
		label: formatMessage(messages.usesLabel),
		width: 'clamp(7rem, 18%, 10rem)',
	},
	{
		key: 'expiration',
		label: formatMessage(messages.expiresLabel),
		width: 'clamp(9rem, 28%, 14rem)',
	},
	{
		key: 'actions',
		label: formatMessage(messages.actionsLabel),
		align: 'right',
		width: '5.5rem',
	},
])

function revokeInvite(inviteId: string) {
	if (revokeInviteMutation.isPending.value) return
	revokeInviteMutation.mutate({ instanceId: instance.value.id, inviteId })
}

async function unpublishSharedInstance() {
	unpublishing.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		queryClient.setQueryData(instanceKeys.sharedMembers(instance.value.id), [])
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		notifySharedInstanceError(error)
	} finally {
		unpublishing.value = false
	}
}

const messages = defineMessages({
	activeInvitesTitle: {
		id: 'instance.settings.sharing.active-invites.title',
		defaultMessage: 'Active invites',
	},
	activeInvitesDescription: {
		id: 'instance.settings.sharing.active-invites.description',
		defaultMessage: 'Anyone with one of these invite links can join while it remains active.',
	},
	inviteCodeLabel: {
		id: 'instance.settings.sharing.active-invites.code',
		defaultMessage: 'Invite link',
	},
	usesLabel: {
		id: 'instance.settings.sharing.active-invites.uses',
		defaultMessage: 'Uses',
	},
	expiresLabel: {
		id: 'instance.settings.sharing.active-invites.expires',
		defaultMessage: 'Expires',
	},
	actionsLabel: {
		id: 'instance.settings.sharing.active-invites.actions',
		defaultMessage: 'Actions',
	},
	noActiveInvites: {
		id: 'instance.settings.sharing.active-invites.empty',
		defaultMessage: 'There are no active invites.',
	},
	revokeInvite: {
		id: 'instance.settings.sharing.active-invites.revoke',
		defaultMessage: 'Revoke invite',
	},
	revokeInviteWithCode: {
		id: 'instance.settings.sharing.active-invites.revoke-with-code',
		defaultMessage: 'Revoke invite {code}',
	},
})
</script>
