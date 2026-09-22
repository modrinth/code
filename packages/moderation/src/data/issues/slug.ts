import misused from '../messages/checklist/messages/title-slug/slug/misused.md'
import { issue } from './component-builders/builders'

export const misusedSlugIssue = issue({
	id: 'slug-misused',
	message: misused.replace('%CORRECT%', '').trim(),
})
