<script setup lang="ts">
import { computed } from 'vue'

import { useInstanceActions } from '@/features/instances/use-instance-actions'
import { useInstanceMenu } from '@/features/instances/use-instance-menu'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import type { GameInstance } from '@/helpers/types'

import HiddenInstanceRow from './row.vue'

const props = defineProps<{ instance: GameInstance }>()

const settingsModal = injectAppSettingsModal()
const { getInstanceOptions } = useInstanceMenu()
const { card, playing, modLoading, installing, seeInstance } = useInstanceActions(
	() => props.instance,
	{
		beforeNavigate: () => settingsModal.close(),
	},
)

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
