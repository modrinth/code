import { useQuery } from '@tanstack/vue-query'
import { computed } from 'vue'

import { maxMemoryQueryOptions } from '@/helpers/jre.js'

export default function () {
	const memoryQuery = useQuery(maxMemoryQueryOptions())
	const maxMemory = computed(() => Math.floor((memoryQuery.data.value ?? 0) / 1024))

	const snapPoints = computed(() => {
		let points = []
		let memory = 2048

		while (memory <= maxMemory.value) {
			points.push(memory)
			memory *= 2
		}

		return points
	})

	return { maxMemory, snapPoints, memoryQuery }
}
