import { TriangleAlertIcon } from '@modrinth/assets'

import disclosuresHeaderMessage from '../messages/checklist/messages/disclosures/header.md'
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
import misusedAdsMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ads.md'
import misusedAiMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/ai.md'
import misusedAiFunctionalityMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/ai-functionality.md'
import misusedAiAssetsMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/ai-usages/assets.md'
import misusedAiCodeMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/ai-usages/code.md'
import misusedAiTextMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/ai-usages/text.md'
import misusedAiListIntroMessage from '../messages/checklist/messages/disclosures/misused-disclosures/ai/list-intro.md'
import misusedArchiveMessage from '../messages/checklist/messages/disclosures/misused-disclosures/archive.md'
import misusedDerivativeContentMessage from '../messages/checklist/messages/disclosures/misused-disclosures/derivative-content.md'
import misusedListIntroMessage from '../messages/checklist/messages/disclosures/misused-disclosures/list-intro.md'
import misusedPaidFeaturesMessage from '../messages/checklist/messages/disclosures/misused-disclosures/paid-features.md'
import misusedPhotosensitivityMessage from '../messages/checklist/messages/disclosures/misused-disclosures/photosensitivity.md'
import misusedSystemInteractionsMessage from '../messages/checklist/messages/disclosures/misused-disclosures/system-interactions.md'
import misusedTelemetryMessage from '../messages/checklist/messages/disclosures/misused-disclosures/telemetry.md'
import nonEnglishMessage from '../messages/checklist/messages/disclosures/non-english.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const disclosuresIssue = issue({
	id: 'disclosures',
	title: 'Invalid disclosures',
	category: 'Disclosures',
	message: ({ selected }) => {
		const toggleIds = new Set(selected.toggleIds)
		const missingParts: string[] = []
		const misusedParts: string[] = []

		for (const field of [
			'ai',
			'ai-functionality',
			'ads',
			'paid-features',
			'telemetry',
			'derivative-content',
			'photosensitivity',
			'system-interactions',
			'archive',
		] as const) {
			const missingSelected = toggleIds.has(`disclosures-missing-${field}`)
			const misusedSelected = toggleIds.has(`disclosures-misused-${field}`)
			if (!missingSelected && !misusedSelected) continue

			let missingMessage: string
			let misusedMessage: string
			switch (field) {
				case 'ai':
					missingMessage = missingAiMessage
					misusedMessage = misusedAiMessage
					break
				case 'ai-functionality':
					missingMessage = missingAiFunctionalityMessage
					misusedMessage = misusedAiFunctionalityMessage
					break
				case 'ads':
					missingMessage = missingAdsMessage
					misusedMessage = misusedAdsMessage
					break
				case 'paid-features':
					missingMessage = missingPaidFeaturesMessage
					misusedMessage = misusedPaidFeaturesMessage
					break
				case 'telemetry':
					missingMessage = missingTelemetryMessage
					misusedMessage = misusedTelemetryMessage
					break
				case 'derivative-content':
					missingMessage = missingDerivativeContentMessage
					misusedMessage = misusedDerivativeContentMessage
					break
				case 'photosensitivity':
					missingMessage = missingPhotosensitivityMessage
					misusedMessage = misusedPhotosensitivityMessage
					break
				case 'system-interactions':
					missingMessage = missingSystemInteractionsMessage
					misusedMessage = misusedSystemInteractionsMessage
					break
				case 'archive':
					missingMessage = missingArchiveMessage
					misusedMessage = misusedArchiveMessage
					break
				default:
					continue
			}

			if (missingSelected) {
				missingParts.push(missingMessage)
				if (field === 'ai') {
					const usages: string[] = []
					if (toggleIds.has('disclosures-missing-ai-code')) usages.push(missingAiCodeMessage)
					if (toggleIds.has('disclosures-missing-ai-assets')) usages.push(missingAiAssetsMessage)
					if (toggleIds.has('disclosures-missing-ai-text')) usages.push(missingAiTextMessage)
					if (usages.length) missingParts.push(missingAiListIntroMessage, ...usages)
				}
				if (field === 'telemetry') {
					if (toggleIds.has('disclosures-missing-telemetry-opt-in')) {
						missingParts.push(missingTelemetryOptInMessage)
					}
					if (toggleIds.has('disclosures-missing-telemetry-opt-out')) {
						missingParts.push(missingTelemetryOptOutMessage)
					}
					if (toggleIds.has('disclosures-missing-telemetry-always')) {
						missingParts.push(missingTelemetryAlwaysMessage)
					}
				}
			}

			if (misusedSelected) {
				misusedParts.push(misusedMessage)
				if (field === 'ai') {
					const usages: string[] = []
					if (toggleIds.has('disclosures-misused-ai-code')) usages.push(misusedAiCodeMessage)
					if (toggleIds.has('disclosures-misused-ai-assets')) usages.push(misusedAiAssetsMessage)
					if (toggleIds.has('disclosures-misused-ai-text')) usages.push(misusedAiTextMessage)
					if (usages.length) misusedParts.push(misusedAiListIntroMessage, ...usages)
				}
			}
		}

		const groups: string[] = []
		if (missingParts.length) {
			const body = missingParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${missingListIntroMessage.trim()}\n\n${body}`)
		}
		if (misusedParts.length) {
			const body = misusedParts.map((part) => part.trimEnd()).join('\n')
			groups.push(`${misusedListIntroMessage.trim()}\n\n${body}`)
		}
		return groups.length ? [disclosuresHeaderMessage.trim(), ...groups].join('\n\n') : ''
	},
	suggestedStatus: ({ selected }) => {
		const toggleIds = new Set(selected.toggleIds)
		const rejected = ['telemetry', 'derivative-content', 'photosensitivity', 'system-interactions']
		if (
			rejected.some(
				(field) =>
					toggleIds.has(`disclosures-missing-${field}`) ||
					toggleIds.has(`disclosures-misused-${field}`),
			)
		)
			return 'rejected'
		if (
			[...toggleIds].some((id) => id.startsWith('disclosures-misused-')) ||
			['ai', 'ai-functionality', 'ads', 'paid-features'].some((field) =>
				toggleIds.has(`disclosures-missing-${field}`),
			)
		)
			return 'flagged'
		return undefined
	},
})

export const disclosuresNonEnglishIssue = issue({
	id: 'disclosures-non-english',
	title: 'Non-English disclosure information',
	category: 'Disclosures',
	message: nonEnglishMessage,
	suggestedStatus: 'flagged',
})

export const aiDisclosureReviewPanel = panel({
	title: 'AI Usage',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-ai',
		label: 'Disclosure Missing',
		issueListLabel: 'AI Usage',
	}),
	section({
		label: 'What kind of AI content?',
		shown: ({ selected }) => selected.toggleIds.includes('disclosures-missing-ai'),
	}).content(
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-ai-code',
			label: 'Code',
			issueListLabel: 'AI Usage: Code',
			issueListGroup: 'Disclosure Missing',
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-ai-assets',
			label: 'Assets',
			issueListLabel: 'AI Usage: Assets',
			issueListGroup: 'Disclosure Missing',
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-ai-text',
			label: 'Text',
			issueListLabel: 'AI Usage: Text',
			issueListGroup: 'Disclosure Missing',
		}),
	),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-ai',
		label: 'Misused',
		issueListLabel: 'AI Usage',
	}),
	section({
		label: 'What kind of AI content?',
		shown: ({ selected }) => selected.toggleIds.includes('disclosures-misused-ai'),
	}).content(
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-misused-ai-code',
			label: 'Code',
			issueListLabel: 'AI Usage: Code',
			issueListGroup: 'Misused',
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-misused-ai-assets',
			label: 'Assets',
			issueListLabel: 'AI Usage: Assets',
			issueListGroup: 'Misused',
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-misused-ai-text',
			label: 'Text',
			issueListLabel: 'AI Usage: Text',
			issueListGroup: 'Misused',
		}),
	),
)

export const aiFunctionalityDisclosureReviewPanel = panel({
	title: 'AI Functionality',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-ai-functionality',
		label: 'Disclosure Missing',
		issueListLabel: 'AI Functionality',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-ai-functionality',
		label: 'Misused',
		issueListLabel: 'AI Functionality',
	}),
)

export const adsDisclosureReviewPanel = panel({
	title: 'Advertisements',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-ads',
		label: 'Disclosure Missing',
		issueListLabel: 'Advertisements',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-ads',
		label: 'Misused',
		issueListLabel: 'Advertisements',
	}),
)

export const paidFeaturesDisclosureReviewPanel = panel({
	title: 'Paid Features',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-paid-features',
		label: 'Disclosure Missing',
		issueListLabel: 'Paid Features',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-paid-features',
		label: 'Misused',
		issueListLabel: 'Paid Features',
	}),
)

export const telemetryDisclosureReviewPanel = panel({
	title: 'Telemetry',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-telemetry',
		label: 'Disclosure Missing',
		issueListLabel: 'Telemetry',
	}),
	section({
		label: 'What is the telemetry’s consent model?',
		shown: ({ selected }) => selected.toggleIds.includes('disclosures-missing-telemetry'),
	}).content(
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-telemetry-opt-in',
			label: 'Opt In',
			issueListLabel: 'Telemetry: Opt In',
			issueListGroup: 'Disclosure Missing',
			disabled: ({ selected }) =>
				selected.toggleIds.includes('disclosures-missing-telemetry-opt-out') ||
				selected.toggleIds.includes('disclosures-missing-telemetry-always'),
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-telemetry-opt-out',
			label: 'Opt Out',
			issueListLabel: 'Telemetry: Opt Out',
			issueListGroup: 'Disclosure Missing',
			disabled: ({ selected }) =>
				selected.toggleIds.includes('disclosures-missing-telemetry-opt-in') ||
				selected.toggleIds.includes('disclosures-missing-telemetry-always'),
		}),
		toggle({
			issue: disclosuresIssue,
			id: 'disclosures-missing-telemetry-always',
			label: 'Always Online',
			issueListLabel: 'Telemetry: Always Online',
			issueListGroup: 'Disclosure Missing',
			disabled: ({ selected }) =>
				selected.toggleIds.includes('disclosures-missing-telemetry-opt-in') ||
				selected.toggleIds.includes('disclosures-missing-telemetry-opt-out'),
		}),
	),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-telemetry',
		label: 'Misused',
		issueListLabel: 'Telemetry',
	}),
)

export const derivativeContentDisclosureReviewPanel = panel({
	title: 'Derivative Content',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-derivative-content',
		label: 'Disclosure Missing',
		issueListLabel: 'Derivative Content',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-derivative-content',
		label: 'Misused',
		issueListLabel: 'Derivative Content',
	}),
)

export const photosensitivityDisclosureReviewPanel = panel({
	title: 'Photosensitivity',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-photosensitivity',
		label: 'Disclosure Missing',
		issueListLabel: 'Photosensitivity',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-photosensitivity',
		label: 'Misused',
		issueListLabel: 'Photosensitivity',
	}),
)

export const systemInteractionsDisclosureReviewPanel = panel({
	title: 'System Interactions',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-system-interactions',
		label: 'Disclosure Missing',
		issueListLabel: 'System Interactions',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-system-interactions',
		label: 'Misused',
		issueListLabel: 'System Interactions',
	}),
)

export const archiveDisclosureReviewPanel = panel({
	title: 'Archive',
	hint: 'Has this project accurately disclosed this content?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-missing-archive',
		label: 'Disclosure Missing',
		issueListLabel: 'Archive',
	}),
	toggle({
		issue: disclosuresIssue,
		id: 'disclosures-misused-archive',
		label: 'Misused',
		issueListLabel: 'Archive',
	}),
)

export const disclosuresReviewPanel = panel({
	title: 'Disclosures',
	hint: 'Has this project selected all proper content disclosures?',
	icon: TriangleAlertIcon,
}).content(
	toggle({
		issue: disclosuresNonEnglishIssue,
		label: 'Non-English',
	}),
)
