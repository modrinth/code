<script setup lang="ts">
import { computed } from 'vue'

import type { SharedInstanceOwner } from '@/features/library/library-instance'
import { injectSharedInstanceActions } from '@/features/library/shared-instances/actions-context'
import type { UninstalledSharedInstance } from '@/features/library/shared-instances/types'
import { useSharedInstanceMenu } from '@/features/library/shared-instances/use-shared-instance-menu'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'

import HiddenInstanceRow from './row.vue'

const props = defineProps<{
	instance: UninstalledSharedInstance
	owner: SharedInstanceOwner | null
}>()

const settingsModal = injectAppSettingsModal()
const { install, loadingIds } = injectSharedInstanceActions()
const { getSharedInstanceOptions } = useSharedInstanceMenu()

const onInstalled = () => settingsModal.close()
const options = computed(() => getSharedInstanceOptions(props.instance, { onInstalled }))
const loading = computed(() => loadingIds.value.has(props.instance.shared_instance_id))
</script>

<template>
	<HiddenInstanceRow
		:instance="instance"
		:owner="owner"
		:options="options"
		:loading="loading"
		@activate="(event) => install(instance, { event, onInstalled })"
	/>
</template>
