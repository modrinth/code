import { ListBulletedIcon } from '@modrinth/assets'
import { md } from '@modrinth/utils'

import aiGeneratedMessage from '../messages/checklist/messages/rules/ai-generated.md'
import aiImagesMessage from '../messages/checklist/messages/rules/ai-images.md'
import aiImagesTargetedMessage from '../messages/checklist/messages/rules/ai-images-targeted.md'
import cheatOrHackAdvertisingMessage from '../messages/checklist/messages/rules/cheat-or-hack-advertising.md'
import paidAccessServerMessage from '../messages/checklist/messages/rules/paid-access-server.md'
import prohibitedContentDiscriminatoryMessage from '../messages/checklist/messages/rules/prohibited-content/discriminatory.md'
import prohibitedContentFalseEndorsementMessage from '../messages/checklist/messages/rules/prohibited-content/false-endorsement.md'
import prohibitedContentHarmfulMessage from '../messages/checklist/messages/rules/prohibited-content/harmful.md'
import prohibitedContentIllegalActivityMessage from '../messages/checklist/messages/rules/prohibited-content/illegal-activity.md'
import prohibitedContentIllegalActivityPiracyMessage from '../messages/checklist/messages/rules/prohibited-content/illegal-activity/piracy.md'
import prohibitedContentImpersonationMessage from '../messages/checklist/messages/rules/prohibited-content/impersonation.md'
import prohibitedContentIpInfringementMessage from '../messages/checklist/messages/rules/prohibited-content/ip-infringement.md'
import prohibitedContentLegalRightsMessage from '../messages/checklist/messages/rules/prohibited-content/legal-rights.md'
import prohibitedContentMisleadingMessage from '../messages/checklist/messages/rules/prohibited-content/misleading.md'
import prohibitedContentMojangBypassMessage from '../messages/checklist/messages/rules/prohibited-content/mojang-bypass.md'
import prohibitedContentObjectionableMessage from '../messages/checklist/messages/rules/prohibited-content/objectionable.md'
import prohibitedContentProfanityMessage from '../messages/checklist/messages/rules/prohibited-content/profanity.md'
import prohibitedContentUndisclosedUploadMessage from '../messages/checklist/messages/rules/prohibited-content/undisclosed-upload.md'
import prohibitedContentHeaderMessage from '../messages/checklist/messages/rules/prohibited-content-header.md'
import serverSideOptInAimBotMessage from '../messages/checklist/messages/rules/server-side-opt-in/aim-bot.md'
import serverSideOptInHidingModsMessage from '../messages/checklist/messages/rules/server-side-opt-in/hiding-mods.md'
import serverSideOptInItemDuplicationMessage from '../messages/checklist/messages/rules/server-side-opt-in/item-duplication.md'
import serverSideOptInMovementMessage from '../messages/checklist/messages/rules/server-side-opt-in/movement.md'
import serverSideOptInPvpMessage from '../messages/checklist/messages/rules/server-side-opt-in/pvp.md'
import serverSideOptInXRayMessage from '../messages/checklist/messages/rules/server-side-opt-in/x-ray.md'
import serverSideOptInHeaderMessage from '../messages/checklist/messages/rules/server-side-opt-in-header.md'
import serverSideOptInHeaderListlessMessage from '../messages/checklist/messages/rules/server-side-opt-in-header-listless.md'
import serverSideOptOutMessage from '../messages/checklist/messages/rules/server-side-opt-out.md'
import { issue, panel, section, select, toggle } from './component-builders/builders'
import { selectedGalleryImages } from './component-builders/gallery-images'
import { IssuePriority } from './component-builders/priority'
import type { IssueAction } from './component-builders/types'

const escapeHtml = md().utils.escapeHtml

const optInTypes = ['X-ray', 'Aimbot', 'Movement', 'PvP', 'Hiding', 'Dupes']

export const rulesPaidAccessServerIssue = issue({
	id: 'rules-paid-access-server',
	title: 'Paid access server',
	category: 'Project wide',
	message: paidAccessServerMessage,
	suggestedStatus: 'rejected',
})

export const rulesCheatOrHackAdvertisingIssue = issue({
	id: 'rules-cheat-or-hack-advertising',
	title: 'Cheat or hack advertising',
	category: 'Project wide',
	message: cheatOrHackAdvertisingMessage,
	suggestedStatus: 'rejected',
})

export const rulesServerSideOptOutIssue = issue({
	id: 'rules-server-side-opt-out',
	title: 'Server-side opt-out required',
	category: 'Project wide',
	message: serverSideOptOutMessage,
	suggestedStatus: 'flagged',
})

