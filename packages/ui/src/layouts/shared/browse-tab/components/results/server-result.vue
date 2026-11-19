<script setup lang="ts">
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'

import { injectBrowseManager } from '../../providers/browse-manager'
import type { BrowseHit, CardAction } from '../../types'
import CardActions from './card-actions.vue'

defineProps<{
	hit: BrowseHit
	actions: CardAction[]
	dense?: boolean
}>()

const ctx = injectBrowseManager()
</script>

<template>
	<ProjectCard
		:title="hit.name"
		:icon-url="hit.icon_url || undefined"
		:summary="hit.summary"
		:tags="hit.categories"
		:link="ctx.links.server(hit)"
		:server-online-players="hit.minecraft_java_server?.ping?.data?.players_online ?? 0"
		:server-region="hit.minecraft_server?.region"
		:server-recent-plays="hit.minecraft_java_server?.verified_plays_2w ?? 0"
		:server-modpack-content="ctx.serverModpackContent?.(hit)"
		:server-ping="ctx.serverPings?.value[hit.project_id]"
		:server-status-online="!!hit.minecraft_java_server?.ping?.data"
		:hide-online-players-label="dense"
		:hide-recent-plays-label="dense"
		:layout="ctx.view.layout.value"
		:max-tags="2"
		is-server-project
		exclude-loaders
		:color="hit.color ?? undefined"
		:banner="hit.featured_gallery ?? undefined"
		@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, hit)"
		@mouseenter="ctx.onHover?.(hit)"
		@mouseleave="ctx.onHoverEnd?.()"
	>
		<template v-if="actions.length" #actions>
			<CardActions :actions="actions" />
		</template>
	</ProjectCard>
</template>
