import type {FileInfo} from "#ui/layouts/shared/files-tab/providers/file-manager.ts";

export function joinDisplayPath(basePath: string | undefined, itemPath: string) {
	if (!basePath) return itemPath

	const separator = basePath.includes('\\') ? '\\' : '/'
	const path = itemPath.replace(/^[\\/]+/, '').replace(/[\\/]+/g, separator)
	const base = basePath.replace(/[\\/]+$/, '')

	return path ? `${base}${separator}${path}` : basePath
}

export function parentInfoFrom(target: string | FileInfo) {
	return infoFrom(parentDirectory(typeof target === 'string' ? target : target.path));
}

export function infoFrom<BL extends boolean = false>(path: string, isFile?: BL): FileInfo<BL extends true ? 'file' : 'directory'> {
	const pathParts = normalizeDirectoryPath(path).split('/');
	const name = pathParts[pathParts.length - 1] ?? '/';
	return {
		type: isFile ? 'file' : 'directory',
		path: path,
		name: name
	} as FileInfo<BL extends true ? 'file' : 'directory'>;
}

export function normalizeDirectoryPath(path: string) {
	return `/${path.split('/').filter(Boolean).join('/')}`
}

export function parentDirectory(path: string) {
	const segments = path.split('/').filter(Boolean)
	segments.pop()
	return `/${segments.join('/')}`
}

export function isSameInfo(a: FileInfo, b: FileInfo) {
	return a.type === b.type && a.path === b.path
}
