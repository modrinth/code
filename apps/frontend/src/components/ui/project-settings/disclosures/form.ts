import type { Labrinth } from '@modrinth/api-client'
import { isDisclosureCompatibleWithProjectTypes, PROJECT_DISCLOSURE_TYPES } from '@modrinth/ui'

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
		enabled: !!disclosure,
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
		lockStatuses[disclosure.type] = disclosure.lock_status
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
		systemInteractions: {
			enabled: !!systemInteractions,
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
		set.push({
			type: 'system_interactions',
			interactions: [...form.systemInteractions.interactions],
			note: form.systemInteractions.note.trim() || null,
		})
	}

	if (form.archived.enabled) {
		set.push({ type: 'archived', note: form.archived.note.trim() || null })
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

	const remove = previousDisclosures
		.map((disclosure) => disclosure.type)
		.filter((type) => !nextTypes.has(type))

	const contentOnly: ProjectDisclosure[] = []
	const lockChangeGroups = new Map<DisclosureLockStatus, ProjectDisclosure[]>()

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

	return requests
}

export function getDisclosureFormSnapshot(form: DisclosureFormState) {
	const disclosures = formToDisclosures(form)
	return {
		disclosures,
		lockStatuses: Object.fromEntries(
			disclosures.map((disclosure) => [
				disclosure.type,
				form.lockStatuses[disclosure.type] ?? 'unlocked',
			]),
		) as Partial<Record<DisclosureType, DisclosureLockStatus>>,
	}
}

export function toCachedDisclosures(
	set: ProjectDisclosure[],
	previous: ProjectDisclosureData[] = [],
	defaults?: {
		setByModerator?: boolean
		lockStatuses?: Partial<Record<DisclosureType, DisclosureLockStatus>>
	},
): ProjectDisclosureData[] {
	const previousByType = new Map(previous.map((disclosure) => [disclosure.type, disclosure]))
	const now = new Date().toISOString()

	return set.map((disclosure) => {
		const existing = previousByType.get(disclosure.type)
		return {
			...disclosure,
			set_by_moderator: defaults?.setByModerator ?? existing?.set_by_moderator ?? false,
			lock_status: defaults?.lockStatuses?.[disclosure.type] ?? existing?.lock_status ?? 'unlocked',
			updated_at: now,
			updated_by: existing?.updated_by ?? null,
		}
	})
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
