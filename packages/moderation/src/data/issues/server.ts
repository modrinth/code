import { ServerIcon } from '@modrinth/assets'

import { issue, markdown, panel, section, toggle, text } from './component-builders/builders'
import excessiveLanguagesMessage from '../messages/checklist/messages/rules/excessive-languages.md'
import temporaryServerMessage from '../messages/checklist/messages/status-alerts/temporary-server.md'
import { metadataGameVersionsIssue } from './metadata'
import {
	reuploadIdentityVerificationServerIssue,
	reuploadRequestProofServerIssue,
	reuploadCustomPackProhibitedIssue,
	reuploadCustomPackVerificationIssue,
} from './reupload'
import { projectHasCustomServerModpack } from '../../utils'

export const serversExcessiveLanguagesIssue = issue({
	id: 'servers-excessive-languages',
	title: 'Excessive languages',
	category: 'Server details',
	message: excessiveLanguagesMessage,
	suggestedStatus: 'flagged',
})

// Temp servers
export const serversTemporaryServerIssue = issue({
	id: 'status-alerts-temporary-server',
	title: 'Temporary server',
	category: 'Project wide',
	message: temporaryServerMessage,
	suggestedStatus: 'flagged',
})

export const serverReviewPanel = panel({
	title: 'Server details',
	hint: "Are there any issues with this project's server details?",
	icon: ServerIcon,
	guidanceUrl: '',
	shown: ({ ProjectV3 }) => !!ProjectV3.minecraft_server,
}).content(
	toggle({
		issue: serversExcessiveLanguagesIssue,
		label: 'Excessive languages',
		id: 'servers-excessive-languages',
	}),
	toggle({
		issue: serversTemporaryServerIssue,
		label: 'Temporary server',
		shown: ({ ProjectV3 }) =>
			['aternos', 'minekeep', 'minehut'].some((host) =>
				ProjectV3.minecraft_java_server?.address?.includes(host),
			),
	}),
	toggle({
		issue: reuploadIdentityVerificationServerIssue,
		label: 'Verify Identity',
	}),

	toggle({
		issue: metadataGameVersionsIssue,
		label: 'Game Versions',
	}),
	section({
		shown: (ctx) =>
			ctx.selected.issueIds.includes(reuploadIdentityVerificationServerIssue.id) &&
			!!ctx.ProjectV3.minecraft_server,
	}).content(
		text({
			issue: reuploadIdentityVerificationServerIssue,
			id: 'contact',
			label: 'Known public contact method',
			required: true,
		}),
	),
	section({
		label: 'Custom Modpack',
		shown: (ctx) => projectHasCustomServerModpack(ctx.ProjectV3),
	}).content(
		toggle({
			issue: reuploadRequestProofServerIssue,
			label: 'Reuploaded pack',
			shown: (ctx) => projectHasCustomServerModpack(ctx.ProjectV3),
		}),
		toggle({
			issue: reuploadCustomPackProhibitedIssue,
			label: 'Forbidden Overrides',
			shown: (ctx) => projectHasCustomServerModpack(ctx.ProjectV3),
		}),
		toggle({
			issue: reuploadCustomPackVerificationIssue,
			label: 'Override verification',
			shown: (ctx) => projectHasCustomServerModpack(ctx.ProjectV3),
		}),
		section({
			shown: (ctx) =>
				ctx.selected.issueIds.includes(reuploadCustomPackProhibitedIssue.id) &&
				projectHasCustomServerModpack(ctx.ProjectV3),
		}).content(
			markdown({
				issue: reuploadCustomPackProhibitedIssue,
				id: 'overrides',
				label: 'Forbidden overrides list',
				required: true,
			}),
		),
		section({
			shown: (ctx) =>
				ctx.selected.issueIds.includes(reuploadCustomPackVerificationIssue.id) &&
				projectHasCustomServerModpack(ctx.ProjectV3),
		}).content(
			toggle({
				issue: reuploadCustomPackVerificationIssue,
				label: 'List overrides?',
				id: 'reupload-list',
			}),
			section({
				shown: ({ selected }) => selected.toggleIds.includes('reupload-list'),
			}).content(
				markdown({
					issue: reuploadCustomPackVerificationIssue,
					id: 'overrides',
					label: 'Add list of overrides.',
					required: false,
				}),
			),
		),
	),
)