export const rulesAiGeneratedIssue = issue({
	id: 'rules-ai-generated',
	priority: IssuePriority.Alerts,
	title: 'AI-generated content',
	category: 'Project wide',
	message: aiGeneratedMessage,
	suggestedStatus: 'flagged',
})

export const rulesAiImagesIssue = issue({
	id: 'rules-ai-images',
	title: 'Prohibited images',
	category: 'Project wide',
	actions: ({ projectV3, selected, sendIssuesAsThreadMessage }) => {
		if (sendIssuesAsThreadMessage) return []
		const keys = new Set((selected.items['gallery-image'] ?? []).map(({ key }) => key))
		const imageIds = projectV3.gallery.flatMap((image) =>
			image.id !== undefined && keys.has(String(image.id)) ? [image.id] : [],
		)
		const actions: IssueAction[] = []
		if (selected.toggleIds.includes('description-ai-images')) {
			actions.push(() => ({
				type: 'modify_description',
				value: { original: projectV3.description },
			}))
		}
		if (selected.toggleIds.includes('icon-ai-images')) {
			actions.push(() => ({
				type: 'remove_icon',
			}))
		}
		if (imageIds.length) {
			actions.push(() => ({
				type: 'remove_gallery_images',
				value: { image_ids: imageIds },
			}))
		}
		return actions
	},
	message: (context) => {
		const { projectV3, selected } = context
		const entries: string[] = []
		const settingsUrl = `https://modrinth.com/project/${encodeURIComponent(projectV3.id)}/settings`
		if (selected.toggleIds.includes('icon-ai-images')) {
			const image = projectV3.icon_url
				? `<image-viewer src="${escapeHtml(projectV3.icon_url)}" alt="" />`
				: ''
			entries.push(`[Project icon](${settingsUrl})\n${image}`)
		}
		const images = selectedGalleryImages(context)
		if (images.length) {
			const previews = images.map(
				(image) =>
					`<image-viewer src="${escapeHtml(image.raw_url || image.url)}" alt="${escapeHtml(image.name ?? '')}" />`,
			)
			entries.push(`[Gallery](${settingsUrl}/gallery)\n${previews.join('\n')}`)
		}
		if (selected.toggleIds.includes('description-ai-images')) {
			entries.push(
				`[Description](${settingsUrl}/description)\n\nRemove any prohibited images in the description.`,
			)
		}
		if (!entries.length) return aiImagesMessage.trimEnd()
		const locations = `:\n\n${entries.join('\n\n')}`
		return aiImagesTargetedMessage.replace('%AI_IMAGE_LOCATIONS%', locations).trimEnd()
	},
	suggestedStatus: 'flagged',
})

const prohibitedContentMessages: Record<string, string> = {
	'rules-prohibited-content-objectionable': prohibitedContentObjectionableMessage,
	'rules-prohibited-content-discriminatory': prohibitedContentDiscriminatoryMessage,
	'rules-prohibited-content-ip-infringement': prohibitedContentIpInfringementMessage,
	'rules-prohibited-content-legal-rights': prohibitedContentLegalRightsMessage,
	'rules-prohibited-content-illegal-activity': prohibitedContentIllegalActivityMessage,
	'rules-prohibited-content-illegal-activity-piracy': prohibitedContentIllegalActivityPiracyMessage,
	'rules-prohibited-content-harmful': prohibitedContentHarmfulMessage,
	'rules-prohibited-content-misleading': prohibitedContentMisleadingMessage,
	'rules-prohibited-content-impersonation': prohibitedContentImpersonationMessage,
	'rules-prohibited-content-false-endorsement': prohibitedContentFalseEndorsementMessage,
	'rules-prohibited-content-profanity': prohibitedContentProfanityMessage,
	'rules-prohibited-content-undisclosed-upload': prohibitedContentUndisclosedUploadMessage,
	'rules-prohibited-content-mojang-bypass': prohibitedContentMojangBypassMessage,
}

export const rulesProhibitedContentIssue = issue({
	id: 'rules-prohibited-content',
	title: 'Prohibited content',
	category: 'Project wide',
	message: ({ selected }) =>
		[
			prohibitedContentHeaderMessage,
			...Object.entries(prohibitedContentMessages)
				.filter(([id]) => selected.toggleIds.includes(id))
				.map(([, message]) => message),
		].join('\n'),
	suggestedStatus: 'rejected',
})

const serverSideOptInMessages: Record<string, string> = {
	'rules-server-side-opt-in-x-ray': serverSideOptInXRayMessage,
	'rules-server-side-opt-in-aimbot': serverSideOptInAimBotMessage,
	'rules-server-side-opt-in-movement': serverSideOptInMovementMessage,
	'rules-server-side-opt-in-pvp': serverSideOptInPvpMessage,
	'rules-server-side-opt-in-hiding': serverSideOptInHidingModsMessage,
	'rules-server-side-opt-in-dupes': serverSideOptInItemDuplicationMessage,
}

