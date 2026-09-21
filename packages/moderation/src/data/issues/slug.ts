import { LinkIcon } from '@modrinth/assets'

import misused from '../messages/checklist/messages/title-slug/slug/misused.md'
import { issue, panel, section, toggle } from './component-builders/builders'

export const misusedSlugIssue = issue({
	id: 'slug-misused',
	message: misused.replace('%CORRECT%', '').trim(),
})

export const slugReviewPanel = panel({
	field: 'slug',
	title: 'Slug',
	hint: "Is the project's URL accurate and appropriate?",
	icon: LinkIcon,
}).content(
	section().content(
		toggle({
			label: 'Misused',
			issue: misusedSlugIssue,
		}),
	),
)
