import { ListBulletedIcon } from '@modrinth/assets'

import aiGeneratedMessage from '../messages/checklist/messages/rules/ai-generated.md'
import aiImagesMessage from '../messages/checklist/messages/rules/ai-images.md'
import cheatOrHackAdvertisingMessage from '../messages/checklist/messages/rules/cheat-or-hack-advertising.md'
import excessiveLanguagesMessage from '../messages/checklist/messages/rules/excessive-languages.md'
import paidAccessServerMessage from '../messages/checklist/messages/rules/paid-access-server.md'
import prohibitedContentDiscriminatoryMessage from '../messages/checklist/messages/rules/prohibited-content/discriminatory.md'
import prohibitedContentFalseEndorsementMessage from '../messages/checklist/messages/rules/prohibited-content/false-endorsement.md'
import prohibitedContentHarmfulMessage from '../messages/checklist/messages/rules/prohibited-content/harmful.md'
import prohibitedContentIllegalActivityMessage from '../messages/checklist/messages/rules/prohibited-content/illegal-activity.md'
import prohibitedContentImpersonationMessage from '../messages/checklist/messages/rules/prohibited-content/impersonation.md'
import prohibitedContentIpInfringementMessage from '../messages/checklist/messages/rules/prohibited-content/ip-infringement.md'
import prohibitedContentLegalRightsMessage from '../messages/checklist/messages/rules/prohibited-content/legal-rights.md'
import prohibitedContentMisleadingMessage from '../messages/checklist/messages/rules/prohibited-content/misleading.md'
import prohibitedContentMojangBypassMessage from '../messages/checklist/messages/rules/prohibited-content/mojang-bypass.md'
import prohibitedContentObjectionableMessage from '../messages/checklist/messages/rules/prohibited-content/objectionable.md'
import prohibitedContentProfanityMessage from '../messages/checklist/messages/rules/prohibited-content/profanity.md'
import prohibitedContentUndisclosedUploadMessage from '../messages/checklist/messages/rules/prohibited-content/undisclosed-upload.md'
import prohibitedContentHeaderMessage from '../messages/checklist/messages/rules/prohibited-content-header.md'
import ruleBreakingOtherMessage from '../messages/checklist/messages/rules/rule-breaking-other.md'
import serverSideOptInAimBotMessage from '../messages/checklist/messages/rules/server-side-opt-in/aim-bot.md'
import serverSideOptInHidingModsMessage from '../messages/checklist/messages/rules/server-side-opt-in/hiding-mods.md'
import serverSideOptInItemDuplicationMessage from '../messages/checklist/messages/rules/server-side-opt-in/item-duplication.md'
import serverSideOptInMovementMessage from '../messages/checklist/messages/rules/server-side-opt-in/movement.md'
import serverSideOptInPvpMessage from '../messages/checklist/messages/rules/server-side-opt-in/pvp.md'
import serverSideOptInXRayMessage from '../messages/checklist/messages/rules/server-side-opt-in/x-ray.md'
import serverSideOptInHeaderMessage from '../messages/checklist/messages/rules/server-side-opt-in-header.md'
import serverSideOptOutMessage from '../messages/checklist/messages/rules/server-side-opt-out.md'
import { issue, markdown, panel, section, toggle } from './component-builders/builders'

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

export const rulesExcessiveLanguagesIssue = issue({
	id: 'rules-excessive-languages',
	title: 'Excessive languages',
	category: 'Project wide',
	message: excessiveLanguagesMessage,
	suggestedStatus: 'flagged',
})

export const rulesAiGeneratedIssue = issue({
	id: 'rules-ai-generated',
	title: 'AI-generated content',
	category: 'Project wide',
	message: aiGeneratedMessage,
	suggestedStatus: 'flagged',
})

export const rulesAiImagesIssue = issue({
	id: 'rules-ai-images',
	title: 'Prohibited images',
	category: 'Project wide',
	message: aiImagesMessage,
	suggestedStatus: 'flagged',
})

const prohibitedContentMessages: Record<string, string> = {
	'rules-prohibited-content-objectionable': prohibitedContentObjectionableMessage,
	'rules-prohibited-content-discriminatory': prohibitedContentDiscriminatoryMessage,
	'rules-prohibited-content-ip-infringement': prohibitedContentIpInfringementMessage,
	'rules-prohibited-content-legal-rights': prohibitedContentLegalRightsMessage,
	'rules-prohibited-content-illegal-activity': prohibitedContentIllegalActivityMessage,
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
	'rules-server-side-opt-in-aim-bot': serverSideOptInAimBotMessage,
	'rules-server-side-opt-in-movement': serverSideOptInMovementMessage,
	'rules-server-side-opt-in-pvp': serverSideOptInPvpMessage,
	'rules-server-side-opt-in-hiding-mods': serverSideOptInHidingModsMessage,
	'rules-server-side-opt-in-item-duplication': serverSideOptInItemDuplicationMessage,
}

