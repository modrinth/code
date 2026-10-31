import type { Labrinth } from '@modrinth/api-client'
import { TriangleAlertIcon } from '@modrinth/assets'

import {
	disclosureExists,
	disclosureHasText,
	hasMissingAiUsage as hasMissingAiContent,
} from '../../utils'
import listItemAdsMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ads.md'
import listItemAiMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/ai.md'
import listItemAiFunctionalityMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/ai-functionality.md'
import listItemAiAssetsMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/ai-usages/assets.md'
import listItemAiCodeMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/ai-usages/code.md'
import listItemAiTextMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/ai-usages/text.md'
import listItemAiListIntroMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/ai/list-intro.md'
import listItemArchiveMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/archive.md'
import listItemDerivativeContentMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/derivative-content.md'
import listItemPaidFeaturesMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/paid-features.md'
import listItemPhotosensitivityMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/photosensitivity.md'
import listItemSystemInteractionsMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/system-interactions.md'
import listItemTelemetryMessage from '../messages/checklist/messages/disclosures/disclosure-list-items/telemetry.md'
import formattingListIntroMessage from '../messages/checklist/messages/disclosures/formatting-disclosures/list-intro.md'
import disclosuresHeaderMessage from '../messages/checklist/messages/disclosures/header.md'
import disclosuresHeaderListlessMessage from '../messages/checklist/messages/disclosures/header-listless.md'
import missingAdsMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ads.md'
import missingAiMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/ai.md'
import missingAiFunctionalityMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/ai-functionality.md'
import missingAiAssetsMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/ai-usages/assets.md'
import missingAiCodeMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/ai-usages/code.md'
import missingAiTextMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/ai-usages/text.md'
import missingAiListIntroMessage from '../messages/checklist/messages/disclosures/missing-disclosures/ai/list-intro.md'
import missingArchiveMessage from '../messages/checklist/messages/disclosures/missing-disclosures/archive.md'
import missingDerivativeContentMessage from '../messages/checklist/messages/disclosures/missing-disclosures/derivative-content.md'
import missingListIntroMessage from '../messages/checklist/messages/disclosures/missing-disclosures/list-intro.md'
import missingPaidFeaturesMessage from '../messages/checklist/messages/disclosures/missing-disclosures/paid-features.md'
import missingPhotosensitivityMessage from '../messages/checklist/messages/disclosures/missing-disclosures/photosensitivity.md'
import missingSystemInteractionsMessage from '../messages/checklist/messages/disclosures/missing-disclosures/system-interactions.md'
import missingTelemetryMessage from '../messages/checklist/messages/disclosures/missing-disclosures/telemetry.md'
import missingTelemetryAlwaysMessage from '../messages/checklist/messages/disclosures/missing-disclosures/telemetry/consent-model/always.md'
import missingTelemetryOptInMessage from '../messages/checklist/messages/disclosures/missing-disclosures/telemetry/consent-model/opt-in.md'
import missingTelemetryOptOutMessage from '../messages/checklist/messages/disclosures/missing-disclosures/telemetry/consent-model/opt-out.md'
import misusedListIntroMessage from '../messages/checklist/messages/disclosures/misused-disclosures/list-intro.md'
import nonEnglishListIntroMessage from '../messages/checklist/messages/disclosures/non-english-disclosures/list-intro.md'
import { issue, panel, section, toggle } from './component-builders/builders'
import { issueLocation } from './component-builders/locations'
import { issueTargets } from './component-builders/targets'
import type { IssueAction } from './component-builders/types'

const DISCLOSURES_MAX_WIDTH = '400px'

type DisclosureType = (typeof disclosureTypes)[keyof typeof disclosureTypes]
type DisclosureTypeOption = (typeof disclosureOptions)[keyof typeof disclosureOptions]
type DisclosureIssueType = (typeof disclosureIssueTypes)[keyof typeof disclosureIssueTypes]

