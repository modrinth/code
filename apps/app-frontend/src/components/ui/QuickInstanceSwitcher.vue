<script setup>
import { SpinnerIcon } from '@modrinth/assets'
import { Avatar, injectNotificationManager } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import dayjs from 'dayjs'
import { computed, onMounted, onUnmounted, ref } from 'vue'

import NavButton from '@/components/ui/NavButton.vue'
import { instance_listener } from '@/helpers/events.js'
import { list } from '@/helpers/instance'

const ITEM_SIZE = 52
const USED_VERTICAL_SPACE = 538

const { handleError } = injectNotificationManager()

const maxVisible = ref(0)
const allInstances = ref([])

const recentInstances = computed(() => allInstances.value.slice(0, maxVisible.value))

const updateMaxVisible = () => {
	maxVisible.value = Math.max(0, Math.floor((window.innerHeight - USED_VERTICAL_SPACE) / ITEM_SIZE))
}

const getInstances = async () => {
	const instances = await list().catch(handleError)

	allInstances.value = instances.sort((a, b) => {
		const dateACreated = dayjs(a.created)
		const dateAPlayed = a.last_played ? dayjs(a.last_played) : dayjs(0)

		const dateBCreated = dayjs(b.created)
		const dateBPlayed = b.last_played ? dayjs(b.last_played) : dayjs(0)

		const dateA = dateACreated.isAfter(dateAPlayed) ? dateACreated : dateAPlayed
		const dateB = dateBCreated.isAfter(dateBPlayed) ? dateBCreated : dateBPlayed

		if (dateA.isSame(dateB)) {
			return a.name.localeCompare(b.name)
		}

		return dateB - dateA
	})
}

await getInstances()
updateMaxVisible()

const unlistenInstance = await instance_listener(async (event) => {
	if (event.event !== 'synced') {
		await getInstances()
	}
})

onMounted(() => {
	window.addEventListener('resize', updateMaxVisible)
})

onUnmounted(() => {
	window.removeEventListener('resize', updateMaxVisible)
	unlistenInstance()
})
</script>

<template>
	<div v-for="instance in recentInstances" :key="instance.id" v-tooltip.right="instance.name">
		<NavButton :to="`/instance/${encodeURIComponent(instance.id)}`" class="relative">
			<Avatar
				:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
				size="28px"
				:tint-by="instance.id"
				:class="`transition-all ${instance.install_stage !== 'installed' ? `brightness-[0.25] scale-[0.85]` : `group-hover:brightness-75`}`"
			/>
			<div
				v-if="instance.install_stage !== 'installed'"
				class="absolute inset-0 flex items-center justify-center z-10 pointer-events-none"
			>
				<SpinnerIcon class="animate-spin w-4 h-4" />
			</div>
		</NavButton>
	</div>
	<div v-if="recentInstances.length > 0" class="h-px w-6 mx-auto my-2 bg-divider"></div>
</template>

<style scoped lang="scss"></style>
