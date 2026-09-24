import { CopyrightIcon } from '@modrinth/assets'

import customPackProhibitedMessage from '../messages/checklist/messages/reupload/custom-pack-prohibited.md'
import customPackVerificationMessage from '../messages/checklist/messages/reupload/custom-pack-verification.md'
import customPackVerificationListMessage from '../messages/checklist/messages/reupload/custom-pack-verification/list.md'
import identityVerificationMessage from '../messages/checklist/messages/reupload/identity-verification.md'
import identityVerificationServerMessage from '../messages/checklist/messages/reupload/identity-verification-server.md'
import insufficientForkMessage from '../messages/checklist/messages/reupload/insufficient-fork.md'
import missingAttributionMessage from '../messages/checklist/messages/reupload/missing-attribution.md'
import requestProofMessage from '../messages/checklist/messages/reupload/request-proof.md'
import requestProofServerMessage from '../messages/checklist/messages/reupload/request-proof-server.md'
import reuploadMessage from '../messages/checklist/messages/reupload/reupload.md'
import unclearForkMessage from '../messages/checklist/messages/reupload/unclear-fork.md'
import { issue, markdown, panel, section, text, toggle } from './component-builders/builders'
import type { ReviewContext } from './component-builders/types'

const isServerModpack = ({ ProjectV3 }: ReviewContext) =>
	!!ProjectV3.minecraft_server &&
	ProjectV3.minecraft_java_server?.content?.kind === 'modpack' &&
	ProjectV3.minecraft_java_server.content.project_id === ProjectV3.id

export const reuploadReuploadIssue = issue({
	id: 'reupload-reupload',
	title: 'Reuploads are forbidden',
	category: 'Project wide',
	message: ({ getTextValue }) =>
		reuploadMessage
			.replaceAll('%ORIGINAL_PROJECT%', () => getTextValue('original-project'))
			.replaceAll('%ORIGINAL_AUTHOR%', () => getTextValue('original-author')),
	suggestedStatus: 'rejected',
})

export const reuploadUnclearForkIssue = issue({
	id: 'reupload-unclear-fork',
	title: 'Unclear fork',
	category: 'Project wide',
	message: unclearForkMessage,
	suggestedStatus: 'rejected',
})

export const reuploadInsufficientForkIssue = issue({
	id: 'reupload-insufficient-fork',
	title: 'Insufficient fork',
	category: 'Project wide',
	message: insufficientForkMessage,
	suggestedStatus: 'rejected',
})

export const reuploadRequestProofIssue = issue({
	id: 'reupload-request-proof',
	title: 'Proof of permissions',
	category: 'Project wide',
	message: requestProofMessage,
	suggestedStatus: 'rejected',
})

export const reuploadIdentityVerificationIssue = issue({
	id: 'reupload-identity-verification',
	title: 'Identity verification',
	category: 'Project wide',
	message: ({ getTextValue }) =>
		identityVerificationMessage.replaceAll('%PLATFORM%', () => getTextValue('platform')),
	suggestedStatus: 'rejected',
})

export const reuploadIdentityVerificationServerIssue = issue({
	id: 'reupload-identity-verification-server',
	title: 'Server identity verification',
	category: 'Project wide',
	message: ({ getTextValue }) =>
		identityVerificationServerMessage.replaceAll('%CONTACT%', () => getTextValue('contact')),
	suggestedStatus: 'rejected',
})

export const reuploadRequestProofServerIssue = issue({
	id: 'reupload-request-proof-server',
	title: 'Server proof of permissions',
	category: 'Project wide',
	message: requestProofServerMessage,
	suggestedStatus: 'rejected',
})

export const reuploadCustomPackProhibitedIssue = issue({
	id: 'reupload-custom-pack-prohibited',
	title: 'Prohibited custom modpack',
	category: 'Project wide',
	message: ({ getMarkdownValue }) =>
		customPackProhibitedMessage.replaceAll('%OVERRIDES%', () => getMarkdownValue('overrides')),
	suggestedStatus: 'rejected',
})

export const reuploadMissingAttributionIssue = issue({
	id: 'reupload-missing-attribution',
	title: 'Missing attribution',
	category: 'Project wide',
	message: missingAttributionMessage,
	suggestedStatus: 'rejected',
})

export const reuploadCustomPackVerificationIssue = issue({
	id: 'reupload-custom-pack-verification',
	title: 'Custom modpack verification',
	category: 'Project wide',
	message: ({ selected, getMarkdownValue }) =>
		[
			customPackVerificationMessage,
			selected.toggleIds.includes('reupload-list')
				? customPackVerificationListMessage.replaceAll('%OVERRIDES%', () =>
						getMarkdownValue('overrides'),
					)
				: '',
		].join('\n'),
	suggestedStatus: 'rejected',
})

export const reuploadReviewPanel = panel({
	title: 'Reupload',
	hint: 'Does the author have proper permissions to post this project?',
	icon: CopyrightIcon,
}).content(
	toggle({
		issue: reuploadReuploadIssue,
		label: 'Re-upload',
		shown: (ctx) => !ctx.ProjectV3.minecraft_server,
	}),

	toggle({
		issue: reuploadUnclearForkIssue,
		label: 'Unclear Fork',
		shown: (ctx) => !ctx.ProjectV3.minecraft_server,
	}),
	toggle({
		issue: reuploadInsufficientForkIssue,
		label: 'Insufficient Fork',
		shown: (ctx) => !ctx.ProjectV3.minecraft_server,
	}),
	toggle({
		issue: reuploadRequestProofIssue,
		label: 'Proof of permissions',
	}),
	toggle({
		issue: reuploadIdentityVerificationIssue,
		label: 'Verify Identity',
		shown: (ctx) => !ctx.ProjectV3.minecraft_server,
	}),

	toggle({
		issue: reuploadIdentityVerificationServerIssue,
		label: 'Verify Identity',
		shown: (ctx) => !!ctx.ProjectV3.minecraft_server,
	}),
	toggle({
		issue: reuploadRequestProofServerIssue,
		label: 'Reuploaded pack',
		shown: (ctx) => isServerModpack(ctx),
	}),
	toggle({
		issue: reuploadCustomPackProhibitedIssue,
		label: 'Forbidden Overrides',
		shown: (ctx) => isServerModpack(ctx),
	}),
	toggle({
		issue: reuploadMissingAttributionIssue,
		label: 'Missing Attribution',
	}),
	toggle({
		issue: reuploadCustomPackVerificationIssue,
		label: 'Override verification',
		shown: isServerModpack,
	}),
	section({
		shown: (ctx) =>
			ctx.selected.issueIds.includes(reuploadReuploadIssue.id) && !ctx.ProjectV3.minecraft_server,
	}).content(
		text({
			issue: reuploadReuploadIssue,
			id: 'original-project',
			label: 'Original Project Title',
			required: true,
		}),
		text({
			issue: reuploadReuploadIssue,
			id: 'original-author',
			label: 'Original project Author',
			required: true,
		}),
	),
	section({
		shown: (ctx) =>
			ctx.selected.issueIds.includes(reuploadIdentityVerificationIssue.id) &&
			!ctx.ProjectV3.minecraft_server,
	}).content(
		text({
			issue: reuploadIdentityVerificationIssue,
			id: 'platform',
			label: 'Where else can the project be found?',
			required: true,
		}),
	),
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
		shown: (ctx) =>
			ctx.selected.issueIds.includes(reuploadCustomPackProhibitedIssue.id) && isServerModpack(ctx),
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
			isServerModpack(ctx),
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
)
