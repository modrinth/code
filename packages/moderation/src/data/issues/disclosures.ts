import { TriangleAlertIcon } from '@modrinth/assets'

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
import missingHeaderMessage from '../messages/checklist/messages/disclosures/missing-disclosures/missing-disclosures-header.md'
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
import misusedHeaderMessage from '../messages/checklist/messages/disclosures/misused-disclosures/misused-disclosures-header.md'
import misusedPaidFeaturesMessage from '../messages/checklist/messages/disclosures/misused-disclosures/paid-features.md'
import misusedPhotosensitivityMessage from '../messages/checklist/messages/disclosures/misused-disclosures/photosensitivity.md'
import misusedSystemInteractionsMessage from '../messages/checklist/messages/disclosures/misused-disclosures/system-interactions.md'
import misusedTelemetryMessage from '../messages/checklist/messages/disclosures/misused-disclosures/telemetry.md'
import nonEnglishMessage from '../messages/checklist/messages/disclosures/non-english.md'
import { issue, panel, section, toggle } from './component-builders/builders'

const missingDisclosureMessages: Record<string, string> = {
	ai: missingAiMessage,
	'ai-functionality': missingAiFunctionalityMessage,
	ads: missingAdsMessage,
	'paid-features': missingPaidFeaturesMessage,
	telemetry: missingTelemetryMessage,
	'derivative-content': missingDerivativeContentMessage,
	photosensitivity: missingPhotosensitivityMessage,
	'system-interactions': missingSystemInteractionsMessage,
	archive: missingArchiveMessage,
}

const missingAiMessages: Record<string, string> = {
	code: missingAiCodeMessage,
	assets: missingAiAssetsMessage,
	text: missingAiTextMessage,
}

const consentMessages: Record<string, string> = {
	'opt-in': missingTelemetryOptInMessage,
	'opt-out': missingTelemetryOptOutMessage,
	always: missingTelemetryAlwaysMessage,
}

export const disclosuresMissingDisclosuresIssue = issue({
	id: 'disclosures-missing-disclosures',
	message: ({ selected }) => {
		const parts = Object.entries(missingDisclosureMessages).flatMap(([key, message]) => {
			if (!selected.toggleIds.includes('disclosures-missing-' + key)) return []
			const parts = [message]
			if (key === 'ai') {
				const usages = Object.entries(missingAiMessages)
					.filter(([key]) => selected.toggleIds.includes('disclosures-missing-ai-' + key))
					.map(([, message]) => message)
				if (usages.length) parts.push(missingAiListIntroMessage, ...usages)
			}
			if (key === 'telemetry')
				parts.push(
					...Object.entries(consentMessages)
						.filter(([key]) => selected.toggleIds.includes('disclosures-missing-telemetry-' + key))
						.map(([, message]) => message),
				)
			return parts
		})
		return [
			missingHeaderMessage,
			...(parts.length ? [missingListIntroMessage, ...parts] : []),
		].join('\n')
	},
	suggestedStatus: ({ selected }) => {
		const rejected = ['telemetry', 'derivative-content', 'photosensitivity', 'system-interactions']
		if (rejected.some((key) => selected.toggleIds.includes('disclosures-missing-' + key)))
			return 'rejected'
		return ['ai', 'ai-functionality', 'ads', 'paid-features'].some((key) =>
			selected.toggleIds.includes('disclosures-missing-' + key),
		)
			? 'flagged'
			: undefined
	},
})

const misusedDisclosureMessages: Record<string, string> = {
	ai: misusedAiMessage,
	'ai-functionality': misusedAiFunctionalityMessage,
	ads: misusedAdsMessage,
	'paid-features': misusedPaidFeaturesMessage,
	telemetry: misusedTelemetryMessage,
	'derivative-content': misusedDerivativeContentMessage,
	photosensitivity: misusedPhotosensitivityMessage,
	'system-interactions': misusedSystemInteractionsMessage,
	archive: misusedArchiveMessage,
}

const misusedAiMessages: Record<string, string> = {
	code: misusedAiCodeMessage,
	assets: misusedAiAssetsMessage,
	text: misusedAiTextMessage,
}

export const disclosuresMisusedDisclosuresIssue = issue({
	id: 'disclosures-misused-disclosures',
	message: ({ selected }) => {
		const parts = Object.entries(misusedDisclosureMessages).flatMap(([key, message]) => {
			if (!selected.toggleIds.includes('disclosures-misused-' + key)) return []
			const parts = [message]
			if (key === 'ai') {
				const usages = Object.entries(misusedAiMessages)
					.filter(([key]) => selected.toggleIds.includes('disclosures-misused-ai-' + key))
					.map(([, message]) => message)
				if (usages.length) parts.push(misusedAiListIntroMessage, ...usages)
			}
			return parts
		})
		return [
			misusedHeaderMessage,
			...(parts.length ? [misusedListIntroMessage, ...parts] : []),
		].join('\n')
	},
	suggestedStatus: ({ selected }) => {
		const rejected = ['telemetry', 'derivative-content', 'photosensitivity', 'system-interactions']
		if (rejected.some((key) => selected.toggleIds.includes('disclosures-misused-' + key)))
			return 'rejected'
		return 'flagged'
	},
})

