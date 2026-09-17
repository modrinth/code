import { injectAuth, type SharingSettingsContext } from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed } from 'vue'

import {
	get_shared_instance_invites,
	revoke_shared_instance_invite,
	type SharedInstanceInvite,
	unpublish_shared_instance,
} from '@/helpers/instance'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'

import { instanceKeys } from '../../../query-options.ts'
import { injectInstanceSettings } from '../instance-settings-context.ts'

export function useInstanceSharingSettings(): SharingSettingsContext {
	const { instance, offline, onUnlinked } = injectInstanceSettings()
	const auth = injectAuth()
	const { notifySharedInstanceError } = useSharedInstanceErrors()
	const queryClient = useQueryClient()
	const targetKey = computed(() => {
		const sharedInstance = instance.value.shared_instance
		return sharedInstance?.role === 'owner' && !instance.value.quarantined && auth.user.value
			? JSON.stringify([instance.value.id, sharedInstance.id, auth.user.value.id])
			: null
	})
	const invitesKey = computed(
		() => ['sharedInstanceInvites', instance.value.id, auth.user.value?.id] as const,
	)
	const invites = useQuery({
		queryKey: invitesKey,
		queryFn: ({ queryKey }) => get_shared_instance_invites(queryKey[1]),
		enabled: () => !!targetKey.value && !offline,
		retry: false,
		staleTime: Infinity,
		refetchOnMount: 'always',
		refetchOnReconnect: false,
		refetchOnWindowFocus: false,
	})

	type ShareTarget = {
		instanceId: string
		invitesKey: typeof invitesKey.value
	}
	const revokeMutation = useMutation({
		mutationFn: (target: ShareTarget & { inviteId: string }) =>
			revoke_shared_instance_invite(target.instanceId, target.inviteId),
		onSuccess: (_data, target) => {
			queryClient.setQueryData<SharedInstanceInvite[]>(target.invitesKey, (current = []) =>
				current.filter((invite) => invite.id !== target.inviteId),
			)
		},
	})
	const unpublishMutation = useMutation({
		mutationFn: (target: ShareTarget) => unpublish_shared_instance(target.instanceId),
		onSuccess: async (_data, target) => {
			queryClient.setQueryData(instanceKeys.sharedMembers(target.instanceId), [])
			queryClient.setQueryData(target.invitesKey, [])
			await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', target.instanceId] })
			if (target.instanceId === instance.value.id && target.invitesKey[2] === auth.user.value?.id) {
				onUnlinked()
			}
		},
	})
	const busy = computed(
		() =>
			!targetKey.value ||
			!!offline ||
			instance.value.install_stage !== 'installed' ||
			revokeMutation.isPending.value ||
			unpublishMutation.isPending.value,
	)

	function currentTarget(expectedKey: string): ShareTarget | undefined {
		if (busy.value || expectedKey !== targetKey.value) return
		return { instanceId: instance.value.id, invitesKey: invitesKey.value }
	}

	return {
		targetKey,
		invites: computed(() => invites.data.value ?? []),
		loading: invites.isLoading,
		error: invites.error,
		busy,
		refresh: async () => {
			if (targetKey.value && !offline) await invites.refetch()
		},
		revokeInvite: async (inviteId, expectedKey) => {
			const target = currentTarget(expectedKey)
			if (target) await revokeMutation.mutateAsync({ ...target, inviteId })
		},
		unpublish: async (expectedKey) => {
			const target = currentTarget(expectedKey)
			if (target) await unpublishMutation.mutateAsync(target)
		},
		onError: notifySharedInstanceError,
	}
}
