<template>
	<div class="flex flex-wrap items-center gap-1 text-sm text-secondary">
		<time :datetime="rule.updated_at" :title="formatTimestamp(rule.updated_at)">
			Last edited {{ formatDate(rule.updated_at) }}
		</time>
		<BulletDivider aria-hidden="true" />
		<time :datetime="rule.created_at" :title="formatTimestamp(rule.created_at)">
			Created {{ formatDate(rule.created_at) }}
		</time>
		<BulletDivider aria-hidden="true" />
		<CopyCode
			v-tooltip="`Rule ID: ${rule.id}`"
			:text="String(rule.id)"
			title=""
			:aria-label="`Copy rule ID ${rule.id}`"
			@click.stop
		/>
	</div>
</template>

<script setup lang="ts">
import type { ProjectFlagRule } from '@modrinth/api-client'
import { BulletDivider, CopyCode } from '@modrinth/ui'

defineProps<{ rule: ProjectFlagRule }>()

function formatDate(value: string) {
	return new Date(value).toLocaleDateString('en-US', { dateStyle: 'medium', timeZone: 'UTC' })
}

function formatTimestamp(value: string) {
	return new Date(value).toLocaleString('en-US', {
		dateStyle: 'full',
		timeStyle: 'long',
		timeZone: 'UTC',
	})
}
</script>
