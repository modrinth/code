import { BookTextIcon } from '@modrinth/assets'

import { promptSourceRequired } from '../../utils'
import invalidLinkMessage from '../messages/checklist/messages/license/invalid-link.md'
import invalidLinkCustomLicenseMessage from '../messages/checklist/messages/license/invalid-link/custom-license.md'
import noSourceMessage from '../messages/checklist/messages/license/no-source.md'
import noSourceForkMessage from '../messages/checklist/messages/license/no-source-fork.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const licenseInvalidLinkIssue = issue({
	id: 'license-invalid-link',
	message: ({ selected }) =>
		[
			invalidLinkMessage,
			selected.toggleIds.includes('license-custom-license') ? invalidLinkCustomLicenseMessage : '',
		].join('\n'),
	suggestedStatus: 'flagged',
})

export const licenseNoSourceIssue = issue({
	id: 'license-no-source',
	message: ({ selected }) =>
		selected.toggleIds.includes('license-fork') ? noSourceForkMessage : noSourceMessage,
	suggestedStatus: 'rejected',
})

export const licenseReviewPanel = panel({
	field: 'license',
	title: 'License',
	hint: 'Is this license and link valid?',
	icon: BookTextIcon,
	shown: ({ ProjectV3 }) => !ProjectV3.minecraft_server,
}).content(
	toggle({
		issue: licenseInvalidLinkIssue,
		label: 'Invalid Link',
		shown: ({ ProjectV3 }) => !!ProjectV3.license.url,
	}),
	section({
		shown: (ctx) =>
			ctx.selected.issueIds.includes(licenseInvalidLinkIssue.id) && !!ctx.ProjectV3.license.url,
	}).content(
		toggle({
			issue: licenseInvalidLinkIssue,
			label: 'Invalid Link: Custom License',
			id: 'license-custom-license',
		}),
	),
	section({
		shown: ({ ProjectV3 }) => promptSourceRequired(ProjectV3.license.id, ProjectV3.project_types),
	}).content(
		toggle({ issue: licenseNoSourceIssue, label: 'No Source' }),
		section({ shown: (ctx) => ctx.selected.issueIds.includes(licenseNoSourceIssue.id) }).content(
			toggle({ issue: licenseNoSourceIssue, label: 'No Source: Fork', id: 'license-fork' }),
		),
	),
)
