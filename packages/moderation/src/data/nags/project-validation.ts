import { defineMessage, useVIntl } from '@modrinth/ui'

import type { Nag, NagContext } from '../../types/nags'
import type { ProjectTextValidationCode } from '../../validators/project-fields/index.ts'
import type {
	ProjectValidationFailure,
	ProjectValidationField,
} from '../../validators/project-validation/index.ts'

const generalSettingsRoutes = new Set(['type-project-settings', 'type-project-settings-general'])

const nameErrorCodes: readonly ProjectTextValidationCode[] = [
	'text-slur',
	'text-profanity',
	'text-non-standard',
]
const summaryErrorCodes: readonly ProjectTextValidationCode[] = [
	'text-slur',
	'text-profanity',
	'text-non-standard',
]
const summaryWarningCodes: readonly ProjectTextValidationCode[] = [
	'summary-link',
	'summary-matches-title',
]
const descriptionErrorCodes: readonly ProjectTextValidationCode[] = [
	'text-slur',
	'text-profanity',
	'text-non-standard',
]

function getFirstFailure(
	context: NagContext,
	fields: readonly ProjectValidationField[],
	severity?: ProjectValidationFailure['severity'],
	codes?: readonly ProjectTextValidationCode[],
): ProjectValidationFailure | undefined {
	return context.projectValidation.failures.find(
		(failure) =>
			fields.includes(failure.field) &&
			(!severity || failure.severity === severity) &&
			(!codes || codes.includes(failure.code)),
	)
}

function getFailureDescription(
	context: NagContext,
	fields: readonly ProjectValidationField[],
	severity?: ProjectValidationFailure['severity'],
	codes?: readonly ProjectTextValidationCode[],
): string {
	const failure = getFirstFailure(context, fields, severity, codes)
	if (!failure) return ''

	const { formatMessage } = useVIntl()
	return formatMessage(failure.message, failure.values)
}

function getCodedFailure(context: NagContext, code: ProjectTextValidationCode) {
	return context.projectValidation.failures.find((failure) => failure.code === code)
}

function getCodedFailureDescription(context: NagContext, code: ProjectTextValidationCode) {
	const failure = getCodedFailure(context, code)
	if (!failure) return ''

	const { formatMessage } = useVIntl()
	return formatMessage(failure.message, failure.values)
}

