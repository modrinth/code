// File extension constants
import { type FileTypeInfo, getFileTypeInfo, getLanguageFromFile } from '#ui/helpers/toolbox/file'

export const FILE_CODE_EXTENSIONS = [
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

export const FILE_TEXT_EXTENSIONS = [
	'txt',
	'md',
	'log',
	'cfg',
	'conf',
	'properties',
	'ini',
	'sk',
] as const

export const FILE_IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'] as const

export const FILE_ARCHIVE_EXTENSIONS = ['zip', 'jar', 'tar', 'gz', 'rar', '7z'] as const

export const FILE_VIDEO_EXTENSIONS = ['mp4', 'webm', 'ogv', 'mov', 'm4v'] as const

/** `ogg` counts as audio, as games mostly use it for sounds. */
export const FILE_AUDIO_EXTENSIONS = [
	'mp3',
	'ogg',
	'oga',
	'wav',
	'flac',
	'm4a',
	'aac',
	'opus',
] as const

// Type for extension strings
export type CodeExtension = (typeof FILE_CODE_EXTENSIONS)[number]
export type TextExtension = (typeof FILE_TEXT_EXTENSIONS)[number]
export type ImageExtension = (typeof FILE_IMAGE_EXTENSIONS)[number]
export type ArchiveExtension = (typeof FILE_ARCHIVE_EXTENSIONS)[number]

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
	return (FILE_CODE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if extension is a text file
 */
export function isTextFile(ext: string): boolean {
	return (FILE_TEXT_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if extension is an image file
 */
export function isImageFile(ext: string): boolean {
	return (FILE_IMAGE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/** Check if extension is a video file */
export function isVideoFile(ext: string): boolean {
	return (FILE_VIDEO_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/** Check if extension is an audio file */
export function isAudioFile(ext: string): boolean {
	return (FILE_AUDIO_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/** How a file plays, from its type when known (e.g. detected from its contents) or else its name. */
export function mediaKindOf(
	filename: string,
	fileInfo: FileTypeInfo | null = getFileTypeInfo(filename),
): 'video' | 'audio' | null {
	const ext = getFileExtension(filename)
	if (isAudioFile(ext) || fileInfo?.mime.startsWith('audio/')) return 'audio'
	if (isVideoFile(ext) || fileInfo?.mime.startsWith('video/')) return 'video'
	return null
}

/**
 * Check if extension is an archive file
 */
export function isArchiveFile(ext: string): boolean {
	return (FILE_ARCHIVE_EXTENSIONS as readonly string[]).includes(ext.toLowerCase())
}

/**
 * Check if file is editable (code or text)
 */
export function isEditableFile(ext: string): boolean {
	return isCodeFile(ext) || isTextFile(ext)
}

/**
 * Check if a file can be opened in the file editor, from its type when known (e.g. detected from
 * its contents) or else from its name. A type detected from its contents as neither text, code nor
 * an image can't be opened, whatever its extension suggests.
 */
export function canOpenInFileEditor(
	filename: string,
	fileInfo: FileTypeInfo | null = getFileTypeInfo(filename),
): boolean {
	const lang = getLanguageFromFile(filename, fileInfo)
	if (lang != null) return true
	if (mediaKindOf(filename, fileInfo)) return true
	if (fileInfo != null) {
		if (fileInfo.mime.startsWith('text/') || isImageFile(fileInfo.extension ?? '')) {
			return true
		}
		if (fileInfo.method === 'content-analysis') return false
	}
	const ext = getFileExtension(filename)
	return !filename.includes('.') || isEditableFile(ext) || isImageFile(ext)
}

/** Check if a file can be opened in the file editor, detecting its type from its contents. */
export async function canOpenDataInFileEditor(
	filename: string,
	data: ArrayBuffer | Blob,
): Promise<boolean> {
	return canOpenInFileEditor(filename, await getFileTypeInfo(filename, data))
}

/**
 * Minecraft's own formats, which the editor highlights by modes of its own. They take precedence
 * over the type a file is detected as, which only knows them as plain text.
 */
const MINECRAFT_EDITOR_LANGUAGES: Record<string, string> = {
	log: 'mclog',
	mcfunction: 'mcfunction',
}

/**
 * Get the editor language mode for a file, from its type when known (e.g. detected from its
 * contents) or else from its name.
 */
export function getEditorLanguage(
	filename: string,
	fileInfo: FileTypeInfo | null = getFileTypeInfo(filename),
): string {
	const minecraftLanguage = MINECRAFT_EDITOR_LANGUAGES[getFileExtension(filename)]
	if (minecraftLanguage) return minecraftLanguage

	const language = getLanguageFromFile(filename, fileInfo)
	if (language) return language

	const lowered = getFileExtension(filename)

	switch (lowered) {
		// Code files
		case 'json':
		case 'json5':
		case 'jsonc':
			return 'json'
		case 'toml':
			return 'toml'
		case 'sh':
			return 'sh'
		case 'bat':
			return 'batchfile'
		case 'ps1':
			return 'powershell'
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
			return 'c_cpp'
		case 'rs':
			return 'rust'
		case 'go':
			return 'golang'
		// Text files
		case 'md':
			return 'markdown'
		case 'properties':
			return 'properties'
		case 'ini':
		case 'cfg':
		case 'conf':
			return 'ini'
		default:
			return 'text'
	}
}
