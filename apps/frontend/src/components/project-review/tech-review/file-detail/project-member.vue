<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CrownIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'

import ProjectStatusStats from '~/components/ProjectStatusStats.vue'

defineProps<{
	member: Labrinth.Projects.v3.TeamMember
	stats: { status: string; count: number }[]
}>()
</script>

<template>
	<li class="flex min-w-0 items-center gap-2 pl-1">
		<Avatar
			:src="member.user.avatar_url"
			:alt="member.user.username"
			size="1.5rem"
			circle
			no-shadow
		/>
		<div class="min-w-0">
			<div class="flex min-w-0 items-center gap-1 text-xs">
				<NuxtLink
					:to="`/user/${member.user.username}`"
					target="_blank"
					rel="noopener noreferrer"
					class="min-w-0 truncate font-medium text-primary hover:underline"
					:title="member.user.username"
				>
					{{ member.user.username }}
				</NuxtLink>
				<CrownIcon
					v-if="member.is_owner"
					class="size-3 shrink-0 text-secondary"
					aria-label="Owner"
				/>
			</div>
			<p class="m-0 truncate text-xs text-secondary">{{ member.role }}</p>
		</div>
		<ProjectStatusStats :stats="stats" class="!mt-0 shrink-0 items-center !gap-x-1" />
	</li>
</template>