export const rulesServerSideOptInIssue = issue({
	id: 'rules-server-side-opt-in',
	title: 'Server-side opt-in required',
	category: 'Project wide',
	message: ({ selected }) =>
		[
			serverSideOptInHeaderMessage,
			...Object.entries(serverSideOptInMessages)
				.filter(([id]) => selected.toggleIds.includes(id))
				.map(([, message]) => message),
		].join('\n'),
	suggestedStatus: 'flagged',
})

export const rulesRuleBreakingOtherIssue = issue({
	id: 'rules-rule-breaking-other',
	title: 'Other content rule violation',
	category: 'Project wide',
	message: ({ getMarkdownValue }) =>
		ruleBreakingOtherMessage.replaceAll('%MESSAGE%', () => getMarkdownValue('message')),
	suggestedStatus: 'rejected',
})

export const rulesReviewPanel = panel({
	title: 'Rule Following',
	hint: 'Does this project violate the rules?',
	icon: ListBulletedIcon,
}).content(
	section().content(
		toggle({
			issue: rulesPaidAccessServerIssue,
			label: 'Paid access server',
			shown: ({ ProjectV3 }) => !!ProjectV3.minecraft_server,
		}),
		toggle({
			issue: rulesCheatOrHackAdvertisingIssue,
			label: 'Hacks',
		}),
		toggle({
			issue: rulesServerSideOptOutIssue,
			label: 'Opt-out',
		}),
		toggle({
			issue: rulesExcessiveLanguagesIssue,
			label: 'Excessive languages',
			shown: ({ ProjectV3 }) =>
				!!ProjectV3.minecraft_server && (ProjectV3.minecraft_server.languages?.length ?? 0) > 4,
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
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Opt-in',
		}),
		toggle({
			issue: rulesRuleBreakingOtherIssue,
			label: 'Other',
		}),
	),
	section({
		label: 'Which Prohibited Content rules does this project violate?',
		shown: ({ selected }) => selected.issueIds.includes(rulesProhibitedContentIssue.id),
	}).content(
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Objectionable',
			id: 'rules-prohibited-content-objectionable',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Discriminatory or Explicit',
			id: 'rules-prohibited-content-discriminatory',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'IP Infringement',
			id: 'rules-prohibited-content-ip-infringement',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Rights Violation',
			id: 'rules-prohibited-content-legal-rights',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Illegal Activity',
			id: 'rules-prohibited-content-illegal-activity',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Harmful or Deceptive',
			id: 'rules-prohibited-content-harmful',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Misleading claims',
			id: 'rules-prohibited-content-misleading',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Impersonation',
			id: 'rules-prohibited-content-impersonation',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'False Endorsement',
			id: 'rules-prohibited-content-false-endorsement',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Profanity',
			id: 'rules-prohibited-content-profanity',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Undisclosed Data Upload',
			id: 'rules-prohibited-content-undisclosed-upload',
		}),
		toggle({
			issue: rulesProhibitedContentIssue,
			label: 'Mojang Bypass',
			id: 'rules-prohibited-content-mojang-bypass',
		}),
	),
	section({
		label: 'Which features require a Server-side Opt-in?',
		shown: ({ selected }) => selected.issueIds.includes(rulesServerSideOptInIssue.id),
	}).content(
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'X-ray',
			id: 'rules-server-side-opt-in-x-ray',
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Aim Assist',
			id: 'rules-server-side-opt-in-aim-bot',
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Movement',
			id: 'rules-server-side-opt-in-movement',
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'PvP',
			id: 'rules-server-side-opt-in-pvp',
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Anti 3.x',
			id: 'rules-server-side-opt-in-hiding-mods',
		}),
		toggle({
			issue: rulesServerSideOptInIssue,
			label: 'Dupe',
			id: 'rules-server-side-opt-in-item-duplication',
		}),
	),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(rulesRuleBreakingOtherIssue.id),
	}).content(
		markdown({
			issue: rulesRuleBreakingOtherIssue,
			id: 'message',
			label: 'Explain how it infringes on content rules.',
			required: true,
		}),
	),
)
