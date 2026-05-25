import type { Labrinth } from '@modrinth/api-client'

import { defineMessage, type MessageDescriptor } from '../../composables/i18n'

export const permissionKinds: Labrinth.Attribution.Internal.AttributionResolutionKind[] = [
	'license',
	'my_project',
	'special_permissions',
	'no_permission',
	'globally_allowed',
]

/** Combobox value when the user picks a non-SPDX custom license (stored as `{ name }`). */
export const CUSTOM_LICENSE_VALUE = '__custom__'

export type ProjectPermissionField =
	| 'license_id'
	| 'custom_license'
	| 'link_to_work'
	| 'notes'
	| 'image_urls'

export const PERMISSION_REASONS = {
	license: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.license',
			defaultMessage: 'License',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.reason.license.description',
			defaultMessage: 'The license of this work permits you to redistribute it in your modpack.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.reason.license.proof-images-description',
			defaultMessage: 'Upload supporting documentation related to this license.',
		}),
		proofImagesOptional: true,
		fields: ['license_id', 'custom_license', 'link_to_work', 'notes', 'image_urls'] as const,
	},
	my_project: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.my-project',
			defaultMessage: 'My project',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.reason.my-project.description',
			defaultMessage: 'Original work created by you.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.reason.my-project.proof-images-description',
			defaultMessage: 'Upload files that help verify you created this work.',
		}),
		proofImagesOptional: true,
		fields: ['license_id', 'custom_license', 'notes', 'image_urls'] as const,
	},
	special_permissions: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.special-permission',
			defaultMessage: 'Special permission',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.reason.special-permission.description',
			defaultMessage:
				'You have obtained special permission to redistribute this work in your modpack.',
		}),
		proofImagesDescription: defineMessage({
			id: 'external-files.permissions-card.reason.proof-description',
			defaultMessage:
				'Include screenshots of messages, emails, or replies from the copyright owner showing that they granted you permission to redistribute their work in your modpack.',
		}),
		proofImagesOptional: false,
		fields: ['link_to_work', 'notes', 'image_urls'] as const,
	},
	no_permission: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.no-permission',
			defaultMessage: 'No permission',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.reason.no-permission.description',
			defaultMessage: "You don't have permission to use this work.",
		}),
		proofImagesDescription: null,
		proofImagesOptional: null,
		fields: ['notes'] as const,
	},
	globally_allowed: {
		label: defineMessage({
			id: 'external-files.permissions-card.reason.globally-allowed',
			defaultMessage: 'Automatically attributed',
		}),
		description: defineMessage({
			id: 'external-files.permissions-card.reason.globally-allowed.description',
			defaultMessage:
				"We've seen this file before and have prepared an attribution for you. If something seems wrong, please contact Modrinth Support via the Help Center",
		}),
		proofImagesDescription: null,
		proofImagesOptional: null,
		fields: ['link_to_work'] as const,
	},
} satisfies Record<
	Labrinth.Attribution.Internal.AttributionResolutionKind,
	{
		label: MessageDescriptor
		description: MessageDescriptor
		proofImagesDescription: MessageDescriptor | null
		proofImagesOptional: boolean | null
		fields: ProjectPermissionField[]
	}
>

export function isHttpUrl(raw: string): boolean {
	const s = raw.trim()
	if (!s) return false
	let parsed: URL
	try {
		parsed = new URL(s)
	} catch {
		return false
	}
	return parsed.protocol === 'http:' || parsed.protocol === 'https:'
}

export function isCustomAttributionLicense(
	license: Labrinth.Attribution.Internal.AttributionLicense,
): license is { name: string } {
	return typeof license === 'object' && license !== null && 'name' in license
}

export function parseAttributionLicense(
	license: Labrinth.Attribution.Internal.AttributionLicense | undefined,
): {
	spdx: string
	custom: string
} {
	if (!license) {
		return { spdx: '', custom: '' }
	}
	if (isCustomAttributionLicense(license)) {
		return { spdx: CUSTOM_LICENSE_VALUE, custom: license.name }
	}
	return { spdx: license, custom: '' }
}

export function attributionLinkToWork(
	attribution: Labrinth.Attribution.Internal.AttributionResolution | null | undefined,
): string | undefined {
	if (!attribution) {
		return undefined
	}
	if (PERMISSION_REASONS[attribution.kind].fields.includes('link_to_work')) {
		return attribution.link_to_work
	}
	return undefined
}

