<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { NavTabs } from '@modrinth/ui'
import { computed, ref, watchEffect } from 'vue'

import GridDisplay from '@/components/ui/library/grid-display.vue'
import { get_project_v3_many } from '@/helpers/cache.js'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

const tabs = [
	{ label: 'All instances', href: 'all' },
	{ label: 'Modpacks', href: 'modpacks' },
	{ label: 'Servers', href: 'servers' },
	{ label: 'Custom', href: 'custom' },
]

const activeTab = ref(0)
const serverProjectIds = ref(new Set<string>())
const linkedInstances = computed(() => props.instances.filter((instance) => instance.link))

watchEffect(async () => {
	const projectIds = [
		...new Set(
			linkedInstances.value.flatMap((instance) =>
				instance.link?.project_id ? [instance.link.project_id] : [],
			),
		),
	]

	if (projectIds.length === 0) {
		serverProjectIds.value = new Set()
		return
	}

	try {
		const projects = (await get_project_v3_many(
			projectIds,
			'must_revalidate',
		)) as Labrinth.Projects.v3.Project[]
		serverProjectIds.value = new Set(
			projects.filter((project) => project?.minecraft_server != null).map((project) => project.id),
		)
	} catch {
		serverProjectIds.value = new Set()
	}
})

const filteredInstances = computed(() => {
	switch (tabs[activeTab.value].href) {
		case 'modpacks':
			return linkedInstances.value.filter(
				(instance) => !serverProjectIds.value.has(instance.link?.project_id ?? ''),
			)
		case 'servers':
			return linkedInstances.value.filter((instance) =>
				serverProjectIds.value.has(instance.link?.project_id ?? ''),
			)
		case 'custom':
			return props.instances.filter((instance) => !instance.link)
		default:
			return props.instances
	}
})
</script>

<template>
	<section class="flex flex-col gap-3">
		<h2 class="m-0 text-lg font-bold text-primary">Library</h2>
		<GridDisplay label="Instances" :instances="filteredInstances" />
	</section>
</template>
