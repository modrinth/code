import misused from '../messages/checklist/messages/title-slug/slug/misused.md'
import { issue } from './component-builders/builders'

export const misusedSlugIssue = issue({
	id: 'slug-misused',
	title: 'Misused project URL',
	category: 'Title',
	message: misused.replace('%CORRECT%', '').trim(),
})
