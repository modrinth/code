<script setup lang="ts">
import { isStaff } from '@modrinth/utils'
import { computed } from 'vue'

import { useAuthState } from '~/composables/auth'
import { useCreatorProjectStats } from '~/composables/creator-project-stats'

import ProjectStatusStats from './ProjectStatusStats.vue'

const props = defineProps<{
	id: string
	kind: 'user' | 'organization'
}>()
const auth = useAuthState()
const enabled = computed(() => isStaff(auth.value.user) && !!props.id)
const { memberStats, organizationStats } = useCreatorProjectStats(
	computed(() => (props.kind === 'user' ? [props.id] : [])),
	computed(() => (props.kind === 'organization' ? props.id : '')),
	enabled,
)
const stats = computed(() =>
	props.kind === 'user' ? (memberStats.value[props.id] ?? []) : organizationStats.value,
)
</script>

<template>
	<ProjectStatusStats v-if="enabled && stats.some((stat) => stat.count > 0)" :stats="stats" />
</template>
