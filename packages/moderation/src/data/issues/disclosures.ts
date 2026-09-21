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

const disclosureFields = {
	ai: 'AI Usage',
	'ai-functionality': 'AI Functionality',
	ads: 'Advertisements',
	'paid-features': 'Paid Features',
	telemetry: 'Telemetry',
	'derivative-content': 'Derivative Content',
	photosensitivity: 'Photosensitivity',
	'system-interactions': 'System Interactions',
	archive: 'Archive',
} as const

type DisclosureField = keyof typeof disclosureFields

function createDisclosurePanel(field: DisclosureField) {
	return panel({
		field: `${field}-disclosure`,
		title: disclosureFields[field],
		hint: 'Has this project accurately disclosed this content?',
		icon: TriangleAlertIcon,
	}).content(
		...[
			{
				problem: 'missing',
				label: 'Disclosure Missing',
				issue: disclosuresMissingDisclosuresIssue,
			},
			{ problem: 'misused', label: 'Misused', issue: disclosuresMisusedDisclosuresIssue },
		].flatMap(({ problem, label, issue }) => {
			const id = `disclosures-${problem}-${field}`
			return [
				toggle({ issue, label, id }),
				...(field === 'ai'
					? [
							section({
								label: 'What kind of AI content?',
								shown: ({ selected }) => selected.toggleIds.includes(id),
							}).content(
								...Object.entries({ code: 'Code', assets: 'Assets', text: 'Text' }).map(
									([usage, label]) => toggle({ issue, label, id: `${id}-${usage}` }),
								),
							),
						]
					: []),
				...(field === 'telemetry' && problem === 'missing'
					? [
							section({
								label: 'What is the telemetry’s consent model?',
								shown: ({ selected }) => selected.toggleIds.includes(id),
							}).content(
								...Object.entries({
									'opt-in': 'Opt In',
									'opt-out': 'Opt Out',
									always: 'Always Online',
								}).map(([consent, label]) =>
									toggle({
										issue,
										label,
										id: `${id}-${consent}`,
										disabled: ({ selected }) =>
											Object.keys(consentMessages).some(
												(key) => key !== consent && selected.toggleIds.includes(`${id}-${key}`),
											),
									}),
								),
							),
						]
					: []),
			]
		}),
	)
}

export const aiDisclosureReviewPanel = createDisclosurePanel('ai')
export const aiFunctionalityDisclosureReviewPanel = createDisclosurePanel('ai-functionality')
export const adsDisclosureReviewPanel = createDisclosurePanel('ads')
export const paidFeaturesDisclosureReviewPanel = createDisclosurePanel('paid-features')
export const telemetryDisclosureReviewPanel = createDisclosurePanel('telemetry')
export const derivativeContentDisclosureReviewPanel = createDisclosurePanel('derivative-content')
export const photosensitivityDisclosureReviewPanel = createDisclosurePanel('photosensitivity')
export const systemInteractionsDisclosureReviewPanel = createDisclosurePanel('system-interactions')
export const archiveDisclosureReviewPanel = createDisclosurePanel('archive')

export const disclosuresReviewPanel = panel({
	field: 'disclosures',
	title: 'Disclosures',
	hint: 'Has this project selected all proper content disclosures?',
	icon: TriangleAlertIcon,
}).content(toggle({ issue: disclosuresNonEnglishIssue, label: 'Non-English' }))
