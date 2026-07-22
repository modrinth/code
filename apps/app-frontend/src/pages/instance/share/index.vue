<template>
	<div class="flex flex-col gap-4">
		<ModrinthAccountRequiredModal ref="accountRequiredModal" :request-auth="requestAuth" />
		<InvitePlayersModal
			ref="invitePlayersModal"
			:header="formatMessage(messages.shareModalHeader, { name: instance.name })"
			:friends="inviteFriends"
			:search-users="searchInviteUsers"
			:link="inviteLink.link.value"
			:link-expires-at="inviteLink.details.value?.expiresAt"
			:link-max-uses="inviteLink.details.value?.maxUses"
			:update-invite-link="inviteLink.update"
			:user-profile-link="userProfileLink"
			@invite="invitePlayer"
			@cancel="cancelInvite"
		/>
		<ConfirmUnlinkModal
			ref="unlinkModal"
			:warning="{
				header: formatMessage(messages.unlinkForShareHeader),
				body: formatMessage(messages.unlinkForShareBody),
			}"
			:backup-tip="importedModpackBackupTip"
			@unlink="unlinkImportedModpack"
		/>
		<SharedInstanceRemoveMemberModal
			ref="removeMemberModal"
			:row="pendingRemovalRow"
			:member-count="members.rows.value.length"
			@confirm="removeMember"
			@clear="pendingRemovalRow = null"
		/>
		<SharedInstancePublishModal
			ref="publishModal"
			:instance="instance"
			@state-change="publishState = $event"
		/>

		<SharedInstanceMembersTable
			v-if="members.rows.value.length > 0"
			:rows="members.rows.value"
			:actions-locked="sharedInstanceActionsLocked"
			:invite-pending="inviteLink.pending.value"
			:push-update-disabled="
				instance.install_stage !== 'installed' || publishState !== 'idle' || !!offline
			"
			:push-update-pending="publishState !== 'idle'"
			@invite="showInvitePlayers"
			@remove="showRemoveMemberModal"
			@push-update="reviewUpdate"
		/>

		<SharedInstanceShareEmptyState
			v-else-if="sharedInstanceUnavailable"
			:heading="formatMessage(sharedInstanceErrorMessages.unavailableTitle)"
			:description="
				formatSharedInstanceUnavailable(
					sharedInstanceUnavailableReason,
					sharedInstanceUnavailableManager,
				)
			"
		/>

		<SharedInstanceShareEmptyState
			v-else-if="sharedInstanceActionsLocked"
			:heading="formatMessage(lockedEmptyHeading)"
		>
			<template #description>
				<span class="flex flex-wrap items-center justify-center gap-x-1.5 gap-y-1">
					<span>{{ formatMessage(messages.lockedEmptyDescriptionPrefix) }}</span>
					<span
						v-if="linkedAccount"
						class="inline-flex max-w-full min-w-0 items-center gap-1.5 align-middle font-semibold text-primary"
					>
						<Avatar
							:src="linkedAccount.avatarUrl"
							:alt="linkedAccount.username"
							:tint-by="linkedAccount.tintBy"
							size="20px"
							circle
							no-shadow
						/>
						<span class="min-w-0 truncate">{{ linkedAccount.username }}</span>
					</span>
					<span v-else class="font-semibold text-primary">{{
						formatMessage(messages.linkedAccountFallback)
					}}</span>
					<span>{{ formatMessage(messages.lockedEmptyDescriptionSuffix) }}</span>
				</span>
			</template>
			<template #actions>
				<ButtonStyled color="brand"
					><button class="!h-10" @click="signInToShare">
						<LogInIcon aria-hidden="true" />{{ formatMessage(lockedActionButton) }}
					</button></ButtonStyled
				>
			</template>
		</SharedInstanceShareEmptyState>

		<SharedInstanceShareEmptyState
			v-else
			:heading="formatMessage(messages.noFriendsInvitedHeading)"
			:description="formatMessage(messages.noFriendsInvitedDescription)"
		>
			<template #actions>
				<ButtonStyled color="brand"
					><button
						class="!h-10"
						:disabled="inviteLink.pending.value"
						@click="showInvitePlayers($event)"
					>
						<SpinnerIcon
							v-if="inviteLink.pending.value"
							class="animate-spin"
							aria-hidden="true"
						/><UserPlusIcon v-else aria-hidden="true" />{{
							formatMessage(messages.inviteFriendsButton)
						}}
					</button></ButtonStyled
				>
			</template>
		</SharedInstanceShareEmptyState>
	</div>
</template>

