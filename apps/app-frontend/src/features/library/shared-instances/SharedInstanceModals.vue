<script setup lang="ts">
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import { instanceKeys } from '@/features/instances/queries'
import { install_shared_instance, wait_for_install_job } from '@/helpers/install'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import {
	leaveRemoteSharedInstance,
	unpublishRemoteSharedInstance,
} from '@/platform/app-lib/shared-instances/commands'
import { injectAppEvents } from '@/providers/app-events'

import { sharedInstanceInstallPreviewQueryOptions, sharedInstanceLibraryKeys } from './queries'
import RemoveSharedInstancesModal from './RemoveSharedInstancesModal.vue'
import type {
	SharedInstanceInstallOptions,
	SharedInstanceModalsExposed,
	UninstalledSharedInstance,
} from './types'

const queryClient = useQueryClient()
const appEvents = injectAppEvents()
const { notifySharedInstanceError } = useSharedInstanceErrors()

const installModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const removeModal = ref<InstanceType<typeof RemoveSharedInstancesModal>>()
const loadingIds = ref(new Set<string>())

function setLoading(sharedInstanceId: string, loading: boolean) {
	const nextLoadingIds = new Set(loadingIds.value)
	if (loading) {
		nextLoadingIds.add(sharedInstanceId)
	} else {
		nextLoadingIds.delete(sharedInstanceId)
	}
	loadingIds.value = nextLoadingIds
}

function prefetchInstall(instance: UninstalledSharedInstance) {
	void queryClient.prefetchQuery(sharedInstanceInstallPreviewQueryOptions(instance))
}

async function install(
	instance: UninstalledSharedInstance,
	options: SharedInstanceInstallOptions = {},
) {
	const sharedInstanceId = instance.shared_instance_id
	if (loadingIds.value.has(sharedInstanceId)) return

	setLoading(sharedInstanceId, true)
	try {
		const { preview, manager } = await queryClient.fetchQuery(
			sharedInstanceInstallPreviewQueryOptions(instance),
		)

		installModal.value?.show(
			instance.icon_path ? { ...preview, iconUrl: instance.icon_path } : preview,
			async () => {
				if (loadingIds.value.has(sharedInstanceId)) return
				setLoading(sharedInstanceId, true)
				let jobId: string
				try {
					const job = await install_shared_instance(
						sharedInstanceId,
						instance.name,
						instance.manager_id,
						null,
						null,
						instance.icon_path ?? null,
						instance.icon_path ? (instance.icon_config ?? null) : null,
					)
					jobId = job.job_id
					await queryClient.invalidateQueries({ queryKey: instanceKeys.all })
					options.onInstalled?.()
				} catch (error) {
					setLoading(sharedInstanceId, false)
					throw error
				}
				void wait_for_install_job(appEvents, jobId)
					.catch(() => {})
					.finally(() => setLoading(sharedInstanceId, false))
			},
			manager
				? { id: manager.id, username: manager.username, avatarUrl: manager.avatar_url ?? null }
				: undefined,
			options.event,
			{ isOwner: instance.role === 'owner' },
		)
	} catch (error) {
		notifySharedInstanceError(error)
	} finally {
		setLoading(sharedInstanceId, false)
	}
}

const sharedInstanceIdsKey = [...sharedInstanceLibraryKeys.all, 'ids'] as const

const removeMutation = useMutation({
	mutationFn: async (sharedInstances: UninstalledSharedInstance[]) => {
		const results = await Promise.allSettled(
			sharedInstances.map((instance) =>
				instance.role === 'owner'
					? unpublishRemoteSharedInstance(instance.shared_instance_id)
					: leaveRemoteSharedInstance(instance.shared_instance_id),
			),
		)
		for (const result of results) {
			if (result.status === 'rejected') notifySharedInstanceError(result.reason)
		}
	},
	onMutate: async (sharedInstances) => {
		await queryClient.cancelQueries({ queryKey: sharedInstanceIdsKey })
		const previous = queryClient.getQueriesData<string[]>({ queryKey: sharedInstanceIdsKey })
		const removedIds = new Set(sharedInstances.map((instance) => instance.shared_instance_id))
		queryClient.setQueriesData<string[]>({ queryKey: sharedInstanceIdsKey }, (ids) =>
			ids?.filter((sharedInstanceId) => !removedIds.has(sharedInstanceId)),
		)
		return { previous }
	},
	onError: (error, _, context) => {
		for (const [queryKey, data] of context?.previous ?? []) {
			queryClient.setQueryData(queryKey, data)
		}
		notifySharedInstanceError(error)
	},
	onSettled: (_, __, sharedInstances) =>
		Promise.all([
			queryClient.invalidateQueries({ queryKey: sharedInstanceIdsKey }),
			sharedInstances.some((instance) => instance.role === 'owner')
				? queryClient.invalidateQueries({ queryKey: [...sharedInstanceLibraryKeys.all, 'limits'] })
				: undefined,
		]),
})
const removing = computed(() => removeMutation.isPending.value)

function requestRemove(sharedInstances: UninstalledSharedInstance[], event?: MouseEvent) {
	if (sharedInstances.length === 0) return
	removeModal.value?.show(
		sharedInstances,
		() => removeMutation.mutateAsync(sharedInstances).catch(() => undefined),
		event,
	)
}

defineExpose<SharedInstanceModalsExposed>({
	get loadingIds() {
		return loadingIds.value
	},
	get removing() {
		return removing.value
	},
	install,
	prefetchInstall,
	requestRemove,
})
</script>

<template>
	<SharedInstanceInstallModal ref="installModal" />
	<RemoveSharedInstancesModal ref="removeModal" />
</template>
