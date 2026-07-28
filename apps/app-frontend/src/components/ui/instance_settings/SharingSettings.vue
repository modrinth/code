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
						{{ formatMessage(messages.noActiveInvites) }}
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
						<ButtonStyled circular type="transparent">
							<button
								v-tooltip="formatMessage(messages.revokeInvite)"
								:aria-label="
									formatMessage(messages.revokeInviteWithCode, {
										code: row.id,
									})
								"
								class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
								@click="revokeInviteModal?.show(row.id)"
							>
								<XIcon aria-hidden="true" />
							</button>
						</ButtonStyled>
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
import { XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	CopyCode,
	defineMessages,
	Table,
	type TableColumn,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ConfirmRevokeSharedInstanceInviteModal from '@/components/ui/shared-instances/ConfirmRevokeSharedInstanceInviteModal.vue'
import SharedInstanceInstallationSettingsControls from '@/components/ui/shared-instances/SharedInstanceInstallationSettingsControls.vue'
import { config } from '@/config'
import { type SharedInstanceInvite, unpublish_shared_instance } from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import { injectInstanceSettings } from '@/providers/instance-settings'

const { instance, offline, onUnlinked } = injectInstanceSettings()
const { notifySharedInstanceError } = useSharedInstanceErrors()
const { formatMessage } = useVIntl()
const queryClient = useQueryClient()
const unpublishing = ref(false)
const revokeInviteModal = ref<InstanceType<typeof ConfirmRevokeSharedInstanceInviteModal>>()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })
const isBusy = computed(
	() => instance.value.install_stage !== 'installed' || unpublishing.value || !!offline,
)

type InviteTableColumn = 'id' | 'uses' | 'expiration' | 'actions'

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

const now = Date.now()

// TODO: Use actual endpoint
const activeInvites = ref<SharedInstanceInvite[]>([
	{
		id: 'wqHPxNagZr',
		expiration: new Date(now + 6 * 24 * 60 * 60 * 1000).toISOString(),
		maxUses: 10,
		uses: 0,
	},
	{
		id: 'GbRGfY7hbs',
		expiration: new Date(now + 3 * 24 * 60 * 60 * 1000).toISOString(),
		maxUses: 10,
		uses: 2,
	},
	{
		id: 'k9mvD2QxLc',
		expiration: new Date(now + 90 * 60 * 1000).toISOString(),
		maxUses: 5,
		uses: 4,
	},
])

function revokeInvite(inviteId: string) {
	activeInvites.value = activeInvites.value.filter((invite) => invite.id !== inviteId)
}

async function unpublishSharedInstance() {
	unpublishing.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		queryClient.setQueryData(['sharedInstanceUsers', instance.value.id], [])
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
