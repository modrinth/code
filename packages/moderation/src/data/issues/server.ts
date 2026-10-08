import { ServerIcon } from '@modrinth/assets'

import { projectHasCustomServerModpack } from '../../utils'
import excessiveLanguagesMessage from '../messages/checklist/messages/rules/excessive-languages.md'
import { issue, markdown, panel, section, text, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'
import { metadataGameVersionsIssue } from './metadata'
import {
	reuploadCustomPackProhibitedIssue,
	reuploadCustomPackVerificationIssue,
	reuploadIdentityVerificationServerIssue,
	reuploadRequestProofServerIssue,
} from './reupload'

export const serversExcessiveLanguagesIssue = issue({
	id: 'servers-excessive-languages',
	title: 'Excessive languages',
	category: 'Server details',
	actions: [issueTargets.modifyServerLanguages()],
	message: excessiveLanguagesMessage,
	suggestedStatus: 'flagged',
})

export const serverReviewPanel = panel({
	title: 'Server details',
	hint: "Are there any issues with this project's server details?",
	icon: ServerIcon,
	guidanceUrl: '',
	shown: ({ projectV3 }) => !!projectV3.minecraft_server,
}).content(
	toggle({
		issue: serversExcessiveLanguagesIssue,
		label: 'Excessive languages',
		id: 'servers-excessive-languages',
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
			!!ctx.projectV3.minecraft_server,
	}).content(
		text({
			issue: reuploadIdentityVerificationServerIssue,
			id: 'contact',
			label: 'Known public contact method',
			required: false,
		}),
	),
	section({
		label: 'Custom Modpack',
		shown: (ctx) => projectHasCustomServerModpack(ctx.projectV3),
	}).content(
		toggle({
			issue: reuploadRequestProofServerIssue,
			label: 'Reuploaded pack',
			shown: (ctx) => projectHasCustomServerModpack(ctx.projectV3),
		}),
		toggle({
			issue: reuploadCustomPackProhibitedIssue,
			label: 'Forbidden Overrides',
			shown: (ctx) => projectHasCustomServerModpack(ctx.projectV3),
		}),
		toggle({
			issue: reuploadCustomPackVerificationIssue,
			label: 'Override verification',
			shown: (ctx) => projectHasCustomServerModpack(ctx.projectV3),
		}),
		section({
			shown: (ctx) =>
				ctx.selected.issueIds.includes(reuploadCustomPackProhibitedIssue.id) &&
				projectHasCustomServerModpack(ctx.projectV3),
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
				projectHasCustomServerModpack(ctx.projectV3),
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
