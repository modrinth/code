<script setup lang="ts">
import { Avatar, CopyCode } from '@modrinth/ui'

export type ModerationOwner = {
	kind: 'user' | 'organization'
	id: string
	name: string
	icon_url?: string | null
}

defineProps<{
	owner: ModerationOwner
}>()
</script>

<template>
	<div class="flex items-center gap-2">
		<NuxtLink
			:to="`/${owner.kind}/${owner.id}`"
			target="_blank"
			class="flex items-center gap-1 text-sm font-medium text-secondary hover:underline"
		>
			<Avatar :src="owner.icon_url" :circle="owner.kind === 'user'" size="1.5rem" no-shadow />
			{{ owner.name }}
		</NuxtLink>
		<ClientOnly>
			<CopyCode
				v-tooltip="owner.kind === 'organization' ? 'Copy organization ID' : 'Copy user ID'"
				:text="owner.id"
			/>
		</ClientOnly>
	</div>
</template>
