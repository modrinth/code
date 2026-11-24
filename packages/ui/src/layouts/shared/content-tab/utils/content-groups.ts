import type { ContentItem } from '../types'

export const UNGROUPED_CONTENT_SECTION_ID = 'ungrouped'

export function getContentGroupItemKey(item: Pick<ContentItem, 'project' | 'file_name'>) {
	if (item.project?.id) return `project:${item.project.id}`
	return `file:${item.file_name.replace(/\.disabled$/i, '')}`
}
