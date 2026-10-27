<script setup lang="ts">
import { computed } from 'vue'

import { useLibrary } from '@/components/ui/library/use-library'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { useInstanceActions } from '@/features/instances/use-instance-actions'
import type { GameInstance } from '@/helpers/types'

import HiddenInstanceRow from './row.vue'

const props = defineProps<{ instance: GameInstance }>()

const settingsModal = injectAppSettingsModal()
const { getInstanceOptions } = useLibrary()
const { card, playing, modLoading, installing, seeInstance } = useInstanceActions(() => props.instance, {
	beforeNavigate: () => settingsModal.close(),
})

const options = computed(() => getInstanceOptions(card.value))
const loading = computed(() => !playing.value && (modLoading.value || installing.value))
</script>

<template>
	<HiddenInstanceRow
		:instance="instance"
		:options="options"
		:loading="loading"
		@activate="seeInstance"
	/>
</template>