const disclosureIssueTypes = {
	missing: { type: 'missing', label: 'Missing' },
	misused: { type: 'misused', label: 'Misused' },
	nonEnglish: { type: 'non_english', label: 'Non-English' },
	formatting: { type: 'formatting', label: 'Formatting' },
}

const disclosureOptions = {
	aiAssets: { type: 'assets', label: 'AI Assets' },
	aiCode: { type: 'code', label: 'AI Code' },
	aiText: { type: 'text', label: 'AI Text' },
	consentIn: { type: 'opt_in', label: 'Telemetry: Opt-in' },
	consentOut: { type: 'opt_out', label: 'Telemetry: Opt-out' },
	consentAlways: { type: 'always', label: 'Telemetry: Always' },
}

const disclosureTypes = {
	aic: {
		type: 'ai_content',
		label: 'AI Content',
		options: [disclosureOptions.aiAssets, disclosureOptions.aiCode, disclosureOptions.aiText],
	},
	aif: {
		type: 'ai_functionality',
		label: 'AI Functionality',
	},
	ads: {
		type: 'advertisements',
		label: 'Advertisements',
	},
	paid: {
		type: 'paid_features',
		label: 'Paid Features',
	},
	telemetry: {
		type: 'telemetry',
		label: 'Telemetry',
		options: [
			disclosureOptions.consentIn,
			disclosureOptions.consentOut,
			disclosureOptions.consentAlways,
		],
	},
	derivative: {
		type: 'derivative_work',
		label: 'Derivative Content',
	},
	photosensitivity: {
		type: 'epilepsy_triggers',
		label: 'Photosensitivity',
	},
	system: {
		type: 'system_interactions',
		label: 'System Interactions',
	},
	archive: {
		type: 'archived',
		label: 'Archive',
	},
} satisfies Record<
	string,
	{
		type: Labrinth.Projects.v3.ProjectDisclosureType
		label: string
		options?: DisclosureTypeOption[]
	}
>

function getDisclosureToggleID(disclosureType: string, issueType: string, option?: string) {
	return disclosureType === 'any'
		? `disclosures-${issueType}`
		: option
			? `${disclosureType}-disclosure-${issueType}-${option}`
			: `${disclosureType}-disclosure-${issueType}`
}

