import type { Labrinth } from '@modrinth/api-client'

import type {
	DisclosureFormState,
	DisclosureOf,
	DisclosureType,
	NoteDisclosure,
	ProjectDisclosure,
	ProjectDisclosureData,
} from './types'

function findDisclosure<T extends DisclosureType>(
	disclosures: ProjectDisclosureData[],
	type: T,
): DisclosureOf<T> | undefined {
	return disclosures.find((disclosure): disclosure is DisclosureOf<T> => disclosure.type === type)
}

type NoteDisclosureType = 'advertisements' | 'epilepsy_triggers' | 'system_interactions'

function createNoteModel(
	disclosures: ProjectDisclosureData[],
	type: NoteDisclosureType,
): NoteDisclosure {
	const disclosure = findDisclosure(disclosures, type)
	return {
		enabled: !!disclosure,
		note: disclosure?.note ?? '',
	}
}

function nonemptyOrPlaceholder(values: string[]): string[] {
	return values.length > 0 ? [...values] : ['']
}

export function disclosuresToForm(disclosures: ProjectDisclosureData[]): DisclosureFormState {
	const ai = findDisclosure(disclosures, 'ai_content')
	const paidFeatures = findDisclosure(disclosures, 'paid_features')
	const telemetry = findDisclosure(disclosures, 'telemetry')
	const derivative = findDisclosure(disclosures, 'derivative_work')

	return {
		ai: {
			enabled: !!ai,
			uses: ai ? [...(ai.uses ?? [])] : [],
			note: ai?.note ?? '',
		},
		advertising: createNoteModel(disclosures, 'advertisements'),
		paidFeatures: {
			enabled: !!paidFeatures,
			features: paidFeatures ? nonemptyOrPlaceholder(paidFeatures.features) : [],
		},
		telemetry: {
			enabled: !!telemetry,
			consent: telemetry?.consent ?? 'opt_in',
			entries: telemetry ? nonemptyOrPlaceholder(telemetry.data_collected) : [],
		},
		derivative: {
			enabled: !!derivative,
			sources: derivative ? derivative.sources.map((source) => ({ ...source })) : [],
		},
		photosensitivity: createNoteModel(disclosures, 'epilepsy_triggers'),
		systemInteractions: createNoteModel(disclosures, 'system_interactions'),
	}
}

export function formToDisclosures(form: DisclosureFormState): ProjectDisclosure[] {
	const set: ProjectDisclosure[] = []

	if (form.ai.enabled) {
		set.push({
			type: 'ai_content',
			uses: [...form.ai.uses],
			note: form.ai.note.trim() || null,
		})
	}

	if (form.advertising.enabled) {
		set.push({ type: 'advertisements', note: form.advertising.note.trim() || null })
	}

	if (form.paidFeatures.enabled) {
		set.push({
			type: 'paid_features',
			features: form.paidFeatures.features.map((feature) => feature.trim()).filter(Boolean),
		})
	}

	if (form.telemetry.enabled) {
		set.push({
			type: 'telemetry',
			consent: form.telemetry.consent,
			data_collected: form.telemetry.entries.map((entry) => entry.trim()).filter(Boolean),
		})
	}

	if (form.derivative.enabled) {
		set.push({
			type: 'derivative_work',
			sources: form.derivative.sources.map((source) => ({
				label: source.label.trim(),
				link: source.link?.trim() || null,
				note: source.note?.trim() || null,
			})),
		})
	}

	if (form.photosensitivity.enabled) {
		set.push({ type: 'epilepsy_triggers', note: form.photosensitivity.note.trim() || null })
	}

	if (form.systemInteractions.enabled) {
		set.push({ type: 'system_interactions', note: form.systemInteractions.note.trim() || null })
	}

	return set
}

export function toModifyRequest(
	form: DisclosureFormState,
	previous: DisclosureFormState,
): Labrinth.Projects.v3.ModifyProjectDisclosures {
	const set = formToDisclosures(form)
	const nextTypes = new Set(set.map((disclosure) => disclosure.type))
	return {
		set,
		remove: formToDisclosures(previous)
			.map((disclosure) => disclosure.type)
			.filter((type) => !nextTypes.has(type)),
	}
}

export function toCachedDisclosures(set: ProjectDisclosure[]): ProjectDisclosureData[] {
	const now = new Date().toISOString()
	return set.map((disclosure) => ({
		...disclosure,
		set_by_moderator: false,
		updated_at: now,
		updated_by: null,
	}))
}

export type DisclosureFormIssue =
	| 'advertising-note'
	| 'paid-features-empty'
	| 'telemetry-empty'
	| 'derivative-empty'
	| 'derivative-source-label'
	| 'photosensitivity-note'
	| 'system-interactions-note'

export function getDisclosureFormIssues(form: DisclosureFormState): DisclosureFormIssue[] {
	const issues: DisclosureFormIssue[] = []
	const missingNote = (model: NoteDisclosure) => model.enabled && !model.note.trim()

	if (missingNote(form.advertising)) {
		issues.push('advertising-note')
	}
	if (form.paidFeatures.enabled && !form.paidFeatures.features.some((feature) => feature.trim())) {
		issues.push('paid-features-empty')
	}
	if (form.telemetry.enabled && !form.telemetry.entries.some((entry) => entry.trim())) {
		issues.push('telemetry-empty')
	}
	if (form.derivative.enabled) {
		if (form.derivative.sources.length === 0) {
			issues.push('derivative-empty')
		} else if (form.derivative.sources.some((source) => !source.label?.trim())) {
			issues.push('derivative-source-label')
		}
	}
	if (missingNote(form.photosensitivity)) {
		issues.push('photosensitivity-note')
	}
	if (missingNote(form.systemInteractions)) {
		issues.push('system-interactions-note')
	}

	return issues
}