export const disclosuresNonEnglishIssue = issue({
	id: 'disclosures-non-english',
	message: nonEnglishMessage,
	suggestedStatus: 'flagged',
})

export const disclosuresReviewPanel = panel({
	field: 'disclosures',
	title: 'Disclosures',
	hint: 'Has this project selected all proper content disclosures?',
	icon: TriangleAlertIcon,
}).content(
	toggle({ issue: disclosuresMissingDisclosuresIssue, label: 'Disclosures Missing' }),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(disclosuresMissingDisclosuresIssue.id),
	}).content(
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'AI Usage',
			id: 'disclosures-missing-ai',
		}),
		section({
			label: 'What kind of AI content?',
			shown: ({ selected }) => selected.toggleIds.includes('disclosures-missing-ai'),
		}).content(
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Code',
				id: 'disclosures-missing-ai-code',
			}),
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Assets',
				id: 'disclosures-missing-ai-assets',
			}),
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Text',
				id: 'disclosures-missing-ai-text',
			}),
		),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'AI Functionality',
			id: 'disclosures-missing-ai-functionality',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Advertisements',
			id: 'disclosures-missing-ads',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Paid Features',
			id: 'disclosures-missing-paid-features',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Telemetry',
			id: 'disclosures-missing-telemetry',
		}),
		section({
			label: 'What is the telemetry’s consent model?',
			shown: ({ selected }) => selected.toggleIds.includes('disclosures-missing-telemetry'),
		}).content(
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Opt In',
				id: 'disclosures-missing-telemetry-opt-in',
				disabled: ({ selected }) =>
					['disclosures-missing-telemetry-opt-out', 'disclosures-missing-telemetry-always'].some(
						(id) => selected.toggleIds.includes(id),
					),
			}),
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Opt Out',
				id: 'disclosures-missing-telemetry-opt-out',
				disabled: ({ selected }) =>
					['disclosures-missing-telemetry-opt-in', 'disclosures-missing-telemetry-always'].some(
						(id) => selected.toggleIds.includes(id),
					),
			}),
			toggle({
				issue: disclosuresMissingDisclosuresIssue,
				label: 'Always Online',
				id: 'disclosures-missing-telemetry-always',
				disabled: ({ selected }) =>
					['disclosures-missing-telemetry-opt-in', 'disclosures-missing-telemetry-opt-out'].some(
						(id) => selected.toggleIds.includes(id),
					),
			}),
		),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Derivative Content',
			id: 'disclosures-missing-derivative-content',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Photosensitivity',
			id: 'disclosures-missing-photosensitivity',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'System Interactions',
			id: 'disclosures-missing-system-interactions',
		}),
		toggle({
			issue: disclosuresMissingDisclosuresIssue,
			label: 'Archive',
			id: 'disclosures-missing-archive',
		}),
	),
	toggle({ issue: disclosuresMisusedDisclosuresIssue, label: 'Misused' }),
	section({
		shown: (ctx) => ctx.selected.issueIds.includes(disclosuresMisusedDisclosuresIssue.id),
	}).content(
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'AI Usage',
			id: 'disclosures-misused-ai',
		}),
		section({
			label: 'What kind of AI content?',
			shown: ({ selected }) => selected.toggleIds.includes('disclosures-misused-ai'),
		}).content(
			toggle({
				issue: disclosuresMisusedDisclosuresIssue,
				label: 'Code',
				id: 'disclosures-misused-ai-code',
			}),
			toggle({
				issue: disclosuresMisusedDisclosuresIssue,
				label: 'Assets',
				id: 'disclosures-misused-ai-assets',
			}),
			toggle({
				issue: disclosuresMisusedDisclosuresIssue,
				label: 'Text',
				id: 'disclosures-misused-ai-text',
			}),
		),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'AI Functionality',
			id: 'disclosures-misused-ai-functionality',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Advertisements',
			id: 'disclosures-misused-ads',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Paid Features',
			id: 'disclosures-misused-paid-features',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Telemetry',
			id: 'disclosures-misused-telemetry',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Derivative Content',
			id: 'disclosures-misused-derivative-content',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Photosensitivity',
			id: 'disclosures-misused-photosensitivity',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'System Interactions',
			id: 'disclosures-misused-system-interactions',
		}),
		toggle({
			issue: disclosuresMisusedDisclosuresIssue,
			label: 'Archive',
			id: 'disclosures-misused-archive',
		}),
	),
	toggle({ issue: disclosuresNonEnglishIssue, label: 'Non-English' }),
)
