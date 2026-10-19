import { SignatureIcon } from '@modrinth/assets'

import invalidPermissionsMessage from '../messages/checklist/messages/permissions/invalid-permissions.md'
import missingPermissionsMessage from '../messages/checklist/messages/permissions/missing-permissions.md'
import nonCommercialExternalContentMessage from '../messages/checklist/messages/permissions/non-commercial-external-content.md'
import prohibitedExternalContentMessage from '../messages/checklist/messages/permissions/prohibited-external-content.md'
import { issue, panel, toggle } from './component-builders/builders'
import { issueLocation } from './component-builders/locations'

export const permissionsInvalidPermissionsIssue = issue({
	id: 'permissions-invalid-permissions',
	locations: [issueLocation('permissions')],
	title: 'Invalid permissions',
	category: 'Permissions',
	message: invalidPermissionsMessage,
	suggestedStatus: 'rejected',
})

export const permissionsProhibitedExternalContentIssue = issue({
	id: 'permissions-prohibited-external-content',
	locations: [issueLocation('permissions')],
	title: 'Prohibited external content',
	category: 'Permissions',
	message: prohibitedExternalContentMessage,
	suggestedStatus: 'rejected',
})

export const permissionsMissingPermissionsIssue = issue({
	id: 'permissions-missing-permissions',
	locations: [issueLocation('permissions')],
	title: 'Missing permissions',
	category: 'Permissions',
	message: missingPermissionsMessage,
	suggestedStatus: 'rejected',
})

export const permissionsNonCommercialExternalContentIssue = issue({
	id: 'permissions-non-commercial-external-content',
	locations: [issueLocation('permissions')],
	title: 'Non-commercial external content',
	category: 'Permissions',
	message: nonCommercialExternalContentMessage,
	suggestedStatus: 'rejected',
})

export const permissionsReviewPanel = panel({
	title: 'Modpack Permissions',
	hint: "Does this project's external content have any issues?",
	icon: SignatureIcon,
	guidanceUrl: 'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892',
	shown: ({ projectV3, permissions }) =>
		projectV3.project_types.includes('modpack') &&
		!projectV3.minecraft_server &&
		permissions.loaded &&
		permissions.unresolvedCount > 0,
}).content(
	toggle({
		issue: permissionsInvalidPermissionsIssue,
		label: 'Invalid permissions',
	}),
	toggle({
		issue: permissionsProhibitedExternalContentIssue,
		label: 'Prohibited externals',
	}),
	toggle({
		issue: permissionsMissingPermissionsIssue,
		label: 'Missing permissions',
	}),
	toggle({
		issue: permissionsNonCommercialExternalContentIssue,
		label: 'Non-commercial externals',
		shown: ({ projectV3 }) => projectV3.monetization_status === 'monetized',
	}),
)