export const rulesServerSideOptInIssue = issue({
	id: 'rules-server-side-opt-in',
	title: 'Server-side opt-in required',
	category: 'Project wide',
	message: (ctx) =>
		ctx.getSelectValues('rules-server-side-opt-in-features').length > 0
			? [
					serverSideOptInHeaderMessage,
					...Object.entries(serverSideOptInMessages)
						.filter(([id]) => ctx.getSelectValues('rules-server-side-opt-in-features').includes(id))
						.map(([, message]) => message),
				].join('\n')
			: serverSideOptInHeaderListlessMessage,
	suggestedStatus: 'rejected',
})

export const rulesReviewPanel = panel({
	title: 'Rule Following',
	hint: 'Does this project violate the rules?',
	icon: ListBulletedIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
}).content(
	section().content(
		toggle({
			issue: rulesPaidAccessServerIssue,
			label: 'Paid access server',
			shown: ({ projectV3 }) => !!projectV3.minecraft_server,
		}),
		toggle({
			issue: rulesCheatOrHackAdvertisingIssue,
			label: 'Hacks',
		}),
		toggle({
			issue: rulesServerSideOptOutIssue,
			label: 'Opt-out',
			id: 'rules-server-side-opt-out',
			unselectToggles: ['rules-server-side-opt-in'],
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Opt-in',
			id: 'rules-server-side-opt-in',
			unselectToggles: ['rules-server-side-opt-out'],
		}),
		toggle({
			issue: rulesAiGeneratedIssue,
			label: 'AI Generated',
		}),
		toggle({
			issue: rulesAiImagesIssue,
			label: 'AI Images',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Prohibited Content',
		}),
	),
	section({
		label: 'Which rules does this project violate?',
		shown: ({ selected }) => selected.issueIds.includes(rulesProhibitedContentIssue.id),
	}).content(
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Objectionable',
			id: 'rules-prohibited-content-objectionable',
			issueCardDisplay: {
				label: 'Objectionable',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Discriminatory or Explicit',
			id: 'rules-prohibited-content-discriminatory',
			issueCardDisplay: {
				label: 'Discriminatory or Explicit',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'IP Infringement',
			id: 'rules-prohibited-content-ip-infringement',
			issueCardDisplay: {
				label: 'IP Infringement',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Rights Violation',
			id: 'rules-prohibited-content-legal-rights',
			issueCardDisplay: {
				label: 'Rights Violation',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Illegal Activity',
			id: 'rules-prohibited-content-illegal-activity',
			issueCardDisplay: {
				label: 'Illegal Activity',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Harmful or Deceptive',
			id: 'rules-prohibited-content-harmful',
			issueCardDisplay: {
				label: 'Harmful or Deceptive',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Misleading claims',
			id: 'rules-prohibited-content-misleading',
			issueCardDisplay: {
				label: 'Misleading claims',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Impersonation',
			id: 'rules-prohibited-content-impersonation',
			issueCardDisplay: {
				label: 'Impersonation',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'False Endorsement',
			id: 'rules-prohibited-content-false-endorsement',
			issueCardDisplay: {
				label: 'False Endorsement',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Profanity',
			id: 'rules-prohibited-content-profanity',
			issueCardDisplay: {
				label: 'Profanity',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Undisclosed Data Upload',
			id: 'rules-prohibited-content-undisclosed-upload',
			issueCardDisplay: {
				label: 'Undisclosed Data Upload',
			},
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Mojang Bypass',
			id: 'rules-prohibited-content-mojang-bypass',
			issueCardDisplay: {
				label: 'Mojang Bypass',
			},
		}),
	),
	section({
		shown: ({ selected }) => selected.issueIds.includes(rulesServerSideOptInIssue.id),
	}).content(
		select({
			issue: rulesServerSideOptInIssue,
			label: 'Which features require a Server-side Opt-in?',
			id: 'rules-server-side-opt-in-features',
			multiple: true,
			required: false,
			options: () =>
				optInTypes.map((type) => ({
					value: `rules-server-side-opt-in-${type.toLowerCase()}`,
					label: type,
				})),
		}),
	),
	section({
		label: 'Type of potentially illegal activity?',
		shown: ({ selected }) =>
			selected.toggleIds.includes('rules-prohibited-content-illegal-activity'),
	}).content(
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Piracy',
			id: 'rules-prohibited-content-illegal-activity-piracy',
			issueCardDisplay: {
				label: 'Piracy',
			},
		}),
	),
)
