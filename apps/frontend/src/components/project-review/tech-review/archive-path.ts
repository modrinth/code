import { slicerFileName } from '~/providers/project-review/review-slicer'

import type { ClassGroup } from './types'

function trimSlashes(path: string): string {
	if (path.startsWith('/')) {
		path = path.slice(1)
	}
	if (path.endsWith('/')) {
		path = path.slice(0, -1)
	}
	return path
}

/**
 * Resolves the slicer workspace entry for a class group, removing trailing and leading slashes and
 * assuming `.class` when the path has no file extension.
 */
export function slicerEntry(url: string, group: ClassGroup) {
	const jarPath = trimSlashes(group.jar ? group.jar.replace('#', '/') : slicerFileName(url))
	const filePath = trimSlashes(group.filePath)
	const hasFileExtension = (filePath.split('/').pop() || '').includes('.')
	return `${jarPath}/${hasFileExtension ? filePath : `${filePath}.class`}`
}
