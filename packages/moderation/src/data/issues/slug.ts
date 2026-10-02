import { LinkIcon } from '@modrinth/assets'

import { generateUrlSlug } from '../../utils'
import misused from '../messages/checklist/messages/title-slug/slug/misused.md'
import { issue, panel, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

export const misusedSlugIssue = issue({
	id: 'slug-misused',
	title: 'Misused project URL',
	category: 'Slug',
	facets: [issueTargets.modifySlug()],
	message: misused.replace('%CORRECT%', '').trim(),
})

export const slugReviewPanel = panel({
	title: 'Slug',
	hint: "Is the project's URL accurate and appropriate?",
	icon: LinkIcon,
	guidanceUrl:
		'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
}).content(
	toggle({
		label: 'Misused slug',
		issue: misusedSlugIssue,
		shown: ({ projectV3 }) => generateUrlSlug(projectV3.name) !== projectV3.slug,
	}),
)
