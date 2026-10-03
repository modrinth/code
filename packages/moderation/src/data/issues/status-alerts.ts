import { MegaphoneIcon } from '@modrinth/assets'

import accountIssuesMessage from '../messages/checklist/messages/status-alerts/account-issues.md'
import demonetizedMessage from '../messages/checklist/messages/status-alerts/demonetized.md'
import demonetizedModpackMessage from '../messages/checklist/messages/status-alerts/demonetized-modpack.md'
import privateUseNoteSharedInstanceMessage from '../messages/checklist/messages/status-alerts/private-use/note/shared-instance.md'
import privateUseProjectMessage from '../messages/checklist/messages/status-alerts/private-use/project.md'
import privateUseServerMessage from '../messages/checklist/messages/status-alerts/private-use/server.md'
import serverUseMessage from '../messages/checklist/messages/status-alerts/server-use.md'
import { issue, panel, toggle } from './component-builders/builders'
import { IssuePriority } from './component-builders/priority'

export const statusAlertsPrivateUseIssue = issue({
	id: 'status-alerts-private-use',
	priority: IssuePriority.Alerts,
	title: 'Private-use project',
	category: 'Project wide',
	message: ({ projectV3 }) => {
		const serverPack = projectV3.minecraft_java_server?.content?.kind === 'modpack'
		return [
			serverPack ? privateUseServerMessage : privateUseProjectMessage,
			serverPack || projectV3.project_types.includes('modpack')
				? privateUseNoteSharedInstanceMessage
				: '',
		].join('\n')
	},
	suggestedStatus: 'flagged',
})

export const statusAlertsServerUseIssue = issue({
	id: 'status-alerts-server-use',
	title: 'Server-use project',
	category: 'Project wide',
	message: serverUseMessage,
})

export const statusAlertsAccountIssuesIssue = issue({
	id: 'status-alerts-account-issues',
	title: 'Account issues',
	category: 'Project wide',
	message: accountIssuesMessage,
	suggestedStatus: 'rejected',
})

export const statusAlertsDemonetizedIssue = issue({
	id: 'status-alerts-demonetized',
	priority: IssuePriority.Alerts,
	title: 'Demonetized project',
	category: 'Project wide',
	message: demonetizedMessage,
})

export const statusAlertsDemonetizedModpackIssue = issue({
	id: 'status-alerts-demonetized-modpack',
	priority: IssuePriority.Alerts,
	title: 'Demonetized modpack',
	category: 'Project wide',
	message: demonetizedModpackMessage,
})

export const statusAlertsReviewPanel = panel({
	title: 'Status Alerts',
	hint: "Is anything else affecting this project's status?",
	icon: MegaphoneIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080968699c397e470eca6',
}).content(
	toggle({
		issue: statusAlertsPrivateUseIssue,
		label: 'Private use',
	}),
	toggle({
		issue: statusAlertsServerUseIssue,
		label: 'Server use',
		shown: ({ projectV3 }) =>
			projectV3.project_types.includes('modpack') && !projectV3.minecraft_server,
	}),
	toggle({
		issue: statusAlertsAccountIssuesIssue,
		label: 'Account issues',
	}),
	toggle({
		issue: statusAlertsDemonetizedIssue,
		label: 'Demonetized',
		shown: ({ projectV3 }) =>
			projectV3.monetization_status === 'force-demonetized' &&
			!projectV3.project_types.includes('modpack') &&
			!projectV3.minecraft_server,
	}),
	toggle({
		issue: statusAlertsDemonetizedModpackIssue,
		label: 'Demonetized',
		shown: ({ projectV3 }) =>
			projectV3.monetization_status === 'force-demonetized' &&
			projectV3.project_types.includes('modpack') &&
			!projectV3.minecraft_server,
	}),
)
