import type { Labrinth } from '@modrinth/api-client'

import { flattenProjectVariables, flattenStaticVariables } from '../utils'
import warning from './messages/checklist/messages/post-approval/issue-warning.md'
import deadline from './messages/checklist/messages/post-approval/missed-deadline.md'
import account from './messages/checklist/messages/status-alerts/account-issues.md'
import demonetized from './messages/checklist/messages/status-alerts/demonetized.md'
import demonetizedModpack from './messages/checklist/messages/status-alerts/demonetized-modpack.md'
import privateProject from './messages/checklist/messages/status-alerts/private-use/project.md'
import privateServer from './messages/checklist/messages/status-alerts/private-use/server.md'
import serverUse from './messages/checklist/messages/status-alerts/server-use.md'
import temporaryServer from './messages/checklist/messages/status-alerts/temporary-server.md'

export interface MessageTemplate {
	label: string
	body: string
	fields?: { token: string; label: string }[]
}

export interface MessageLibraryContext {
	project: Labrinth.Projects.v3.Project
	projectV2: Labrinth.Projects.v2.Project
}

export interface MessageLibraryEntry extends MessageTemplate {
	shown?: (context: MessageLibraryContext) => boolean
}

export const messageLibrary: readonly MessageLibraryEntry[] = [
	{
		label: 'Private-use project',
		body: privateProject,
		shown: ({ project }) => !project.minecraft_server,
	},
	{
		label: 'Private-use server',
		body: privateServer,
		shown: ({ project }) => !!project.minecraft_server,
	},
	{
		label: 'Server-use project',
		body: serverUse,
		shown: ({ project }) => project.project_types.includes('modpack') && !project.minecraft_server,
	},
	{
		label: 'Demonetized project',
		body: demonetized,
		shown: ({ project }) =>
			project.monetization_status === 'force-demonetized' &&
			!project.project_types.includes('modpack') &&
			!project.minecraft_server,
	},
	{
		label: 'Demonetized modpack',
		body: demonetizedModpack,
		shown: ({ project }) =>
			project.monetization_status === 'force-demonetized' &&
			project.project_types.includes('modpack') &&
			!project.minecraft_server,
	},
	{
		label: 'Post-approval issue warning',
		body: warning,
		shown: ({ project }) => project.status === 'approved',
	},
	{
		label: 'Missed review deadline',
		body: deadline,
		shown: ({ project }) => project.status === 'approved',
		fields: [{ token: '%STATUS%', label: 'What status is the project being set to?' }],
	},
	{
		label: 'Temporary server',
		body: temporaryServer,
		shown: ({ project }) =>
			!!project.minecraft_server &&
			['aternos', 'minekeep', 'minehut'].some((host) =>
				project.minecraft_java_server?.address?.includes(host),
			),
	},
	{ label: 'Account issues', body: account },
]

export function getMessageTemplates(context: MessageLibraryContext): MessageTemplate[] {
	const variables = { ...flattenStaticVariables(), ...flattenProjectVariables(context.projectV2) }
	return messageLibrary
		.filter((template) => template.shown?.(context) ?? true)
		.map((template) => ({
			label: template.label,
			fields: template.fields,
			body: Object.entries(variables)
				.reduce((result, [key, value]) => result.replaceAll(`%${key}%`, () => value), template.body)
				.trim(),
		}))
}
