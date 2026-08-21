import type { Labrinth } from '@modrinth/api-client'
import {
	isActiveDisclosure,
	isDisclosureCompatibleWithProjectTypes,
	PROJECT_DISCLOSURE_TYPES,
} from '@modrinth/ui'

import type {
	DisclosureFormState,
	DisclosureLockStatus,
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

type NoteDisclosureType = 'advertisements' | 'epilepsy_triggers' | 'archived'

function createNoteModel(
	disclosures: ProjectDisclosureData[],
	type: NoteDisclosureType,
): NoteDisclosure {
	const disclosure = findDisclosure(disclosures, type)
	return {
		enabled: isActiveDisclosure(disclosure),
		note: disclosure?.note ?? '',
	}
}

function nonemptyOrPlaceholder(values: string[]): string[] {
	return values.length > 0 ? [...values] : ['']
}

function createLockStatuses(
	disclosures: ProjectDisclosureData[],
): Record<DisclosureType, DisclosureLockStatus> {
	const lockStatuses = Object.fromEntries(
		PROJECT_DISCLOSURE_TYPES.map((type) => [type, 'unlocked']),
	) as Record<DisclosureType, DisclosureLockStatus>

	for (const disclosure of disclosures) {
		lockStatuses[disclosure.type] = disclosure.lock_status ?? 'unlocked'
	}

	return lockStatuses
}

export function disclosuresToForm(disclosures: ProjectDisclosureData[]): DisclosureFormState {
	const ai = findDisclosure(disclosures, 'ai_content')
	const paidFeatures = findDisclosure(disclosures, 'paid_features')
	const telemetry = findDisclosure(disclosures, 'telemetry')
	const derivative = findDisclosure(disclosures, 'derivative_work')
	const systemInteractions = findDisclosure(disclosures, 'system_interactions')

	return {
		ai: {
			enabled: isActiveDisclosure(ai),
			uses: ai ? [...(ai.uses ?? [])] : [],
			note: ai?.note ?? '',
		},
		advertising: createNoteModel(disclosures, 'advertisements'),
		paidFeatures: {
			enabled: isActiveDisclosure(paidFeatures),
			features: paidFeatures ? nonemptyOrPlaceholder(paidFeatures.features) : [],
		},
		telemetry: {
			enabled: isActiveDisclosure(telemetry),
			consent: telemetry?.consent ?? 'opt_in',
			entries: telemetry ? nonemptyOrPlaceholder(telemetry.data_collected) : [],
		},
		derivative: {
			enabled: isActiveDisclosure(derivative),
			sources: derivative ? derivative.sources.map((source) => ({ ...source })) : [],
		},
		photosensitivity: createNoteModel(disclosures, 'epilepsy_triggers'),
		systemInteractions: {
			enabled: isActiveDisclosure(systemInteractions),
			note: systemInteractions?.note ?? '',
			interactions: systemInteractions ? [...systemInteractions.interactions] : [],
		},
		archived: createNoteModel(disclosures, 'archived'),
		lockStatuses: createLockStatuses(disclosures),
	}
}

export function findDisclosureData<T extends DisclosureType>(
	disclosures: ProjectDisclosureData[] | undefined,
	type: T,
): DisclosureOf<T> | undefined {
	if (!disclosures) return undefined
	return findDisclosure(disclosures, type)
}

export function formToDisclosure(
	form: DisclosureFormState,
	type: DisclosureType,
): ProjectDisclosure {
	switch (type) {
		case 'ai_content':
			return {
				type: 'ai_content',
				uses: [...form.ai.uses],
				note: form.ai.note.trim() || null,
			}
		case 'advertisements':
			return { type: 'advertisements', note: form.advertising.note.trim() || null }
		case 'paid_features':
			return {
				type: 'paid_features',
				features: form.paidFeatures.features.map((feature) => feature.trim()).filter(Boolean),
			}
		case 'telemetry':
			return {
				type: 'telemetry',
				consent: form.telemetry.consent,
				data_collected: form.telemetry.entries.map((entry) => entry.trim()).filter(Boolean),
			}
		case 'derivative_work':
			return {
				type: 'derivative_work',
				sources: form.derivative.sources.map((source) => ({
					label: source.label.trim(),
					link: source.link?.trim() || null,
					note: source.note?.trim() || null,
				})),
			}
		case 'epilepsy_triggers':
			return { type: 'epilepsy_triggers', note: form.photosensitivity.note.trim() || null }
		case 'system_interactions':
			return {
				type: 'system_interactions',
				interactions: [...form.systemInteractions.interactions],
				note: form.systemInteractions.note.trim() || null,
			}
		case 'archived':
			return { type: 'archived', note: form.archived.note.trim() || null }
	}
}

export function formToDisclosures(form: DisclosureFormState): ProjectDisclosure[] {
	const set: ProjectDisclosure[] = []

	if (form.ai.enabled) {
		set.push(formToDisclosure(form, 'ai_content'))
	}
	if (form.advertising.enabled) {
		set.push(formToDisclosure(form, 'advertisements'))
	}
	if (form.paidFeatures.enabled) {
		set.push(formToDisclosure(form, 'paid_features'))
	}
	if (form.telemetry.enabled) {
		set.push(formToDisclosure(form, 'telemetry'))
	}
	if (form.derivative.enabled) {
		set.push(formToDisclosure(form, 'derivative_work'))
	}
	if (form.photosensitivity.enabled) {
		set.push(formToDisclosure(form, 'epilepsy_triggers'))
	}
	if (form.systemInteractions.enabled) {
		set.push(formToDisclosure(form, 'system_interactions'))
	}
	if (form.archived.enabled) {
		set.push(formToDisclosure(form, 'archived'))
	}

	return set
}

export function toModifyRequests(
	form: DisclosureFormState,
	previous: DisclosureFormState,
): Labrinth.Projects.v3.ModifyProjectDisclosures[] {
	const next = formToDisclosures(form)
	const previousDisclosures = formToDisclosures(previous)
	const previousByType = new Map(
		previousDisclosures.map((disclosure) => [disclosure.type, disclosure]),
	)
	const nextTypes = new Set(next.map((disclosure) => disclosure.type))

	const remove: DisclosureType[] = []
	const contentOnly: ProjectDisclosure[] = []
	const lockChangeGroups = new Map<DisclosureLockStatus, ProjectDisclosure[]>()
	const disabledLockGroups = new Map<DisclosureLockStatus, ProjectDisclosure[]>()

	for (const disclosure of next) {
		const existing = previousByType.get(disclosure.type)
		const contentChanged = !existing || JSON.stringify(existing) !== JSON.stringify(disclosure)
		const previousLock = previous.lockStatuses[disclosure.type] ?? 'unlocked'
		const nextLock = form.lockStatuses[disclosure.type] ?? 'unlocked'
		const lockChanged = previousLock !== nextLock

		if (!contentChanged && !lockChanged) {
			continue
		}

		if (lockChanged) {
			const group = lockChangeGroups.get(nextLock) ?? []
			group.push(disclosure)
			lockChangeGroups.set(nextLock, group)
		} else {
			contentOnly.push(disclosure)
		}
	}

	for (const type of PROJECT_DISCLOSURE_TYPES) {
		if (nextTypes.has(type)) {
			continue
		}

		const previousLock = previous.lockStatuses[type] ?? 'unlocked'
		const nextLock = form.lockStatuses[type] ?? 'unlocked'
		const lockChanged = previousLock !== nextLock
		const wasEnabled = previousByType.has(type)

		if (wasEnabled && !lockChanged) {
			remove.push(type)
			continue
		}

		if (!lockChanged) {
			continue
		}

		const group = disabledLockGroups.get(nextLock) ?? []
		group.push(formToDisclosure(form, type))
		disabledLockGroups.set(nextLock, group)
	}

	const requests: Labrinth.Projects.v3.ModifyProjectDisclosures[] = []

	if (remove.length > 0) {
		requests.push({ set: [], remove })
	}

	if (contentOnly.length > 0) {
		requests.push({ set: contentOnly, remove: [] })
	}

	for (const [lockStatus, set] of lockChangeGroups) {
		requests.push({ set, remove: [], lock_status: lockStatus })
	}

	for (const [lockStatus, set] of disabledLockGroups) {
		requests.push({ set, remove: [], lock_status: lockStatus })
		requests.push({ set: [], remove: set.map((disclosure) => disclosure.type) })
	}

	return requests
}

export function getDisclosureFormSnapshot(form: DisclosureFormState) {
	return {
		disclosures: formToDisclosures(form),
		lockStatuses: { ...form.lockStatuses },
	}
}

export type DisclosureFormIssue =
	| 'advertising-note'
	| 'paid-features-empty'
	| 'telemetry-empty'
	| 'derivative-empty'
	| 'derivative-source-label'
	| 'photosensitivity-note'
	| 'system-interactions-note'

export function getDisclosureFormIssues(
	form: DisclosureFormState,
	projectTypes?: readonly string[],
): DisclosureFormIssue[] {
	const issues: DisclosureFormIssue[] = []
	const missingNote = (model: NoteDisclosure) => model.enabled && !model.note.trim()
	const compatible = (type: DisclosureType) =>
		!projectTypes || isDisclosureCompatibleWithProjectTypes(type, projectTypes)

	if (compatible('advertisements') && missingNote(form.advertising)) {
		issues.push('advertising-note')
	}
	if (
		compatible('paid_features') &&
		form.paidFeatures.enabled &&
		!form.paidFeatures.features.some((feature) => feature.trim())
	) {
		issues.push('paid-features-empty')
	}
	if (
		compatible('telemetry') &&
		form.telemetry.enabled &&
		!form.telemetry.entries.some((entry) => entry.trim())
	) {
		issues.push('telemetry-empty')
	}
	if (compatible('derivative_work') && form.derivative.enabled) {
		if (form.derivative.sources.length === 0) {
			issues.push('derivative-empty')
		} else if (form.derivative.sources.some((source) => !source.label?.trim())) {
			issues.push('derivative-source-label')
		}
	}
	if (compatible('epilepsy_triggers') && missingNote(form.photosensitivity)) {
		issues.push('photosensitivity-note')
	}
	if (
		compatible('system_interactions') &&
		form.systemInteractions.enabled &&
		!form.systemInteractions.note.trim()
	) {
		issues.push('system-interactions-note')
	}

	return issues
}
