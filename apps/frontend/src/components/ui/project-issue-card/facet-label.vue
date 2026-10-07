<template>
	<span class="flex min-w-0 flex-col gap-1">
		<span>{{ label }}</span>
		<span v-if="details" class="break-words text-xs text-secondary">{{ details }}</span>
	</span>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { issueTargetLabels } from '@modrinth/moderation/src/data/issues/component-builders/targets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		facet: Labrinth.Threads.v3.ThreadIssueFacet
		project?: Labrinth.Projects.v3.Project
		members?: Labrinth.Projects.v3.TeamMember[]
	}>(),
	{ project: undefined, members: () => [] },
)
const { formatMessage } = useVIntl()
const messages = defineMessages({
	checkbox: {
		id: 'thread-issues.facet.acknowledge-checkbox',
		defaultMessage: 'Acknowledge this request',
	},
	reply: {
		id: 'thread-issues.facet.acknowledge-reply',
		defaultMessage: 'Explain your changes in a reply',
	},
})
const versionMessages = defineMessages({
	remove: { id: 'thread-issues.facet.version.remove', defaultMessage: 'Remove this version' },
	modify_environment: {
		id: 'thread-issues.facet.version.environment',
		defaultMessage: 'Change the version environment',
	},
	modify_game_versions: {
		id: 'thread-issues.facet.version.game-versions',
		defaultMessage: 'Change the supported game versions',
	},
	modify_dependencies: {
		id: 'thread-issues.facet.version.dependencies',
		defaultMessage: 'Change the version dependencies',
	},
	modify_changelog: {
		id: 'thread-issues.facet.version.changelog',
		defaultMessage: 'Change the version changelog',
	},
	remove_additional_files: {
		id: 'thread-issues.facet.version.remove-files',
		defaultMessage: 'Remove the requested additional files',
	},
	modify_additional_file_type: {
		id: 'thread-issues.facet.version.file-type',
		defaultMessage: 'Change the additional file type',
	},
})
const label = computed(() => {
	const { what } = props.facet
	if (what.type === 'version') return formatMessage(versionMessages[what.value.target.type])
	if (what.type === 'acknowledge') return formatMessage(messages[what.value.mode])
	return formatMessage(issueTargetLabels[what.type])
})
const details = computed(() => {
	const { what } = props.facet
	switch (what.type) {
		case 'version':
			return [
				what.value.version_number,
				what.value.target.type === 'modify_additional_file_type'
					? what.value.target.value.filename
					: undefined,
			]
				.filter(Boolean)
				.join(' · ')
		case 'remove_tags':
			return what.value.tags.join(', ')
		case 'modify_links':
			return Object.keys(what.value.links).join(', ')
		case 'modify_gallery_image':
			return props.project?.gallery.find((image) => image.id === what.value.image_id)?.name
		case 'modify_team_member_role':
			return props.members.find(
				(member) => member.team_id === what.value.team_id && member.user.id === what.value.user_id,
			)?.user.username
		default:
			return undefined
	}
})
</script>