<script setup lang="ts">
import { LogInIcon, SpinnerIcon, UserPlusIcon } from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	ConfirmUnlinkModal,
	defineMessages,
	injectAuth,
	injectNotificationManager,
	type InvitePlayersInvitePayload,
	InvitePlayersModal,
	type InvitePlayersUser,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, ref, toRef, watch } from 'vue'

import ModrinthAccountRequiredModal from '@/components/ui/modal/ModrinthAccountRequiredModal.vue'
import SharedInstancePublishModal from '@/components/ui/shared-instances/SharedInstancePublishModal.vue'
import {
	getSharedInstanceUnavailableReason,
	isSharedInstanceUnavailableError,
} from '@/helpers/install'
import { edit } from '@/helpers/instance'
import type { ModrinthAuthFlow } from '@/helpers/mr_auth.ts'
import {
	sharedInstanceErrorMessages,
	useSharedInstanceErrors,
} from '@/helpers/shared-instance-errors'
import type { GameInstance } from '@/helpers/types'
import { provideInstanceBackup } from '@/providers/instance-backup'

import { injectSharedInstanceState } from '../use-shared-instance-state'
import SharedInstanceMembersTable from './shared-instance-members-table.vue'
import SharedInstanceRemoveMemberModal from './shared-instance-remove-member-modal.vue'
import SharedInstanceShareEmptyState from './shared-instance-share-empty-state.vue'
import type { ShareRow } from './shared-instance-share-types'
import { useSharedInstanceInviteCandidates } from './use-shared-instance-invite-candidates'
import { useSharedInstanceInviteLink } from './use-shared-instance-invite-link'
import { useSharedInstanceMembers } from './use-shared-instance-members'

const props = defineProps<{
	instance: GameInstance
	offline?: boolean
}>()
const auth = injectAuth()
const queryClient = useQueryClient()
const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const {
	formatSharedInstanceUnavailable,
	notifySharedInstanceError,
	notifySharedInstanceUnavailable,
} = useSharedInstanceErrors()
const sharedInstanceState = injectSharedInstanceState()
const instance = toRef(props, 'instance')
const actionsLocked = sharedInstanceState.shareActionsLocked
const sharedInstanceActionsLocked = actionsLocked
const currentUserId = computed(() => auth.user.value?.id ?? null)
const isSignedIn = computed(() => !!auth.session_token.value)
const accountRequiredModal = ref<InstanceType<typeof ModrinthAccountRequiredModal>>()
const invitePlayersModal = ref<InstanceType<typeof InvitePlayersModal>>()
const unlinkModal = ref<InstanceType<typeof ConfirmUnlinkModal>>()
const removeMemberModal = ref<InstanceType<typeof SharedInstanceRemoveMemberModal>>()
const publishModal = ref<InstanceType<typeof SharedInstancePublishModal>>()
const publishState = ref<'idle' | 'reviewing' | 'publishing'>('idle')
const pendingRemovalRow = ref<ShareRow | null>(null)
const importedModpackUnlinked = ref(false)

function notifyOperationError(error: unknown) {
	if (isSharedInstanceUnavailableError(error)) {
		notifySharedInstanceUnavailable(
			getSharedInstanceUnavailableReason(error),
			sharedInstanceState.unavailableManager.value,
		)
	} else {
		notifySharedInstanceError(error)
	}
}

const members = useSharedInstanceMembers({
	instance,
	currentUserId,
	isSignedIn,
	actionsLocked,
	onError: notifyOperationError,
	onHydrationError: handleError,
})
const {
	inviteFriends,
	search: searchInviteUsers,
	requestFriend,
} = useSharedInstanceInviteCandidates({
	rows: members.rows,
	currentUserId,
	isSignedIn,
	actionsLocked,
})
const inviteLink = useSharedInstanceInviteLink(
	computed(() => props.instance.id),
	notifyOperationError,
)

const linkedAccount = computed(() => {
	const manager = sharedInstanceState.manager.value
	return manager?.type === 'user'
		? { username: manager.name, avatarUrl: manager.avatarUrl, tintBy: manager.tintBy }
		: null
})
const lockedEmptyHeading = computed(() =>
	isSignedIn.value ? messages.lockedWrongAccountHeading : messages.lockedSignedOutHeading,
)
const lockedActionButton = computed(() =>
	isSignedIn.value ? messages.switchAccountButton : messages.signInButton,
)
const sharedInstanceUnavailableReason = sharedInstanceState.unavailableReason
const sharedInstanceUnavailable = computed(() => !!sharedInstanceUnavailableReason.value)
const sharedInstanceUnavailableManager = sharedInstanceState.unavailableManager
const requiresUnlink = computed(
	() =>
		props.instance.link?.type === 'imported_modpack' &&
		!props.instance.shared_instance &&
		!importedModpackUnlinked.value,
)
const importedModpackBackupTip = computed(() =>
	props.instance.link?.type === 'imported_modpack'
		? (props.instance.link.name ?? props.instance.link.filename ?? undefined)
		: undefined,
)

