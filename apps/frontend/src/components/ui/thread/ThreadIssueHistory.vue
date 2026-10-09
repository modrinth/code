<template>
	<article
		class="issue-history relative grid min-w-0 grid-cols-[2rem_minmax(0,1fr)] gap-x-2 gap-y-1 px-4 py-2 [overflow-wrap:anywhere]"
		:class="
			settings.get(moderationSettings.General.PrivateMessageHighlight)
				? `before:pointer-events-none before:absolute before:inset-0 before:bg-orange before:opacity-5 before:content-['']`
				: ''
		"
	>
		<AutoLink
			v-if="author"
			:to="`/user/${author.username}`"
			class="self-start"
			tabindex="-1"
			aria-hidden="true"
		>
			<Avatar :src="author.avatar_url" size="2rem" circle :raised="raised" />
		</AutoLink>
		<div v-else class="flex size-8 items-center justify-center rounded-full bg-surface-3">
			<ThreadRoleBadge role="moderator" avatar class="!size-6" aria-hidden="true" />
		</div>
		<div class="min-w-0 text-primary">
			<div class="break-all align-middle leading-5">
				<AutoLink
					v-if="author"
					:to="`/user/${author.username}`"
					class="min-w-0 font-semibold [overflow-wrap:anywhere]"
					:class="author.role === 'admin' ? 'text-green' : 'text-orange'"
				>
					{{ author.username }}
				</AutoLink>
				<span v-else class="font-semibold text-orange">{{
					formatMessage(messages.moderator)
				}}</span>
				<ThreadRoleBadge :role="author ? author.role : 'moderator'" />
				{{ ' ' }}
				<IntlFormatted :message-id="messages.flagged" :values="{ count: entry.issues.length }" />
				<EyeOffIcon
					v-tooltip="formatMessage(messages.privateNote)"
					:aria-label="formatMessage(messages.privateNote)"
					class="mb-0.5 ml-1.5 inline-block align-middle text-orange"
				/>
			</div>
			<span v-tooltip="formatDateTime(entry.created)" class="mt-1 block text-xs text-secondary">{{
				relativeTime(entry.created)
			}}</span>
		</div>
		<div class="col-span-2 flex min-w-0 flex-col gap-2 pt-2">
			<div v-for="issue in entry.issues" :key="issue.id" class="min-w-0">
				<ProjectIssueCard :issues="[issue]" thread-history />
			</div>
		</div>
	</article>
</template>

<script setup lang="ts">
import { EyeOffIcon } from '@modrinth/assets'
import { moderationSettings } from '@modrinth/moderation'
import {
	AutoLink,
	Avatar,
	defineMessages,
	IntlFormatted,
	useFormatDateTime,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'

import ProjectIssueCard from '../project-issue-card/index.vue'
import ThreadRoleBadge from './ThreadRoleBadge.vue'
import type { ThreadIssueHistoryEntry } from './timeline'

defineProps<{
	entry: ThreadIssueHistoryEntry
	author?: { username: string; avatar_url?: string | null; role?: string }
	raised?: boolean
}>()

const settings = useModerationSettings()
const { formatMessage } = useVIntl()
const formatDateTime = useFormatDateTime({ timeStyle: 'short', dateStyle: 'long' })
const relativeTime = useRelativeTime()
const messages = defineMessages({
	flagged: {
		id: 'thread.issue-history.flagged',
		defaultMessage: 'flagged {count, plural, one {# issue} other {# issues}}',
	},
	moderator: { id: 'thread.message.moderator', defaultMessage: 'Moderator' },
	privateNote: {
		id: 'thread.issue-history.private-note',
		defaultMessage: 'Only visible to moderators',
	},
})
</script>
