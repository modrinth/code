import { injectModrinthClient, injectNotificationManager } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import { sharedInstanceQueryKeys } from '~/composables/queries/shared-instance'
import { getRequestErrorMessage } from '~/helpers/shared-instance-moderation'

type InstanceRef = { id: string; name: string }
type UserRef = { id: string; username: string }

export function useSharedInstanceModeration() {
	const client = injectModrinthClient()
	const queryClient = useQueryClient()
	const { addNotification } = injectNotificationManager()
	const pending = ref(false)

	async function run(
		errorTitle: string,
		request: () => Promise<unknown>,
		success: { title: string; text: string },
		invalidate: () => Promise<unknown>,
	): Promise<boolean> {
		if (pending.value) return false
		pending.value = true
		try {
			await request()
			await invalidate()
			addNotification({ type: 'success', ...success })
			return true
		} catch (error) {
			addNotification({
				type: 'error',
				title: errorTitle,
				text: getRequestErrorMessage(error, 'The shared instances service could not be reached.'),
			})
			return false
		} finally {
			pending.value = false
		}
	}

	function invalidateInstance(instanceId: string) {
		return Promise.all([
			queryClient.invalidateQueries({
				queryKey: sharedInstanceQueryKeys.moderationContext(instanceId),
			}),
			queryClient.invalidateQueries({ queryKey: sharedInstanceQueryKeys.allUserInstances }),
		])
	}

	function invalidateUser(userId: string) {
		return Promise.all([
			queryClient.invalidateQueries({ queryKey: sharedInstanceQueryKeys.blacklist(userId) }),
			queryClient.invalidateQueries({ queryKey: sharedInstanceQueryKeys.allModerationContexts }),
			queryClient.invalidateQueries({ queryKey: sharedInstanceQueryKeys.allUserInstances }),
		])
	}

	function invalidateVersion(instanceId: string, version: number) {
		return Promise.all([
			queryClient.invalidateQueries({
				queryKey: sharedInstanceQueryKeys.version(instanceId, version),
			}),
			queryClient.invalidateQueries({
				queryKey: sharedInstanceQueryKeys.versionContent(instanceId, version),
			}),
		])
	}

	function quarantine(instance: InstanceRef, reason: string | null, editing: boolean) {
		return run(
			editing ? 'Failed to update quarantine' : 'Failed to quarantine instance',
			() =>
				client.sharedinstances.moderation_v1.quarantineInstance(instance.id, {
					...(reason ? { reason } : {}),
					notify: !!reason,
				}),
			editing
				? {
						title: 'Quarantine updated',
						text: reason
							? `The owner of ${instance.name} was sent the new message.`
							: `The message for ${instance.name} was removed.`,
					}
				: {
						title: 'Instance quarantined',
						text: reason
							? `${instance.name} was quarantined and its owner was notified.`
							: `${instance.name} was quarantined without notifying its owner.`,
					},
			() => invalidateInstance(instance.id),
		)
	}

	function liftQuarantine(instance: InstanceRef) {
		return run(
			'Failed to lift quarantine',
			() => client.sharedinstances.moderation_v1.unquarantineInstance(instance.id),
			{ title: 'Quarantine lifted', text: `${instance.name} is no longer quarantined.` },
			() => invalidateInstance(instance.id),
		)
	}

	function banUser(user: UserRef) {
		return run(
			'Failed to ban user from shared instances',
			() => client.sharedinstances.moderation_v1.blacklistUsers({ user_ids: [user.id] }),
			{
				title: 'User banned from shared instances',
				text: `${user.username} was banned and all of their shared instances were quarantined.`,
			},
			() => invalidateUser(user.id),
		)
	}

	function unbanUser(user: UserRef) {
		return run(
			'Failed to unban user from shared instances',
			() => client.sharedinstances.moderation_v1.unblacklistUsers({ user_ids: [user.id] }),
			{
				title: 'User unbanned from shared instances',
				text: `${user.username} can use shared instances again.`,
			},
			() => invalidateUser(user.id),
		)
	}

	function deleteFile(instance: InstanceRef, version: number, fileName: string, fileType: string) {
		return run(
			'Failed to delete file',
			() =>
				client.sharedinstances.moderation_v1.deleteFile(instance.id, version, fileName, fileType),
			{ title: 'File deleted', text: `${fileName} was removed from version ${version}.` },
			() => invalidateVersion(instance.id, version),
		)
	}

	function deleteFiles(
		instance: InstanceRef,
		version: number,
		files: { fileName: string; fileType: string }[],
	) {
		if (files.length === 1) {
			return deleteFile(instance, version, files[0].fileName, files[0].fileType)
		}

		return run(
			'Failed to delete files',
			async () => {
				try {
					for (const { fileName, fileType } of files) {
						await client.sharedinstances.moderation_v1.deleteFile(
							instance.id,
							version,
							fileName,
							fileType,
						)
					}
				} catch (error) {
					await invalidateVersion(instance.id, version)
					throw error
				}
			},
			{
				title: 'Files deleted',
				text: `${files.length} files were removed from version ${version}.`,
			},
			() => invalidateVersion(instance.id, version),
		)
	}

	return { pending, quarantine, liftQuarantine, banUser, unbanUser, deleteFile, deleteFiles }
}

export function useCopyText() {
	const { addNotification } = injectNotificationManager()

	return async (text: string, title: string) => {
		await navigator.clipboard.writeText(text)
		addNotification({ type: 'success', title, text })
	}
}
