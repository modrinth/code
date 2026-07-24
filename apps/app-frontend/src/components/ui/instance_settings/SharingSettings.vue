<template>
	<div>
		<SharedInstanceInstallationSettingsControls
			can-unpublish
			:busy="isBusy"
			:unpublishing="unpublishing"
			:unpublish="unpublishSharedInstance"
		/>
	</div>
</template>

<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import SharedInstanceInstallationSettingsControls from '@/components/ui/shared-instances/SharedInstanceInstallationSettingsControls.vue'
import { unpublish_shared_instance } from '@/helpers/instance'
import { injectInstanceSettings } from '@/providers/instance-settings'

const { instance, offline, onUnlinked } = injectInstanceSettings()
const { handleError } = injectNotificationManager()
const queryClient = useQueryClient()
const unpublishing = ref(false)
const isBusy = computed(
	() => instance.value.install_stage !== 'installed' || unpublishing.value || !!offline,
)

async function unpublishSharedInstance() {
	unpublishing.value = true
	try {
		await unpublish_shared_instance(instance.value.id)
		queryClient.setQueryData(['sharedInstanceUsers', instance.value.id], [])
		await queryClient.invalidateQueries({ queryKey: ['linkedModpackInfo', instance.value.id] })
		onUnlinked()
	} catch (error) {
		handleError(error)
	} finally {
		unpublishing.value = false
	}
}
</script>
