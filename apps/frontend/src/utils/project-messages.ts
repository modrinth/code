import type { Labrinth } from '@modrinth/api-client'
import { defineMessage, defineMessages } from '@modrinth/ui'

export const projectSummaryHelp = defineMessage({
	id: 'project.summary.search-description',
	defaultMessage: 'One sentence shown under your project name in search results.',
})

export const projectMessages = defineMessages({
	summaryHelp: {
		id: 'project.summary.discovery-help',
		defaultMessage:
			'The summary is shown in your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}’s Discovery card and project page. It should provide an appealing introduction to your {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}}.',
	},
	visibilityApprovalHelp: {
		id: 'project.visibility.initial-review-help',
		defaultMessage:
			'This {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} still needs to be approved by Moderation. Until then, only members can see it. When ready, you may submit this {contentType, select, mod {mod} modpack {modpack} resourcepack {resource pack} shader {shader} plugin {plugin} datapack {data pack} server {server} other {project}} for review.',
	},
})

const visibilityMessages = defineMessages({
	publicLabel: {
		id: 'create.project.visibility-public',
		defaultMessage: 'Public',
	},
	publicDescription: {
		id: 'project.visibility.public.discovery-description',
		defaultMessage: 'Visible for Discovery on Modrinth, the Modrinth App, and search engines.',
	},
	unlistedLabel: {
		id: 'create.project.visibility-unlisted',
		defaultMessage: 'Unlisted',
	},
	unlistedDescription: {
		id: 'project.visibility.unlisted.discovery-description',
		defaultMessage: 'Visible to anyone with a link. Only appears in the Modrinth App if installed.',
	},
	privateLabel: {
		id: 'create.project.visibility-private',
		defaultMessage: 'Private',
	},
	privateDescription: {
		id: 'project.visibility.private.discovery-description',
		defaultMessage: 'Only visible to project members. Does not appear in Discovery.',
	},
})

export const projectVisibilityOptions = {
	approved: {
		label: visibilityMessages.publicLabel,
		description: visibilityMessages.publicDescription,
	},
	unlisted: {
		label: visibilityMessages.unlistedLabel,
		description: visibilityMessages.unlistedDescription,
	},
	private: {
		label: visibilityMessages.privateLabel,
		description: visibilityMessages.privateDescription,
	},
}

export function projectCopyType(project: Labrinth.Projects.v3.Project) {
	if (project.minecraft_server != null) return 'server'
	return project.project_types?.length === 1 ? project.project_types[0] : 'project'
}
