<script setup lang="ts">
import { useQueryClient } from '@tanstack/vue-query'
import { ref } from 'vue'

import SharedInstanceInstallModal from '@/components/ui/shared-instances/shared-instance-install-modal/index.vue'
import { install_shared_instance } from '@/helpers/install'
import { useSharedInstanceErrors } from '@/helpers/shared-instance-errors'
import {
	leaveRemoteSharedInstance,
	unpublishRemoteSharedInstance,
} from '@/platform/app-lib/shared-instances/commands'

import type { SharedInstanceInstallOptions } from './actions-context'
import { sharedInstanceInstallPreviewQueryOptions, sharedInstanceLibraryKeys } from './queries'
import RemoveSharedInstancesModal from './remove-shared-instances-modal.vue'
import type { UninstalledSharedInstance } from './types'

const queryClient = useQueryClient()
const { notifySharedInstanceError } = useSharedInstanceErrors()

const installModal = ref<InstanceType<typeof SharedInstanceInstallModal>>()
const removeModal = ref<InstanceType<typeof RemoveSharedInstancesModal>>()
const loadingIds = ref(new Set<string>())
const removing = ref(false)

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
				await install_shared_instance(
					sharedInstanceId,
					instance.name,
					instance.manager_id,
					null,
					null,
					instance.icon_path ?? null,
				)
				await queryClient.invalidateQueries({ queryKey: ['instances'] })
				options.onInstalled?.()
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

async function remove(sharedInstances: UninstalledSharedInstance[]) {
	removing.value = true
	const results = await Promise.allSettled(
		sharedInstances.map((instance) =>
			instance.role === 'owner'
				? unpublishRemoteSharedInstance(instance.shared_instance_id)
				: leaveRemoteSharedInstance(instance.shared_instance_id),
		),
	)
	removing.value = false
	for (const result of results) {
		if (result.status === 'rejected') notifySharedInstanceError(result.reason)
	}

	const removedSharedInstanceIds = new Set(
		sharedInstances
			.filter((_, index) => results[index].status === 'fulfilled')
			.map((instance) => instance.shared_instance_id),
	)
	queryClient.setQueriesData<string[]>(
		{ queryKey: [...sharedInstanceLibraryKeys.all, 'ids'] },
		(sharedInstanceIds) =>
			sharedInstanceIds?.filter(
				(sharedInstanceId) => !removedSharedInstanceIds.has(sharedInstanceId),
			),
	)
}

function requestRemove(sharedInstances: UninstalledSharedInstance[], event?: MouseEvent) {
	if (sharedInstances.length === 0) return
	removeModal.value?.show(sharedInstances, () => remove(sharedInstances), event)
}

defineExpose({ loadingIds, removing, install, prefetchInstall, requestRemove })
</script>

<template>
	<SharedInstanceInstallModal ref="installModal" />
	<RemoveSharedInstancesModal ref="removeModal" />
</template>
