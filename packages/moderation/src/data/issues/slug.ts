import { LinkIcon } from '@modrinth/assets'

import { messageWithOptionalSuggestion } from '../../utils'
import misused from '../messages/checklist/messages/title-slug/slug/misused.md'
import { issue, panel, section, text, toggle } from './component-builders/builders'
import { issueTargets } from './component-builders/targets'

const panelCategory = 'Slug'

export const misusedSlugIssue = issue({
	id: 'slug-misused',
	title: 'Misused project URL',
	category: 'Slug',
	suggestedStatus: 'flagged',
	actions: [
		issueTargets.modifySlug(({ getTextValue }) => {
			const suggestion = getTextValue('correct-slug').toLowerCase()
			return suggestion.trim() ? suggestion : undefined
		}),
	],
	message: ({ getTextValue }) =>
		messageWithOptionalSuggestion(misused, panelCategory, getTextValue('correct-slug')),
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
	}),
	section({
		shown: ({ selected }) => selected.issueIds.includes(misusedSlugIssue.id),
	}).content(
		text({
			issue: misusedSlugIssue,
			id: 'correct-slug',
			label: 'Correct project URL',
		}),
	),
)
