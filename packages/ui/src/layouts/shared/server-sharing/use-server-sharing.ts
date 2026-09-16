import type { SharedInstances } from '@modrinth/api-client'
import { useIsMutating, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed } from 'vue'

import { useServerPermissions } from '#ui/composables/server-permissions'
import {
	injectAuth,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

import type { SharingSettingsContext } from '../sharing-settings'
import { clearServerSharedInstance } from './cache'
import { sharedInstanceInvitesQueryOptions } from './query-options'

export function useServerSharingSettings() {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()
	const { serverId, worldId, serverFull, busyReasons } = injectModrinthServerContext()
	const { canSetup, permissionDeniedMessage } = useServerPermissions()
	const preferences = useStorage(
		`pyro-server-${serverId}-preferences`,
		{ reviewChangesBeforePlaying: false },
		undefined,
		{ mergeDefaults: true },
	)
	const userId = computed(() => auth.user.value?.id)
	const sharedInstanceId = computed(() =>
		serverFull.value?.worlds.find((world) => world.id === worldId.value)?.content?.shared_instance_id,
	)
	const targetKey = computed(() =>
		canSetup.value && worldId.value && sharedInstanceId.value && userId.value
			? JSON.stringify([serverId, worldId.value, sharedInstanceId.value, userId.value])
			: null,
	)
	const invites = useQuery(
		computed(() => ({
			...sharedInstanceInvitesQueryOptions(client, sharedInstanceId.value ?? '', userId.value),
			enabled: !!targetKey.value,
			refetchOnMount: 'always' as const,
			refetchInterval: 30_000,
		})),
	)
	const shareActions = useIsMutating({ mutationKey: ['servers', 'share-action', serverId] })

	type ShareTarget = { worldId: string; instanceId: string }
	const revokeMutation = useMutation({
		mutationFn: (target: ShareTarget & { inviteId: string }) =>
			client.sharedinstances.invites_v1.delete(target.instanceId, target.inviteId),
		onSuccess: (_data, target) => {
			queryClient.setQueriesData<SharedInstances.Invites.v1.InviteLink[]>(
				{ queryKey: ['shared-instances', target.instanceId, 'invites'] },
				(current) => current?.filter((invite) => invite.id !== target.inviteId),
			)
		},
		onSettled: (_data, _error, target) =>
			queryClient.invalidateQueries({ queryKey: ['shared-instances', target.instanceId, 'invites'] }),
	})
	const unpublishMutation = useMutation({
		mutationKey: ['servers', 'share-action', serverId],
		mutationFn: (target: ShareTarget) => client.archon.content_v1.unshare(serverId, target.worldId),
		onSuccess: (_data, target) =>
			clearServerSharedInstance(queryClient, serverId, target.worldId, target.instanceId),
		onSettled: () => queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
	})
	const busy = computed(
		() =>
			!targetKey.value ||
			busyReasons.value.length > 0 ||
			shareActions.value > 0 ||
			revokeMutation.isPending.value ||
			unpublishMutation.isPending.value,
	)

	function currentTarget(expectedKey: string): ShareTarget | undefined {
		if (busy.value || expectedKey !== targetKey.value || !worldId.value || !sharedInstanceId.value) {
			return
		}
		return { worldId: worldId.value, instanceId: sharedInstanceId.value }
	}

	const settings: SharingSettingsContext = {
		targetKey,
		invites: computed(() =>
			(invites.data.value ?? []).map((invite) => ({
				id: invite.id,
				expiration: invite.expiration,
				maxUses: invite.max_uses,
				uses: invite.uses,
			})),
		),
		loading: invites.isLoading,
		error: invites.error,
		busy,
		refresh: async () => {
			if (targetKey.value) await invites.refetch()
		},
		revokeInvite: async (inviteId, expectedKey) => {
			const target = currentTarget(expectedKey)
			if (target) await revokeMutation.mutateAsync({ ...target, inviteId })
		},
		unpublish: async (expectedKey) => {
			const target = currentTarget(expectedKey)
			if (target) await unpublishMutation.mutateAsync(target)
		},
		onError: (error) => handleError(error),
		reviewChangesBeforePlaying: computed({
			get: () => preferences.value.reviewChangesBeforePlaying,
			set: (value) => {
				preferences.value.reviewChangesBeforePlaying = value
			},
		}),
	}

	return { settings, canSetup, permissionDeniedMessage, sharedInstanceId }
}
