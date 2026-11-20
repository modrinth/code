<template>
	<article
		class="flex min-w-0 flex-col gap-2 rounded-xl border border-solid border-surface-3 bg-surface-2 p-2.5 text-sm"
	>
		<div class="flex items-center gap-2">
			<h3 class="m-0 flex min-w-0 flex-1 items-center gap-1.5 text-sm font-medium text-contrast">
				<TagCategoryFlagIcon class="size-4 shrink-0 text-purple" aria-hidden="true" />
				<span class="min-w-0 break-words">{{ flag.name || flag.key }}</span>
			</h3>
			<ButtonLink
				v-tooltip="'View flag rule'"
				type="quiet"
				size="xs"
				class="-my-1 -mr-1 !size-7 shrink-0 !rounded-full !p-0"
				href="/moderation/project-flag"
				target="_blank"
				rel="noopener noreferrer"
				aria-label="View flag rule"
			>
				<ExternalIcon class="!size-3.5 !min-h-0 !min-w-0" aria-hidden="true" />
			</ButtonLink>
		</div>
		<p v-if="flag.reason" class="m-0 whitespace-pre-wrap break-words text-primary">
			{{ flag.reason }}
		</p>
		<div v-if="flag.location" class="flex">
			<Button
				size="xs"
				type="quiet"
				class="!h-auto !min-h-0 !rounded-full !bg-surface-3 !px-2.5 !py-1 !text-xs !font-medium hover:!bg-surface-4"
				@click="emit('reveal', flag.location.review_panel_key)"
			>
				<PanelTopIcon class="!size-3.5 shrink-0" aria-hidden="true" />
				{{ locationTitle || flag.location.review_panel_key }}
			</Button>
		</div>
	</article>
</template>

<script setup lang="ts">
import type { ProjectFlag } from '@modrinth/api-client'
import { ExternalIcon, PanelTopIcon, TagCategoryFlagIcon } from '@modrinth/assets'
import { Button, ButtonLink } from '@modrinth/ui'

defineProps<{ flag: ProjectFlag; locationTitle?: string }>()
const emit = defineEmits<{ reveal: [key: string] }>()
</script>
