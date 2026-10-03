import { ScaleIcon } from '@modrinth/assets'

import metadataIssueMessage from '../messages/checklist/messages/post-approval/metadata-issue.md'
import metadataIssueDependenciesMessage from '../messages/checklist/messages/post-approval/metadata-issue/dependencies.md'
import metadataIssueLicenseMessage from '../messages/checklist/messages/post-approval/metadata-issue/license.md'
import metadataIssueLoadersMessage from '../messages/checklist/messages/post-approval/metadata-issue/loaders.md'
import metadataIssueMcVersionsMessage from '../messages/checklist/messages/post-approval/metadata-issue/mc-versions.md'
import { issue, panel, section, text, toggle } from './component-builders/builders'

export const postApprovalMetadataIssueIssue = issue({
	id: 'post-approval-metadata-issue',
	title: 'Post-approval metadata issues',
	category: 'Project wide',
	message: ({ selected, getTextValue }) =>
		[
			metadataIssueMessage,
			selected.toggleIds.includes('post-approval-dependencies')
				? metadataIssueDependenciesMessage
						.replaceAll('%DEPENDENCY_NAME%', () => getTextValue('name'))
						.replaceAll('%DEPENDENCY_LINK%', () => getTextValue('link'))
				: '',
			selected.toggleIds.includes('post-approval-mc-versions')
				? metadataIssueMcVersionsMessage.replaceAll('%SPECIFICS%', () =>
						getTextValue('mc-versions-specifics'),
					)
				: '',
			selected.toggleIds.includes('post-approval-loaders')
				? metadataIssueLoadersMessage.replaceAll('%SPECIFICS%', () =>
						getTextValue('loaders-specifics'),
					)
				: '',
			selected.toggleIds.includes('post-approval-license') ? metadataIssueLicenseMessage : '',
		].join('\n'),
	suggestedStatus: 'approved',
})

export const postApprovalReviewPanel = panel({
	title: 'Post-Approval',
	hint: 'Issue warnings, notices, or takedowns?',
	icon: ScaleIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#3475ee711bf080c5a13cda0b1e4ae9ed',
	shown: ({ projectV3 }) => projectV3.status === 'approved',
}).content(
	toggle({
		issue: postApprovalMetadataIssueIssue,
		label: 'Incorrect metadata',
	}),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(postApprovalMetadataIssueIssue.id),
	}).content(
		toggle({
			issue: postApprovalMetadataIssueIssue,
			label: 'Missing Dependencies',
			id: 'post-approval-dependencies',
		}),
		toggle({
			issue: postApprovalMetadataIssueIssue,
			label: 'Game versions',
			id: 'post-approval-mc-versions',
		}),
		toggle({
			issue: postApprovalMetadataIssueIssue,
			label: 'Loaders',
			id: 'post-approval-loaders',
		}),
		section({
			shown: ({ selected }) => selected.toggleIds.includes('post-approval-dependencies'),
		}).content(
			text({
				issue: postApprovalMetadataIssueIssue,
				id: 'name',
				label: 'Dependency name',
				required: true,
			}),
			text({
				issue: postApprovalMetadataIssueIssue,
				id: 'link',
				label: 'Dependency link',
				required: true,
			}),
		),
		section({
			shown: ({ selected }) => selected.toggleIds.includes('post-approval-mc-versions'),
		}).content(
			text({
				issue: postApprovalMetadataIssueIssue,
				id: 'mc-versions-specifics',
				label: 'More details about the game versions issue?',
				required: false,
			}),
		),
		section({
			shown: ({ selected }) => selected.toggleIds.includes('post-approval-loaders'),
		}).content(
			text({
				issue: postApprovalMetadataIssueIssue,
				id: 'loaders-specifics',
				label: 'More details about the loaders issue?',
				required: false,
			}),
		),
		toggle({
			issue: postApprovalMetadataIssueIssue,
			label: 'Inconsistent Licensing',
			id: 'post-approval-license',
		}),
	),
)