export const projectValidationNags: Nag[] = [
	{
		id: 'invalid-project-name',
		title: defineMessage({
			id: 'nags.invalid-project-name.title',
			defaultMessage: 'Fix the project name',
		}),
		description: (context) => getFailureDescription(context, ['name'], 'error', nameErrorCodes),
		status: 'required',
		shouldShow: (context) =>
			getFirstFailure(context, ['name'], 'error', nameErrorCodes) !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-title.title',
				defaultMessage: 'Edit title',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'project-name-version',
		title: defineMessage({
			id: 'nags.project-name-version.title',
			defaultMessage: 'Fix project name',
		}),
		description: (context) => getCodedFailureDescription(context, 'title-version-number'),
		status: 'required',
		shouldShow: (context) => getCodedFailure(context, 'title-version-number') !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-title.title',
				defaultMessage: 'Edit title',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'minecraft-title-clause',
		title: defineMessage({
			id: 'nags.minecraft-title-clause.title',
			defaultMessage: 'Avoid brand infringement',
		}),
		description: (context) => getCodedFailureDescription(context, 'title-minecraft-branding'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'title-minecraft-branding') !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-title.title',
				defaultMessage: 'Edit title',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'invalid-project-summary',
		title: defineMessage({
			id: 'nags.invalid-project-summary.title',
			defaultMessage: 'Fix the project summary',
		}),
		description: (context) =>
			getFailureDescription(context, ['summary'], 'error', summaryErrorCodes),
		status: 'required',
		shouldShow: (context) =>
			getFirstFailure(context, ['summary'], 'error', summaryErrorCodes) !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'project-summary-content',
		title: defineMessage({
			id: 'nags.project-summary-content.title',
			defaultMessage: 'Review the project summary',
		}),
		description: (context) =>
			getFailureDescription(context, ['summary'], 'warn', summaryWarningCodes),
		status: 'warning',
		shouldShow: (context) =>
			getFirstFailure(context, ['summary'], 'warn', summaryWarningCodes) !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'summary-too-short',
		title: defineMessage({
			id: 'nags.summary-too-short.title',
			defaultMessage: 'Expand the summary',
		}),
		description: (context) => getCodedFailureDescription(context, 'summary-too-short'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'summary-too-short') !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'summary-special-formatting',
		title: defineMessage({
			id: 'nags.summary-special-formatting.title',
			defaultMessage: 'Clean up the summary',
		}),
		description: (context) => getCodedFailureDescription(context, 'summary-special-formatting'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'summary-special-formatting') !== undefined,
		link: {
			path: 'settings',
			title: defineMessage({
				id: 'nags.edit-summary.title',
				defaultMessage: 'Edit summary',
			}),
			shouldShow: (context) => !generalSettingsRoutes.has(context.currentRoute),
		},
	},
	{
		id: 'add-description',
		title: defineMessage({
			id: 'nags.add-description.title',
			defaultMessage: 'Add a description',
		}),
		description: (context) => getCodedFailureDescription(context, 'description-required'),
		status: 'required',
		shouldShow: (context) => getCodedFailure(context, 'description-required') !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.settings.description.title',
				defaultMessage: 'Visit description settings',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'invalid-project-description',
		title: defineMessage({
			id: 'nags.invalid-project-description.title',
			defaultMessage: 'Fix the project description',
		}),
		description: (context) =>
			getFailureDescription(context, ['description'], 'error', descriptionErrorCodes),
		status: 'required',
		shouldShow: (context) =>
			getFirstFailure(context, ['description'], 'error', descriptionErrorCodes) !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'description-too-short',
		title: defineMessage({
			id: 'nags.description-too-short.title',
			defaultMessage: 'Expand the description',
		}),
		description: (context) => getCodedFailureDescription(context, 'description-too-short'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'description-too-short') !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'long-headers',
		title: defineMessage({
			id: 'nags.long-headers.title',
			defaultMessage: 'Shorten headers',
		}),
		description: (context) => getCodedFailureDescription(context, 'description-long-headers'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'description-long-headers') !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'image-heavy-description',
		title: defineMessage({
			id: 'nags.image-heavy-description.title',
			defaultMessage: 'Ensure accessibility',
		}),
		description: (context) => getCodedFailureDescription(context, 'description-image-heavy'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'description-image-heavy') !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'missing-alt-text',
		title: defineMessage({
			id: 'nags.missing-alt-text.title',
			defaultMessage: 'Add image alt text',
		}),
		description: (context) => getCodedFailureDescription(context, 'description-missing-alt-text'),
		status: 'warning',
		shouldShow: (context) => getCodedFailure(context, 'description-missing-alt-text') !== undefined,
		link: {
			path: 'settings/description',
			title: defineMessage({
				id: 'nags.edit-description.title',
				defaultMessage: 'Edit description',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-description',
		},
	},
	{
		id: 'invalid-gallery-text',
		title: defineMessage({
			id: 'nags.invalid-gallery-text.title',
			defaultMessage: 'Fix gallery text',
		}),
		description: (context) =>
			getFailureDescription(context, ['gallery-name', 'gallery-description'], 'error'),
		status: 'required',
		shouldShow: (context) =>
			getFirstFailure(context, ['gallery-name', 'gallery-description'], 'error') !== undefined,
		link: {
			path: 'settings/gallery',
			title: defineMessage({
				id: 'nags.edit-gallery.title',
				defaultMessage: 'Edit gallery',
			}),
			shouldShow: (context) => context.currentRoute !== 'type-project-settings-gallery',
		},
	},
]