const messages = defineMessages({
	signInButton: { id: 'app.instance.share.sign-in.button', defaultMessage: 'Sign in' },
	noFriendsInvitedHeading: {
		id: 'app.instance.share.empty.heading',
		defaultMessage: 'No friends invited',
	},
	noFriendsInvitedDescription: {
		id: 'app.instance.share.empty.description',
		defaultMessage: 'You can share this instance with your friends!',
	},
	inviteFriendsButton: {
		id: 'app.instance.share.empty.invite-friends-button',
		defaultMessage: 'Invite friends',
	},
	shareModalHeader: {
		id: 'app.instance.share.invite-modal.heading',
		defaultMessage: 'Share {name}',
	},
	lockedWrongAccountHeading: {
		id: 'app.instance.share.locked.wrong-account-heading',
		defaultMessage: 'Wrong account',
	},
	lockedSignedOutHeading: {
		id: 'app.instance.share.locked.signed-out-heading',
		defaultMessage: 'Not signed in',
	},
	lockedEmptyDescriptionPrefix: {
		id: 'app.instance.share.locked.empty-description-prefix',
		defaultMessage: 'You need to sign in as',
	},
	lockedEmptyDescriptionSuffix: {
		id: 'app.instance.share.locked.empty-description-suffix',
		defaultMessage: 'to access this page.',
	},
	linkedAccountFallback: {
		id: 'app.instance.share.locked.linked-account-fallback',
		defaultMessage: 'the linked account',
	},
	switchAccountButton: {
		id: 'app.instance.share.locked.switch-account-button',
		defaultMessage: 'Switch account',
	},
	unlinkForShareHeader: {
		id: 'app.instance.share.unlink.header',
		defaultMessage: 'Sharing requires unlinking',
	},
	unlinkForShareBody: {
		id: 'app.instance.share.unlink.body',
		defaultMessage: 'You must unlink this modpack to share your instance',
	},
})

function invitePlayer(payload: InvitePlayersInvitePayload) {
	if (actionsLocked.value) return
	if (payload.source === 'search') void requestFriend(payload.user)
	members.invite(payload.user)
}
function cancelInvite(user: InvitePlayersUser) {
	const row = members.find(user.id, user.username)
	if (row) members.remove(row.id)
}
async function showInvitePlayers(event?: MouseEvent) {
	if (actionsLocked.value) return
	if (!isSignedIn.value) return signInToShare(event)
	if (requiresUnlink.value) return unlinkModal.value?.show()
	if (await inviteLink.ensure()) invitePlayersModal.value?.show(event)
}
async function unlinkImportedModpack() {
	try {
		await edit(props.instance.id, { link: null as unknown as undefined })
		importedModpackUnlinked.value = true
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', props.instance.id] })
		if (await inviteLink.ensure()) invitePlayersModal.value?.show()
	} catch (error) {
		notifyOperationError(error)
	}
}
function showRemoveMemberModal(row: ShareRow) {
	if (!actionsLocked.value) {
		pendingRemovalRow.value = row
		removeMemberModal.value?.show()
	}
}
function reviewUpdate(event: MouseEvent) {
	publishModal.value?.show(event)
}
function removeMember(row: ShareRow) {
	members.remove(row.id)
}
function userProfileLink(username: string) {
	return !username || username.includes('@')
		? undefined
		: () => openUrl(`https://modrinth.com/user/${encodeURIComponent(username)}`)
}
async function requestAuth(flow: ModrinthAuthFlow) {
	await auth.requestSignIn(`/instance/${encodeURIComponent(props.instance.id)}/share`, flow, {
		showModal: false,
	})
	return !!auth.session_token.value
}
function signInToShare(event?: MouseEvent) {
	void accountRequiredModal.value?.show(event)
}

watch(
	() => props.instance.id,
	() => {
		importedModpackUnlinked.value = false
	},
)
watch(
	[() => auth.isReady.value, isSignedIn, actionsLocked],
	([ready, signedIn, locked]) => {
		if (ready && !signedIn && !locked) signInToShare()
	},
	{ immediate: true, flush: 'post' },
)

provideInstanceBackup(() => props.instance)
</script>
