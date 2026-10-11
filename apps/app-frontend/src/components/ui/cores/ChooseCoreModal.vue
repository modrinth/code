<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { Button, injectModrinthClient, NewModal } from '@modrinth/ui'
import { ref } from 'vue'

import { ensureCorePaired } from '@/composables/use-cores'

// Decides which Core a new server goes on. Mount it where servers are created and call `pick`.

const client = injectModrinthClient()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const cores = ref<Archon.Cores.v1.Core[]>([])
let settle: ((core: Archon.Cores.v1.Core | null) => void) | null = null

/**
 * Resolves the Core to create a server on, or null if the user backs out. Pairs a Core first
 * when there is none, and only asks which one when there are several.
 */
async function pick(): Promise<Archon.Cores.v1.Core | null> {
	if (!(await ensureCorePaired())) return null
	cores.value = await client.archon.cores_v1.list().catch(() => [])
	if (cores.value.length <= 1) return cores.value[0] ?? null
	return new Promise((resolve) => {
		settle = resolve
		modal.value?.show()
	})
}

function choose(core: Archon.Cores.v1.Core | null) {
	settle?.(core)
	settle = null
	if (core) modal.value?.hide()
}

defineExpose({ pick })
</script>

<template>
	<NewModal
		ref="modal"
		header="Choose a Core"
		max-width="460px"
		width="100%"
		:on-hide="() => choose(null)"
	>
		<div class="flex flex-col gap-2">
			<Button v-for="core in cores" :key="core.id" class="w-full" @click="choose(core)">
				{{ core.name }}
			</Button>
		</div>
	</NewModal>
</template>
