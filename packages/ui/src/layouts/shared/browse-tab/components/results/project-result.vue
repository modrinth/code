<script setup lang="ts">
import ProjectCard from '#ui/components/project/card/ProjectCard.vue'

import { injectBrowseManager } from '../../providers/browse-manager'
import type { BrowseHit, CardAction } from '../../types'
import CardActions from './card-actions.vue'

const props = defineProps<{
	hit: BrowseHit
	actions: CardAction[]
}>()

const ctx = injectBrowseManager()
const { view } = ctx

function loaderFieldValues(field: string): string[] {
	return (props.hit.project_loader_fields?.[field] ?? []).filter(
		(value): value is string => typeof value === 'string',
	)
}

function tags(displayOnly: boolean) {
	const result = new Set(displayOnly ? props.hit.display_categories : props.hit.categories)
	for (const loader of props.hit.loaders) {
		if (loader !== 'mrpack') result.add(loader)
	}
	if (props.hit.loaders.includes('mrpack')) {
		for (const loader of loaderFieldValues('mrpack_loaders')) result.add(loader)
	}
	return Array.from(result)
}
</script>

<template>
	<ProjectCard
		:link="ctx.links.project(hit)"
		:title="hit.name"
		:icon-url="hit.icon_url ?? undefined"
		:author="{
			name: hit.organization == null ? hit.author : hit.organization,
			link:
				hit.organization_id == null
					? `/user/${encodeURIComponent(hit.author_id ?? hit.author)}`
					: ctx.links.organization(hit.organization_id),
		}"
		:date-updated="hit.date_modified"
		:date-published="hit.date_created"
		:displayed-date="view.sort.value.name === 'newest' ? 'published' : 'updated'"
		:downloads="hit.downloads"
		:summary="hit.summary"
		:tags="tags(true)"
		:all-tags="tags(false)"
		:deprioritized-tags="view.deprioritizedTags.value"
		:exclude-loaders="view.excludeLoaders.value"
		:followers="hit.follows"
		:banner="hit.featured_gallery ?? undefined"
		:color="hit.color ?? undefined"
		:environment="
			['mod', 'modpack'].includes(ctx.projectType.value)
				? hit.project_loader_fields?.environment?.[0]
				: undefined
		"
		:layout="view.layout.value"
		@contextmenu.prevent.stop="(event: MouseEvent) => ctx.onContextMenu?.(event, hit)"
		@mouseenter="ctx.onHover?.(hit)"
		@mouseleave="ctx.onHoverEnd?.()"
	>
		<template v-if="actions.length" #actions>
			<CardActions :actions="actions" />
		</template>
	</ProjectCard>
</template>
