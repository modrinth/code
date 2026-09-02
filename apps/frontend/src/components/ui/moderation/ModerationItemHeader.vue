<script setup lang="ts">
import { Avatar } from '@modrinth/ui'

import ModerationOwnerLink, { type ModerationOwner } from './ModerationOwnerLink.vue'
export type { ModerationOwner }

defineProps<{
	avatarUrl?: string | null
	title: string
	titleTo?: string | null
	owner?: ModerationOwner | null
	circle?: boolean
	padTransparentCorners?: boolean
}>()
</script>

<template>
	<div class="flex items-center gap-3">
		<NuxtLink v-if="titleTo" :to="titleTo" target="_blank" tabindex="-1">
			<Avatar
				:src="avatarUrl"
				:circle="circle"
				:pad-transparent-corners="padTransparentCorners"
				no-shadow
				size="4rem"
			/>
		</NuxtLink>
		<Avatar
			v-else
			:src="avatarUrl"
			:circle="circle"
			:pad-transparent-corners="padTransparentCorners"
			no-shadow
			size="4rem"
		/>

		<div class="flex flex-col gap-1.5">
			<div class="flex flex-wrap items-center gap-2">
				<NuxtLink
					v-if="titleTo"
					:to="titleTo"
					target="_blank"
					class="text-lg font-semibold text-contrast hover:underline focus-visible:underline"
				>
					{{ title }}
				</NuxtLink>
				<span v-else class="text-lg font-semibold text-contrast">
					{{ title }}
				</span>
				<slot name="badges" />
			</div>

			<ModerationOwnerLink v-if="owner" :owner="owner" />
			<slot name="subtitle" />
		</div>
	</div>
</template>
