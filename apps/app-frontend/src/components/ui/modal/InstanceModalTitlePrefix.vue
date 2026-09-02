<script setup lang="ts">
import { ChevronRightIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'

import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

defineProps<{
	instance: GameInstance
}>()
</script>
<template>
	<span class="flex items-center gap-2 text-lg font-semibold text-primary">
		<Avatar :src="getInstanceIconUrl(instance.icon_path)" size="24px" :tint-by="instance.id" />
		{{ instance.name }} <ChevronRightIcon />
	</span>
</template>
