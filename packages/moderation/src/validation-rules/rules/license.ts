import { defineMessages } from '@modrinth/ui/i18n'
import { formatProjectTypeSentence } from '@modrinth/ui/src/utils/common-messages.ts'

import type { Nag, ProjectValidationContext } from '../../types/nags.ts'
import { getLinkHostname, isInappropriateLicenseLink } from '../../validators/links/index.ts'
import { evaluateRules } from '../evaluate-rules.ts'
import { toNags } from '../to-nags.ts'
import type { ValidationRuleSet } from '../types.ts'

const messages = defineMessages({
	selectLicense: {
		id: 'nags.select-license.title',
		defaultMessage: `Select a license`,
	},
	selectLicenseDescription: {
		id: 'nags.select-license.description',
		defaultMessage: `Select the license your {type} is distributed under.`,
	},
	addDetails: {
		id: 'nags.add-license-details.title',
		defaultMessage: `Add license details`,
	},
	addDetailsDescription: {
		id: 'nags.add-license-details.description',
		defaultMessage: `Add a valid URL and name or SPDX identifier for your custom license.`,
	},
	invalidUrl: {
		id: 'nags.invalid-license-url.title',
		defaultMessage: `Add a valid license link`,
	},
	invalidUrlDefault: {
		id: 'nags.invalid-license-url.description.default',
		defaultMessage: `License URL is invalid.`,
	},
	invalidUrlDomain: {
		id: 'nags.invalid-license-url.description.domain',
		defaultMessage: `Your license URL points to {domain}, which is not appropriate for license information. License URLs should link directly to your license text.`,
	},
	invalidUrlMalformed: {
		id: 'nags.invalid-license-url.description.malformed',
		defaultMessage: `Your license URL appears to be malformed. Please provide a valid URL to your license text.`,
	},
	editLicense: {
		id: 'nags.edit-license.title',
		defaultMessage: `Edit license`,
	},
})

export const projectLicenseValidationRules = {
	'select-license': {
		severity: 'error',
		evaluate: (context) => {
			const licenseId = context.project.license.id
			const unknown =
				licenseId === 'LicenseRef-Unknown' ||
				licenseId === 'NOASSERTION' ||
				licenseId === 'LicenseRef-NOASSERTION'
			return unknown && !context.projectV3.minecraft_server
				? { valid: false, values: { projectType: context.project.project_type } }
				: { valid: true }
		},
		presentation: {
			message: messages.selectLicenseDescription,
			nag: {
				title: messages.selectLicense,
				destination: 'license',
				formatValues: (values, formatMessage) => ({
					type: formatProjectTypeSentence(formatMessage, String(values.projectType)),
				}),
			},
		},
	},
	'add-custom-license-details': {
		severity: 'error',
		evaluate: (context) => {
			const license = context.project.license
			const missingDetails =
				license.id === 'LicenseRef-' ||
				(license.id.startsWith('LicenseRef-') &&
					!license.url &&
					license.id !== 'LicenseRef-Unknown' &&
					license.id !== 'LicenseRef-All-Rights-Reserved')
			return { valid: Boolean(context.projectV3.minecraft_server) || !missingDetails }
		},
		presentation: {
			message: messages.addDetailsDescription,
			nag: { title: messages.addDetails, destination: 'license' },
		},
	},
	'invalid-license-url': {
		severity: 'error',
		evaluate: (context) => {
			const licenseUrl = context.project.license.url
			if (!licenseUrl) return { valid: true }

			const domain = getLinkHostname(licenseUrl)
			if (domain && isInappropriateLicenseLink(licenseUrl)) {
				return {
					valid: false,
					message: messages.invalidUrlDomain,
					values: { domain },
				}
			}
			if (!domain) return { valid: false, message: messages.invalidUrlMalformed }
			return { valid: true }
		},
		presentation: {
			message: messages.invalidUrlDefault,
			nag: {
				title: messages.invalidUrl,
				destination: 'license',
				linkTitle: messages.editLicense,
			},
		},
	},
} satisfies ValidationRuleSet<ProjectValidationContext>

export function getLicenseNags(context: ProjectValidationContext): Nag[] {
	return toNags(evaluateRules(context, projectLicenseValidationRules))
}
