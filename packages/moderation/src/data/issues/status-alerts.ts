import { MegaphoneIcon } from '@modrinth/assets'

import accountIssuesMessage from '../messages/checklist/messages/status-alerts/account-issues.md'
import correctionsAppliedMessage from '../messages/checklist/messages/status-alerts/corrections-applied.md'
import correctionsAppliedApprovedMessage from '../messages/checklist/messages/status-alerts/corrections-applied-approved.md'
import demonetizedMessage from '../messages/checklist/messages/status-alerts/demonetized.md'
import demonetizedModpackMessage from '../messages/checklist/messages/status-alerts/demonetized-modpack.md'
import privateUseNoteSharedInstanceMessage from '../messages/checklist/messages/status-alerts/private-use/note/shared-instance.md'
import privateUseProjectMessage from '../messages/checklist/messages/status-alerts/private-use/project.md'
import privateUseServerMessage from '../messages/checklist/messages/status-alerts/private-use/server.md'
import serverUseMessage from '../messages/checklist/messages/status-alerts/server-use.md'
import temporaryServerMessage from '../messages/checklist/messages/status-alerts/temporary-server.md'
import { issue, panel, toggle } from './component-builders/builders'

export const statusAlertsCorrectionsAppliedIssue = issue({
	id: 'status-alerts-corrections-applied',
	title: 'Apply selected corrections',
	category: 'Project wide',
	message: ({ ProjectV3 }) =>
		ProjectV3.status === 'approved' ? correctionsAppliedApprovedMessage : correctionsAppliedMessage,
	suggestedStatus: 'approved',
	applyCorrections: true,
})

export const statusAlertsPrivateUseIssue = issue({
	id: 'status-alerts-private-use',
	title: 'Private-use project',
	category: 'Project wide',
	message: ({ ProjectV3 }) => {
		const serverPack = ProjectV3.minecraft_java_server?.content?.kind === 'modpack'
		return [
			serverPack ? privateUseServerMessage : privateUseProjectMessage,
			serverPack || ProjectV3.project_types.includes('modpack')
				? privateUseNoteSharedInstanceMessage
				: '',
		].join('\n')
	},
	suggestedStatus: 'flagged',
})

export const statusAlertsTemporaryServerIssue = issue({
	id: 'status-alerts-temporary-server',
	title: 'Temporary server',
	category: 'Project wide',
	message: temporaryServerMessage,
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
	title: 'Demonetized project',
	category: 'Project wide',
	message: demonetizedMessage,
})

export const statusAlertsDemonetizedModpackIssue = issue({
	id: 'status-alerts-demonetized-modpack',
	title: 'Demonetized modpack',
	category: 'Project wide',
	message: demonetizedModpackMessage,
})

export const statusAlertsReviewPanel = panel({
	title: 'Status Alerts',
	hint: "Is anything else affecting this project's status?",
	icon: MegaphoneIcon,
}).content(
	toggle({
		issue: statusAlertsCorrectionsAppliedIssue,
		label: 'Corrections applied',
	}),
	toggle({
		issue: statusAlertsPrivateUseIssue,
		label: 'Private use',
	}),
	toggle({
		issue: statusAlertsTemporaryServerIssue,
		label: 'Temporary server',
		shown: ({ ProjectV3 }) =>
			['aternos', 'minekeep', 'minehut'].some((host) =>
				ProjectV3.minecraft_java_server?.address?.includes(host),
			),
	}),
	toggle({
		issue: statusAlertsServerUseIssue,
		label: 'Server use',
		shown: ({ ProjectV3 }) =>
			ProjectV3.project_types.includes('modpack') && !ProjectV3.minecraft_server,
	}),
	toggle({
		issue: statusAlertsAccountIssuesIssue,
		label: 'Account issues',
	}),
	toggle({
		issue: statusAlertsDemonetizedIssue,
		label: 'Demonetized',
		shown: ({ ProjectV3 }) =>
			ProjectV3.monetization_status === 'force-demonetized' &&
			!ProjectV3.project_types.includes('modpack') &&
			!ProjectV3.minecraft_server,
	}),
	toggle({
		issue: statusAlertsDemonetizedModpackIssue,
		label: 'Demonetized',
		shown: ({ ProjectV3 }) =>
			ProjectV3.monetization_status === 'force-demonetized' &&
			ProjectV3.project_types.includes('modpack') &&
			!ProjectV3.minecraft_server,
	}),
)