export const disclosuresIssue = issue({
	id: 'disclosures',
	locations: [issueLocation('disclosures')],
	title: 'Invalid disclosures',
	category: 'Disclosures',
	actions: ({ selected }) => {
		const actions: IssueAction[] = []

		const toggleIds = new Set(selected.toggleIds)

		for (const disclosureType of Object.values(disclosureTypes)) {
			for (const issueType of Object.values(disclosureIssueTypes)) {
				const toggleId = getDisclosureToggleID(disclosureType.type, issueType.type)

				if (!toggleIds.has(toggleId)) continue

				switch (issueType.type) {
					case disclosureIssueTypes.missing.type:
						actions.push(
							disclosureType === disclosureTypes.aic
								? issueTargets.modifyDisclosure(disclosureType.type) // missing a part of existing disclosure should modify, suh as for ai usage disclosure
								: issueTargets.addDisclosures([disclosureType.type]),
						)
						break
					case disclosureIssueTypes.misused.type:
						actions.push(issueTargets.modifyDisclosure(disclosureType.type))
						break
					default:
						actions.push(issueTargets.markAddressed())
						break
				}
			}
		}

		return actions
	},

	message: ({ selected }) => {
		const toggleIds = new Set(selected.toggleIds)
		const missingParts: string[] = []
		const misusedParts: string[] = []
		const nonEnglishParts: string[] = []
		const formattingParts: string[] = []

		for (const disclosureType of Object.values(disclosureTypes) as DisclosureType[]) {
			for (const issueType of Object.values(disclosureIssueTypes) as DisclosureIssueType[]) {
				const toggleId = getDisclosureToggleID(disclosureType.type, issueType.type)

				if (!toggleIds.has(toggleId)) continue

				let message: string

				if (issueType.type === disclosureIssueTypes.missing.type) {
					switch (disclosureType) {
						case disclosureTypes.aic:
							message = missingAiMessage
							break
						case disclosureTypes.aif:
							message = missingAiFunctionalityMessage
							break
						case disclosureTypes.ads:
							message = missingAdsMessage
							break
						case disclosureTypes.paid:
							message = missingPaidFeaturesMessage
							break
						case disclosureTypes.telemetry:
							message = missingTelemetryMessage
							break
						case disclosureTypes.derivative:
							message = missingDerivativeContentMessage
							break
						case disclosureTypes.photosensitivity:
							message = missingPhotosensitivityMessage
							break
						case disclosureTypes.system:
							message = missingSystemInteractionsMessage
							break
						case disclosureTypes.archive:
							message = missingArchiveMessage
							break
						default:
							continue
					}

					missingParts.push(message)

					if (disclosureType === disclosureTypes.aic) {
						const usages: string[] = []
						if (toggleIds.has('ai_content-disclosure-missing-code'))
							usages.push(missingAiCodeMessage)
						if (toggleIds.has('ai_content-disclosure-missing-assets'))
							usages.push(missingAiAssetsMessage)
						if (toggleIds.has('ai_content-disclosure-missing-text'))
							usages.push(missingAiTextMessage)
						if (usages.length) missingParts.push(missingAiListIntroMessage, ...usages)
					}

					if (disclosureType === disclosureTypes.telemetry) {
						if (toggleIds.has('telemetry-disclosure-missing-opt-in')) {
							missingParts.push(missingTelemetryOptInMessage)
						}
						if (toggleIds.has('telemetry-disclosure-missing-opt-out')) {
							missingParts.push(missingTelemetryOptOutMessage)
						}
						if (toggleIds.has('telemetry-disclosure-missing-always')) {
							missingParts.push(missingTelemetryAlwaysMessage)
						}
					}
				} else {
					switch (disclosureType) {
						case disclosureTypes.aic:
							message = listItemAiMessage
							break
						case disclosureTypes.aif:
							message = listItemAiFunctionalityMessage
							break
						case disclosureTypes.ads:
							message = listItemAdsMessage
							break
						case disclosureTypes.paid:
							message = listItemPaidFeaturesMessage
							break
						case disclosureTypes.telemetry:
							message = listItemTelemetryMessage
							break
						case disclosureTypes.derivative:
							message = listItemDerivativeContentMessage
							break
						case disclosureTypes.photosensitivity:
							message = listItemPhotosensitivityMessage
							break
						case disclosureTypes.system:
							message = listItemSystemInteractionsMessage
							break
						case disclosureTypes.archive:
							message = listItemArchiveMessage
							break
						default:
							continue
					}

					switch (issueType) {
						case disclosureIssueTypes.misused:
							misusedParts.push(message)

							if (disclosureType === disclosureTypes.aic) {
								const usages: string[] = []
								if (toggleIds.has('ai-disclosure-misused-code')) usages.push(listItemAiCodeMessage)
								if (toggleIds.has('ai-disclosure-misused-assets'))
									usages.push(listItemAiAssetsMessage)
								if (toggleIds.has('ai-disclosure-misused-text')) usages.push(listItemAiTextMessage)
								if (usages.length) misusedParts.push(listItemAiListIntroMessage, ...usages)
							}
							break

						case disclosureIssueTypes.nonEnglish:
							nonEnglishParts.push(message)
							break

						case disclosureIssueTypes.formatting:
							formattingParts.push(message)
							break
					}
				}
			}
		}

		const groups: string[] = []

		if (selected.toggleIds.includes('disclosures-issues')) {
			return disclosuresHeaderListlessMessage
		}

		if (missingParts.length) {
			const body = missingParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${missingListIntroMessage.trim()}\n\n${body}`)
		}

		if (misusedParts.length) {
			const body = misusedParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${misusedListIntroMessage.trim()}\n\n${body}`)
		}

		if (nonEnglishParts.length) {
			const body = nonEnglishParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${nonEnglishListIntroMessage.trim()}\n\n${body}`)
		}

		if (formattingParts.length) {
			const body = formattingParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${formattingListIntroMessage.trim()}\n\n${body}`)
		}

		return groups.length ? [disclosuresHeaderMessage.trim(), ...groups].join('\n\n') : ''
	},

	suggestedStatus: ({ selected }) => {
		// TODO: account for new types etc etc
		const toggleIds = new Set(selected.toggleIds)
		const rejected = ['telemetry', 'derivative-content', 'photosensitivity', 'system-interactions']
		if (
			rejected.some(
				(field) =>
					toggleIds.has(`${field}-disclosure-missing`) ||
					toggleIds.has(`${field}-disclosure-misused`),
			)
		)
			return 'rejected'
		if (
			[...toggleIds].some((id) => id.includes('-disclosure-misused')) ||
			['ai', 'ai-functionality', 'ads', 'paid-features'].some((field) =>
				toggleIds.has(`${field}-disclosure-missing`),
			)
		)
			return 'flagged'
		return undefined
	},
})

