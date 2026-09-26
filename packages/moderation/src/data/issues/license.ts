import { BookTextIcon } from '@modrinth/assets'

import { promptSourceRequired } from '../../utils'
import invalidLinkMessage from '../messages/checklist/messages/license/invalid-link.md'
import invalidLinkCustomLicenseMessage from '../messages/checklist/messages/license/invalid-link/custom-license.md'
import noSourceMessage from '../messages/checklist/messages/license/no-source.md'
import noSourceForkMessage from '../messages/checklist/messages/license/no-source-fork.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const licenseInvalidLinkIssue = issue({
	id: 'license-invalid-link',
	title: 'Invalid license link',
	category: 'License',
	message: ({ selected }) =>
		[
			invalidLinkMessage,
			selected.toggleIds.includes('license-custom-license') ? invalidLinkCustomLicenseMessage : '',
		].join('\n'),
	suggestedStatus: 'flagged',
})

export const licenseNoSourceIssue = issue({
	id: 'license-no-source',
	title: 'Missing source code',
	category: 'License',
	message: ({ selected }) =>
		selected.toggleIds.includes('license-fork') ? noSourceForkMessage : noSourceMessage,
	suggestedStatus: 'rejected',
})

export const licenseReviewPanel = panel({
	title: 'License',
	hint: 'Is this license and link valid?',
	icon: BookTextIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080f8805df7d012a8f770',
}).content(
	toggle({
		issue: licenseInvalidLinkIssue,
		label: 'Invalid Link',
		shown: (ctx) => !ctx.ProjectV3.minecraft_server && !!ctx.ProjectV3.license.url,
	}),
	section({
		shown: (ctx) =>
			!ctx.ProjectV3.minecraft_server &&
			ctx.selected.issueIds.includes(licenseInvalidLinkIssue.id) &&
			!!ctx.ProjectV3.license.url,
	}).content(
		toggle({
			shown: (ctx) => !ctx.ProjectV3.minecraft_server,
			issue: licenseInvalidLinkIssue,
			label: 'Invalid Link: Custom License',
			id: 'license-custom-license',
		}),
	),
	section({
		shown: (ctx) =>
			!ctx.ProjectV3.minecraft_server &&
			promptSourceRequired(ctx.ProjectV3.license.id, ctx.ProjectV3.project_types),
	}).content(
		toggle({
			shown: (ctx) => !ctx.ProjectV3.minecraft_server,
			issue: licenseNoSourceIssue,
			label: 'No Source',
		}),
		section({
			shown: (ctx) =>
				!ctx.ProjectV3.minecraft_server && ctx.selected.issueIds.includes(licenseNoSourceIssue.id),
		}).content(
			toggle({
				shown: (ctx) => !ctx.ProjectV3.minecraft_server,
				issue: licenseNoSourceIssue,
				label: 'No Source: Fork',
				id: 'license-fork',
			}),
		),
	),
)
