// File extension constants
export const CODE_EXTENSIONS = [
	'json',
	'json5',
	'jsonc',
	'java',
	'kt',
	'kts',
	'sh',
	'bat',
	'ps1',
	'yml',
	'yaml',
	'toml',
	'js',
	'ts',
	'py',
	'rb',
	'php',
	'html',
	'css',
	'cpp',
	'c',
	'h',
	'rs',
	'go',
] as const

export const TEXT_EXTENSIONS = [
	'txt',
	'md',
	'log',
	'cfg',
	'conf',
	'properties',
	'ini',
	'sk',
] as const

export const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'] as const

export const ARCHIVE_EXTENSIONS = ['zip'] as const

// Type for extension strings
export type CodeExtension = (typeof CODE_EXTENSIONS)[number]
export type TextExtension = (typeof TEXT_EXTENSIONS)[number]
export type ImageExtension = (typeof IMAGE_EXTENSIONS)[number]
export type ArchiveExtension = (typeof ARCHIVE_EXTENSIONS)[number]

/**
 * Extract file extension from filename (lowercase)
 */
export function getFileExtension(filename: string): string {
	return filename.split('.').pop()?.toLowerCase() ?? ''
}

/**
 * Check if extension is a code file
 */
export function isCodeFile(ext: string): boolean {
	return (CODE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if extension is a text file
 */
export function isTextFile(ext: string): boolean {
	return (TEXT_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if extension is an image file
 */
export function isImageFile(ext: string): boolean {
	return (IMAGE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if extension is an archive file
 */
export function isArchiveFile(ext: string): boolean {
	return (ARCHIVE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if file is editable (code or text)
 */
export function isEditableFile(ext: string): boolean {
	return isCodeFile(ext) || isTextFile(ext)
}

/**
 * Get Monaco editor language identifier for a file extension
 */
export function getEditorLanguage(ext: string): string {
	const lowered = ext.toLowerCase()
	switch (lowered) {
		case 'json':
		case 'json5':
		case 'jsonc':
			return 'json'
		case 'toml':
			return 'toml'
		case 'sh':
		case 'bat':
		case 'ps1':
			return 'shell'
		case 'yml':
		case 'yaml':
			return 'yaml'
		case 'js':
			return 'javascript'
		case 'ts':
			return 'typescript'
		case 'py':
			return 'python'
		case 'rb':
			return 'ruby'
		case 'php':
			return 'php'
		case 'html':
			return 'html'
		case 'css':
			return 'css'
		case 'java':
		case 'kt':
		case 'kts':
			return 'java'
		case 'cpp':
		case 'c':
		case 'h':
			return 'cpp'
		case 'rs':
			return 'rust'
		case 'go':
			return 'go'
		case 'md':
			return 'markdown'
		default:
			return 'plaintext'
	}
}
