<template>
	<div
		v-if="heldBy"
		class="flex shrink-0 items-center gap-1 rounded-xl border border-solid border-orange bg-highlight-orange p-2.5"
		role="status"
	>
		<Avatar :src="heldBy.avatar_url" :alt="heldBy.username" size="1.75rem" circle no-shadow />
		<span class="min-w-0 grow break-words text-contrast">
			<NuxtLink
				:to="`/user/${heldBy.username}`"
				target="_blank"
				rel="noopener noreferrer"
				class="font-semibold hover:underline"
			>
				{{ heldBy.username }}
			</NuxtLink>
			is moderating
		</span>
		<IconButton
			v-tooltip="'Take over project'"
			label="Take over project"
			type="colored"
			color="orange"
			size="xs"
			:disabled="lockPending || disabled"
			:loading="lockPending"
			@click="lock.takeOver()"
		>
			<SpinnerIcon v-if="lockPending" class="animate-spin" aria-hidden="true" />
			<LockIcon v-else aria-hidden="true" />
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import { LockIcon, SpinnerIcon } from '@modrinth/assets'
import { Avatar, IconButton } from '@modrinth/ui'

import { injectProjectReviewPageContext } from '~/providers/project-review'

defineProps<{
	disabled: boolean
}>()

const { lock } = injectProjectReviewPageContext()
const { heldBy, pending: lockPending } = lock
</script>
