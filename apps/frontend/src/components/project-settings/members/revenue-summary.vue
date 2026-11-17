<script setup lang="ts">
import { Card, defineMessages, useVIntl } from '@modrinth/ui'

import SettingsSection from '../settings-section.vue'
import type { MemberDraft } from './types'

defineProps<{
	members: MemberDraft[]
	totalWeight: number
	revenuePercentage: (member: MemberDraft) => number
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: { id: 'project.settings.members.revenue.title', defaultMessage: 'Revenue split' },
	empty: {
		id: 'project.settings.members.revenue.empty',
		defaultMessage: 'Set member monetization weights to distribute revenue.',
	},
})

const colors = [
	'var(--color-brand)',
	'var(--color-blue)',
	'var(--color-purple)',
	'var(--color-orange)',
]
function revenueColor(index: number) {
	return colors[index % colors.length]
}
</script>

<template>
	<SettingsSection :title="formatMessage(messages.title)">
		<Card class="settings-content-card">
			<div class="mb-4 flex h-3 overflow-hidden rounded-full bg-surface-4" aria-hidden="true">
				<div
					v-for="(member, index) in members"
					:key="member.user.id"
					:style="{ width: `${revenuePercentage(member)}%`, background: revenueColor(index) }"
				/>
			</div>
			<ul class="m-0 flex list-none flex-col gap-3 p-0">
				<li
					v-for="(member, index) in members"
					:key="member.user.id"
					class="flex items-center gap-2"
				>
					<span class="size-2 shrink-0 rounded-full" :style="{ background: revenueColor(index) }" />
					<span class="flex-1">{{ member.user.username }}</span>
					<span>{{ member.payouts_split }} ({{ revenuePercentage(member) }}%)</span>
				</li>
			</ul>
			<p v-if="!totalWeight" class="mb-0 text-secondary">{{ formatMessage(messages.empty) }}</p>
		</Card>
	</SettingsSection>
</template>