export function parseInitialAttribution(
	raw: unknown,
): Labrinth.Attribution.Internal.AttributionResolution | null {
	if (!raw || typeof raw !== 'object') {
		return null
	}
	const obj = raw as Record<string, unknown>
	const kind = obj.kind
	if (typeof kind !== 'string' || !(permissionKinds as string[]).includes(kind)) {
		return null
	}
	return obj as Labrinth.Attribution.Internal.AttributionResolution
}

const unnamedMultiAttributionGroupTitle = defineMessage({
	id: 'external-files.permissions-card.unnamed-multi-group-title',
	defaultMessage: '{filename} + {count} more',
})

const fallbackAttributionGroupTitle = defineMessage({
	id: 'external-files.permissions-card.fallback-group-title',
	defaultMessage: 'Attribution group {id}',
})

export function createAttributionGroupTitle(
	group: Labrinth.Attribution.Internal.AttributionGroup,
	formatMessage: (descriptor: MessageDescriptor, values?: Record<string, unknown>) => string,
): string {
	const fileCount = group.files?.length ?? 0
	if (group.flame_project?.title) {
		return group.flame_project.title
	}
	const firstFileName = group.files?.[0]?.name ?? group.files?.[0]?.sha1 ?? ''
	if (firstFileName) {
		const base = firstFileName.split('/').pop() ?? firstFileName
		if (fileCount === 1) {
			return base
		}
		return formatMessage(unnamedMultiAttributionGroupTitle, {
			filename: base,
			count: fileCount - 1,
		})
	}
	return formatMessage(fallbackAttributionGroupTitle, { id: group.id })
}

export function moderatorAttributionGroupTitle(
	group: Labrinth.Attribution.Internal.AttributionGroup,
): string {
	const fileCount = group.files?.length ?? 0
	if (group.flame_project?.title) {
		return group.flame_project.title
	}
	const firstFileName = group.files?.[0]?.name ?? group.files?.[0]?.sha1 ?? ''
	if (firstFileName) {
		const base = firstFileName.split('/').pop() ?? firstFileName
		if (fileCount === 1) {
			return base
		}
		return `${base} + ${fileCount - 1} more`
	}
	return `Attribution group ${group.id}`
}

export const MODERATOR_ATTRIBUTION_KIND_LABELS: Record<
	Labrinth.Attribution.Internal.AttributionResolutionKind,
	string
> = {
	license: 'License',
	my_project: 'My project',
	special_permissions: 'Special permission',
	no_permission: 'No permission',
	globally_allowed: 'Automatically attributed',
}

export type ExternalLicenseStatus = Labrinth.ExternalProjects.Internal.ExternalLicenseStatus

export function attributionKindToDefaultExternalStatus(
	kind: Labrinth.Attribution.Internal.AttributionResolutionKind,
): ExternalLicenseStatus | undefined {
	if (kind === 'no_permission') {
		return 'no'
	}
	if (kind === 'license') {
		return 'yes'
	}
	return undefined
}

export function buildExternalLicenseProofFromAttribution(
	attribution: Labrinth.Attribution.Internal.AttributionResolution,
): string {
	const parts: string[] = []
	const notes = attribution.notes?.trim()
	if (notes) {
		parts.push(notes)
	}
	for (const url of attribution.image_urls ?? []) {
		parts.push(`![proof image](${url})`)
	}
	return parts.join('\n\n')
}

export function groupLinkForExternalLicense(
	group: Labrinth.Attribution.Internal.AttributionGroup,
	attribution: Labrinth.Attribution.Internal.AttributionResolution | null,
): string {
	return attributionLinkToWork(attribution) ?? group.flame_project?.url ?? ''
}

export const MODERATION_DB_BADGE: Record<
	Labrinth.ExternalProjects.Internal.ExternalLicenseStatus,
	{
		color?: string
		label: string
	}
> = {
	no: {
		color: 'var(--color-red)',
		label: 'Likely not licensed',
	},
	'permanent-no': {
		color: 'var(--color-purple)',
		label: 'May violate rules',
	},
	yes: {
		color: 'var(--color-green)',
		label: 'Allowed',
	},
	'with-attribution': {
		color: 'var(--color-green)',
		label: 'Allowed',
	},
	'with-attribution-and-source': {
		color: 'var(--color-green)',
		label: 'Allowed',
	},
	unidentified: {
		label: 'Unidentified',
	},
}
