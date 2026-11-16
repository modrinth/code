import JSZip from 'jszip'

export type DownloadableFile = {
	href: string
	filename: string
}

type DownloadedFile = DownloadableFile & {
	blob: Blob
}

const DOWNLOAD_URL_REVOKE_MS = 60000
const DOWNLOAD_STAGGER_MS = 500

export function dedupeDownloadFiles(files: DownloadableFile[]) {
	const result: DownloadableFile[] = []
	const hrefs = new Set<string>()

	for (const file of files) {
		if (hrefs.has(file.href)) continue
		hrefs.add(file.href)
		result.push(file)
	}

	return result
}

export async function downloadFiles(files: DownloadableFile[]) {
	await Promise.all(
		files.map(async (file, index) => {
			await delay(DOWNLOAD_STAGGER_MS * index)
			downloadFileLink(file)
		}),
	)
}

export async function downloadFilesAsZip(files: DownloadableFile[], zipFilename: string) {
	const zip = new JSZip()
	const usedFilenames = new Set<string>()
	const downloadedFiles = await Promise.all(files.map((file) => downloadFileBlob(file)))

	for (const file of downloadedFiles) {
		zip.file(uniqueFilename(file.filename, usedFilenames), file.blob)
	}

	downloadBlob(
		await zip.generateAsync({
			type: 'blob',
			mimeType: 'application/zip',
		}),
		zipFilename,
	)
}

export function versionZipFilename(projectTitle: string, versionNumber: string) {
	return `${sanitizeFilename(projectTitle)} ${sanitizeFilename(versionNumber)}-EXTRACT_ME.zip`
}

async function downloadFileBlob(file: DownloadableFile): Promise<DownloadedFile> {
	const response = await fetch(file.href)

	if (!response.ok) {
		throw new Error(`Failed to download ${file.filename}`)
	}

	return {
		...file,
		blob: await response.blob(),
	}
}

function downloadBlob(blob: Blob, filename: string) {
	const url = URL.createObjectURL(blob)
	const link = document.createElement('a')

	link.href = url
	link.download = filename
	document.body.appendChild(link)
	link.click()
	link.remove()
	window.setTimeout(() => URL.revokeObjectURL(url), DOWNLOAD_URL_REVOKE_MS)
}

function downloadFileLink(file: DownloadableFile) {
	const link = document.createElement('a')
	link.href = file.href
	link.download = file.filename
	document.body.appendChild(link)
	link.click()
	link.remove()
}

function sanitizeFilename(value: string) {
	const sanitized = value
		.replace(/[<>:"/\\|?*]/g, '')
		.replace(/\s+/g, ' ')
		.trim()

	return sanitized || 'download'
}

function uniqueFilename(filename: string, usedFilenames: Set<string>) {
	const sanitizedFilename = sanitizeFilename(filename)

	if (!usedFilenames.has(sanitizedFilename)) {
		usedFilenames.add(sanitizedFilename)
		return sanitizedFilename
	}

	const extensionIndex = sanitizedFilename.lastIndexOf('.')
	const basename =
		extensionIndex > 0 ? sanitizedFilename.slice(0, extensionIndex) : sanitizedFilename
	const extension = extensionIndex > 0 ? sanitizedFilename.slice(extensionIndex) : ''
	let index = 2
	let candidate = `${basename} (${index})${extension}`

	while (usedFilenames.has(candidate)) {
		index += 1
		candidate = `${basename} (${index})${extension}`
	}

	usedFilenames.add(candidate)
	return candidate
}

async function delay(ms: number) {
	await new Promise((resolve) => setTimeout(resolve, ms))
}
