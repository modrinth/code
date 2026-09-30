<script setup lang="ts">
import { SpinnerIcon, XIcon } from '@modrinth/assets'
import { computed } from 'vue'

import {
	Admonition,
	Button,
	CopyCode,
	IconButton,
	Table,
	type TableColumn,
} from '#ui/components/base'
import { useFormatDateTime, useRelativeTime } from '#ui/composables'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import { injectSharingSettings } from '../providers/sharing-settings'

defineProps<{
	siteUrl: string
	busy: boolean
	revokingId?: string
}>()
defineEmits<{ revoke: [inviteId: string] }>()

const ctx = injectSharingSettings()
const { formatMessage } = useVIntl()
const formatRelativeTime = useRelativeTime()
const formatDateTime = useFormatDateTime({ dateStyle: 'medium', timeStyle: 'short' })

async function refresh() {
	try {
		await ctx.refresh()
	} catch (error) {
		ctx.onError(error)
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
	inviteLink: {
		id: 'instance.settings.sharing.active-invites.code',
		defaultMessage: 'Invite link',
	},
	uses: { id: 'instance.settings.sharing.active-invites.uses', defaultMessage: 'Uses' },
	expires: { id: 'instance.settings.sharing.active-invites.expires', defaultMessage: 'Expires' },
	actions: { id: 'instance.settings.sharing.active-invites.actions', defaultMessage: 'Actions' },
	noInvites: {
		id: 'instance.settings.sharing.active-invites.empty',
		defaultMessage: 'There are no active invites.',
	},
	invitesError: {
		id: 'server.settings.sharing.invites-error',
		defaultMessage: 'Failed to load active invites',
	},
	retry: { id: 'server.settings.sharing.retry', defaultMessage: 'Retry' },
	revoke: {
		id: 'instance.settings.sharing.active-invites.revoke',
		defaultMessage: 'Revoke invite',
	},
	revokeWithCode: {
		id: 'instance.settings.sharing.active-invites.revoke-with-code',
		defaultMessage: 'Revoke invite {code}',
	},
})
const columns = computed<TableColumn<'id' | 'uses' | 'expiration' | 'actions'>[]>(() => [
	{ key: 'id', label: formatMessage(messages.inviteLink), width: 'clamp(11rem, 34%, 19rem)' },
	{ key: 'uses', label: formatMessage(messages.uses), width: 'clamp(7rem, 18%, 10rem)' },
	{ key: 'expiration', label: formatMessage(messages.expires), width: 'clamp(9rem, 28%, 14rem)' },
	{ key: 'actions', label: formatMessage(messages.actions), align: 'right', width: '5.5rem' },
])
</script>

<template>
	<section class="flex flex-col gap-4">
		<div class="flex flex-col gap-1">
			<h3 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.activeInvitesTitle) }}
			</h3>
			<p class="m-0 text-secondary">{{ formatMessage(messages.activeInvitesDescription) }}</p>
		</div>
		<Admonition
			v-if="ctx.error.value"
			type="critical"
			:header="formatMessage(messages.invitesError)"
		>
			<Button @click="refresh">{{ formatMessage(messages.retry) }}</Button>
		</Admonition>
		<Table v-else :columns="columns" :data="ctx.invites.value" row-key="id" table-min-width="36rem">
			<template #empty-state>
				<div class="flex h-40 items-center justify-center text-secondary">
					<SpinnerIcon
						v-if="ctx.loading.value"
						class="animate-spin"
						:aria-label="formatMessage(commonMessages.loadingLabel)"
						role="status"
					/>
					<template v-else>{{ formatMessage(messages.noInvites) }}</template>
				</div>
			</template>
			<template #cell-id="{ row }">
				<CopyCode
					:text="`${siteUrl.replace(/\/$/, '')}/share/${encodeURIComponent(row.id)}`"
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
						v-tooltip="formatMessage(messages.revoke)"
						type="quiet"
						:label="formatMessage(messages.revokeWithCode, { code: row.id })"
						:disabled="busy"
						class="text-secondary hover:!filter-none hover:text-red focus-visible:!filter-none"
						@click="$emit('revoke', row.id)"
					>
						<SpinnerIcon v-if="revokingId === row.id" class="animate-spin" aria-hidden="true" />
						<XIcon v-else aria-hidden="true" />
					</IconButton>
				</div>
			</template>
		</Table>
	</section>
</template>
