import { computed, ref } from 'vue'

import { get_max_memory } from '@/helpers/jre.js'

export default async function () {
	const maxMemory = ref(Math.floor((await get_max_memory()) / 1024))

	const snapPoints = computed(() => {
		let points = []
		let memory = 2048

		while (memory <= maxMemory.value) {
			points.push(memory)
			memory *= 2
		}

		return points
	})

	return { maxMemory, snapPoints }
}
