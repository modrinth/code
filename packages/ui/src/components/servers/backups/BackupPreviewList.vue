<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { onMounted, ref } from 'vue'

import { useScrollIndicator } from '../../../composables/scroll-indicator'
import BackupItem from './BackupItem.vue'

defineProps<{
	backups: Archon.BackupsQueue.v1.BackupQueueBackup[]
	label: string
}>()

const listRef = ref<HTMLElement | null>(null)
const { showTopFade, showBottomFade, checkScrollState, forceCheck } = useScrollIndicator(listRef)

onMounted(forceCheck)
</script>

<template>
	<div class="flex min-w-0 flex-col gap-2">
		<span class="font-semibold text-contrast">{{ label }}</span>
		<div class="relative">
			<Transition
				enter-active-class="transition-all duration-200 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-2"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-2"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="showTopFade"
					class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
				/>
			</Transition>
			<div
				ref="listRef"
				class="flex max-h-[240px] flex-col gap-2 overflow-y-auto"
				@scroll="checkScrollState"
			>
				<BackupItem v-for="backup in backups" :key="backup.id" :backup="backup" />
			</div>
			<Transition
				enter-active-class="transition-all duration-200 ease-out"
				enter-from-class="opacity-0 max-h-0"
				enter-to-class="opacity-100 max-h-2"
				leave-active-class="transition-all duration-200 ease-in"
				leave-from-class="opacity-100 max-h-2"
				leave-to-class="opacity-0 max-h-0"
			>
				<div
					v-if="showBottomFade"
					class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
				/>
			</Transition>
		</div>
	</div>
</template>