export const disclosuresReviewPanel = panel({
	title: 'Disclosures',
	hint: 'Does this project provide accurate disclosure information?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-issues',
		label: 'Incorrect',
		disabled: ({ selected }) => selected.toggleIds.some((id) => id.includes('-disclosure-')),
	}),
)

function disclosureIssueToggle(
	disclosureType: DisclosureType,
	issueType: DisclosureIssueType,
	option?: DisclosureTypeOption,
) {
	const toggleLabel = option ? `${issueType.label}: ${option.label}` : issueType.label
	const groupLabel = option ? `${disclosureType.label}: ${option.label}` : disclosureType.label

	return toggle({
		issue: disclosuresIssue,
		id: getDisclosureToggleID(disclosureType.type, issueType.type, option?.type),
		label: toggleLabel,
		issueCardDisplay: {
			label: groupLabel,
			group: issueType.label,
		},
		unselectToggles: ['disclosures-issues'],
		shown: ({ disclosures }) => {
			if (!option) {
				if (issueType === disclosureIssueTypes.missing) {
					if (disclosureType === disclosureTypes.aic) {
						return hasMissingAiContent(disclosures)
					}
					return !disclosureExists(disclosures, disclosureType.type)
				} else if (issueType === disclosureIssueTypes.misused) {
					return disclosureExists(disclosures, disclosureType.type)
				} else if (
					issueType === disclosureIssueTypes.nonEnglish ||
					issueType === disclosureIssueTypes.formatting
				) {
					const disclosure = disclosures.find(
						(disclosure) => disclosure.type === disclosureType.type,
					)
					return disclosure ? disclosureHasText(disclosure) : false
				}
			} else if (disclosureType === disclosureTypes.aic) {
				if (
					issueType === disclosureIssueTypes.missing &&
					(option.type === 'code' || option.type === 'assets' || option.type === 'text')
				) {
					return hasMissingAiContent(disclosures, [option.type])
				} else if (issueType === disclosureIssueTypes.misused) {
					return true
				}
			} else if (disclosureType === disclosureTypes.telemetry) {
				return true
			}

			return true
		},
	})
}

