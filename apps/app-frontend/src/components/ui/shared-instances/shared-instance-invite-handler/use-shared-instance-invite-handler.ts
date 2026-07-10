import {
	injectAuth,
	injectModrinthClient,
	injectNotificationManager,
	injectPopupNotificationManager,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type Ref, watch } from 'vue'

import { config } from '@/config'
import { get_user } from '@/helpers/cache'
import { toError } from '@/helpers/errors'
import {
	install_accept_shared_instance_invite,
	install_get_shared_instance_preview,
	install_shared_instance,
} from '@/helpers/install'

import { parseSharedInstanceInviteNotification } from './shared-instance-invite-parser'
import type { AppNotification, SharedInstanceInvite } from './shared-instance-invite-types'

type InstallModal = {
	show(
		preview: Awaited<ReturnType<typeof install_get_shared_instance_preview>>,
		install: () => Promise<void>,
	): void
}

type AccountRequiredModal = {
	show(event?: MouseEvent): Promise<boolean>
}

export function useSharedInstanceInviteHandler(
	installModal: Ref<InstallModal | undefined>,
	accountRequiredModal: Ref<AccountRequiredModal | undefined>,
) {
	const auth = injectAuth()
	const client = injectModrinthClient()
	const { handleError } = injectNotificationManager()
	const { addPopupNotification } = injectPopupNotificationManager()
	const queryClient = useQueryClient()
	const displayedNotifications = new Set<string | number>()

	async function markNotificationRead(notification: AppNotification) {
		await client.labrinth.notifications_v2.markAsRead(String(notification.id))
	}

	async function resolveInvite(invite: SharedInstanceInvite) {
		const invitedBy =
			!invite.invitedByUsername && invite.invitedById
				? await get_user(invite.invitedById, 'bypass').catch(() => null)
				: null

		return {
			...invite,
			invitedByUsername: invite.invitedByUsername ?? invitedBy?.username ?? null,
			invitedByAvatarUrl: invite.invitedByAvatarUrl ?? invitedBy?.avatar_url ?? null,
		}
	}

	function showInstall(
		preview: Awaited<ReturnType<typeof install_get_shared_instance_preview>>,
		install: () => Promise<void>,
	) {
		if (!installModal.value) throw new Error('Shared instance install modal is not available.')
		installModal.value.show(preview, install)
	}

	async function acceptNotification(notification: AppNotification, invite: SharedInstanceInvite) {
		try {
			const preview = await install_get_shared_instance_preview(
				invite.sharedInstanceId,
				invite.sharedInstanceName,
			)
			if (invite.instanceIconUrl) preview.iconUrl = invite.instanceIconUrl

			showInstall(
				preview,
				async () => {
					await install_shared_instance(
						invite.sharedInstanceId,
						invite.sharedInstanceName,
						invite.invitedById,
						null,
						null,
						invite.instanceIconUrl,
					)
					await markNotificationRead(notification)
					await queryClient.invalidateQueries({ queryKey: ['instances'] })
				},
			)
		} catch (error) {
			handleError(toError(error))
		}
	}

	async function handleNotification(notification: AppNotification) {
		const parsedInvite = parseSharedInstanceInviteNotification(notification)
		if (!parsedInvite) return false
		if (displayedNotifications.has(notification.id)) return true

		displayedNotifications.add(notification.id)
		const invite = await resolveInvite(parsedInvite)
		addPopupNotification({
			title: invite.sharedInstanceName,
			autoCloseMs: null,
			toast: {
				type: 'instance-invite',
				actorName: invite.invitedByUsername,
				actorAvatarUrl: invite.invitedByAvatarUrl ?? undefined,
				entityName: invite.sharedInstanceName,
				entityIconUrl: invite.instanceIconUrl ?? undefined,
				onAccept: () => acceptNotification(notification, invite),
				onDecline: () => markNotificationRead(notification).catch((error) => handleError(toError(error))),
				onOpenActor: () => {
					if (invite.invitedByUsername) {
						openUrl(`${config.siteUrl}/user/${encodeURIComponent(invite.invitedByUsername)}`)
					}
				},
			},
		})
		return true
	}

	async function requireAccount() {
		if (!auth.isReady?.value) {
			await new Promise<void>((resolve) => {
				const stop = watch(auth.isReady!, (ready) => {
					if (ready) {
						stop()
						resolve()
					}
				})
			})
		}
		if (auth.session_token.value) return true
		return (await accountRequiredModal.value?.show()) ?? false
	}

	async function installFromInviteId(inviteId: string) {
		try {
			if (!(await requireAccount())) return
			const invite = await install_accept_shared_instance_invite(inviteId)
			showInstall(invite.preview, async () => {
				await install_shared_instance(
					invite.sharedInstanceId,
					invite.preview.name,
					invite.managerId,
					invite.serverManagerName,
					invite.serverManagerIconUrl,
					invite.instanceIconUrl,
				)
				await queryClient.invalidateQueries({ queryKey: ['instances'] })
			})
		} catch (error) {
			handleError(toError(error))
		}
	}

	return { handleNotification, installFromInviteId }
}
