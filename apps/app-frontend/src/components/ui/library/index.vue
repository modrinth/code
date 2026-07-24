<script setup lang="ts">
import { toRef } from 'vue'

import InstanceGroup from '@/components/ui/library/instance-group/index.vue'
import LibraryToolbar from '@/components/ui/library/library-toolbar/index.vue'
import { provideLibrary } from '@/components/ui/library/use-library'
import type { GameInstance } from '@/helpers/types'

const props = defineProps<{
	instances: GameInstance[]
}>()

provideLibrary(toRef(props, 'instances'))
</script>

<template>
	<section class="flex flex-col gap-3">
		<h2 class="m-0 text-2xl font-semibold text-contrast">Library</h2>
		<LibraryToolbar />
		<InstanceGroup />
	</section>
</template>