export const aiDisclosureReviewPanel = panel({
	title: 'AI Usage',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.aic, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.aic, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.aic, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.aic, disclosureIssueTypes.formatting),
	section({
		label: 'Type of missing disclosure?',
		shown: ({ selected }) =>
			selected.toggleIds.includes(
				getDisclosureToggleID(disclosureTypes.aic.type, disclosureIssueTypes.missing.type),
			),
	}).content(
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.missing,
			disclosureOptions.aiCode,
		),
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.missing,
			disclosureOptions.aiAssets,
		),
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.missing,
			disclosureOptions.aiText,
		),
	),
	section({
		label: 'Type of misused disclosure?',
		shown: ({ selected }) =>
			selected.toggleIds.includes(
				getDisclosureToggleID(disclosureTypes.aic.type, disclosureIssueTypes.misused.type),
			),
	}).content(
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.misused,
			disclosureOptions.aiCode,
		),
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.misused,
			disclosureOptions.aiAssets,
		),
		disclosureIssueToggle(
			disclosureTypes.aic,
			disclosureIssueTypes.misused,
			disclosureOptions.aiText,
		),
	),
)

export const aiFunctionalityDisclosureReviewPanel = panel({
	title: 'AI Functionality',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.aif, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.aif, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.aif, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.aif, disclosureIssueTypes.formatting),
)

export const adsDisclosureReviewPanel = panel({
	title: 'Advertisements',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.ads, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.ads, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.ads, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.ads, disclosureIssueTypes.formatting),
)

export const paidFeaturesDisclosureReviewPanel = panel({
	title: 'Paid Features',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.paid, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.paid, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.paid, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.paid, disclosureIssueTypes.formatting),
)

export const telemetryDisclosureReviewPanel = panel({
	title: 'Telemetry',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.telemetry, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.telemetry, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.telemetry, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.telemetry, disclosureIssueTypes.formatting),
	section({
		label: "What is the telemetry's consent model?",
		shown: ({ selected }) => selected.toggleIds.includes('telemetry-disclosure-missing'),
	}).content(
		toggle({
			issue: disclosuresIssue,
			id: 'telemetry-disclosure-missing-opt-in',
			label: 'Opt In',
			issueCardDisplay: {
				label: 'Telemetry: Opt In',
				group: 'Missing',
			},
			unselectToggles: [
				'telemetry-disclosure-missing-opt-out',
				'telemetry-disclosure-missing-always',
			],
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'telemetry-disclosure-missing-opt-out',
			label: 'Opt Out',
			issueCardDisplay: {
				label: 'Telemetry: Opt Out',
				group: 'Missing',
			},
			unselectToggles: [
				'telemetry-disclosure-missing-opt-in',
				'telemetry-disclosure-missing-always',
			],
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'telemetry-disclosure-missing-always',
			label: 'Always Online',
			issueCardDisplay: {
				label: 'Telemetry: Always Online',
				group: 'Missing',
			},
			unselectToggles: [
				'telemetry-disclosure-missing-opt-in',
				'telemetry-disclosure-missing-opt-out',
			],
		}),
	),
)

export const derivativeContentDisclosureReviewPanel = panel({
	title: 'Derivative Content',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.derivative, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.derivative, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.derivative, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.derivative, disclosureIssueTypes.formatting),
)

export const photosensitivityDisclosureReviewPanel = panel({
	title: 'Photosensitivity',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.photosensitivity, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.photosensitivity, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.photosensitivity, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.photosensitivity, disclosureIssueTypes.formatting),
)

export const systemInteractionsDisclosureReviewPanel = panel({
	title: 'System Interactions',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.system, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.system, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.system, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.system, disclosureIssueTypes.formatting),
)

export const archiveDisclosureReviewPanel = panel({
	title: 'Archive',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	maxWidth: DISCLOSURES_MAX_WIDTH,
}).content(
	disclosureIssueToggle(disclosureTypes.archive, disclosureIssueTypes.missing),
	disclosureIssueToggle(disclosureTypes.archive, disclosureIssueTypes.misused),
	disclosureIssueToggle(disclosureTypes.archive, disclosureIssueTypes.nonEnglish),
	disclosureIssueToggle(disclosureTypes.archive, disclosureIssueTypes.formatting),
)
