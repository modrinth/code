<script setup lang="ts">
import { computed } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'
import type { UninstalledSharedInstance } from '@/components/ui/library/use-uninstalled-shared-instances'

import HiddenInstanceRow from './row.vue'

const props = defineProps<{ instance: UninstalledSharedInstance }>()

const {
	getUninstalledSharedInstanceOptions,
	installUninstalledSharedInstance,
	loadingSharedInstanceIds,
} = useLibrary()

const options = computed(() => getUninstalledSharedInstanceOptions(props.instance))
const loading = computed(() =>
	loadingSharedInstanceIds.value.has(props.instance.shared_instance_id),
)
</script>

<template>
	<HiddenInstanceRow
		:instance="instance"
		:options="options"
		:loading="loading"
		@activate="(event) => installUninstalledSharedInstance(instance, event)"
	/>
</template>
