import { AbstractModule } from '../../../core/abstract-module'
import type { Kyros } from '../types'

export class KyrosFilesV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_files_v1'
	}

	/**
	 * Get metadata for a path in world storage.
	 */
	public async stat(
		worldId: string,
		data: Kyros.Files.v1.FileStatRequest,
	): Promise<Kyros.Files.v1.FileStatResponse> {
		return this.client.request<Kyros.Files.v1.FileStatResponse>(`/worlds/${worldId}/files/stat`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: data,
			useNodeAuth: true,
		})
	}

	/**
	 * Create a ZIP archive beside a directory in world storage.
	 */
	public async createZip(
		worldId: string,
		data: Kyros.Files.v1.ZipRequest,
		onProgress?: (record: Kyros.Files.v1.ZipProgress) => void,
	): Promise<void> {
		const stream = await this.client.stream(`/worlds/${worldId}/files/zip`, {
			api: '',
			version: 'v1',
			method: 'POST',
			body: data,
			useNodeAuth: true,
			headers: { Accept: 'application/json-seq' },
		})
		const reader = stream.getReader()
		const decoder = new TextDecoder()
		let buffer = ''
		let completed = false

		const parseRecord = (value: string) => {
			const trimmedValue = value.trim()
			const text = trimmedValue.startsWith('\u001e') ? trimmedValue.slice(1).trim() : trimmedValue
			if (!text) return
			const record = JSON.parse(text) as Kyros.Files.v1.ZipProgress
			onProgress?.(record)
			if (record.error) throw new Error(record.error)
			if (record.done === true) completed = true
		}

		const parseRecords = (flush = false) => {
			let newlineIndex = buffer.indexOf('\n')
			while (newlineIndex !== -1) {
				parseRecord(buffer.slice(0, newlineIndex))
				buffer = buffer.slice(newlineIndex + 1)
				newlineIndex = buffer.indexOf('\n')
			}
			if (flush && buffer.trim()) {
				parseRecord(buffer)
				buffer = ''
			}
		}

		try {
			while (!completed) {
				const { done, value } = await reader.read()
				if (done) break
				buffer += decoder.decode(value, { stream: true })
				parseRecords()
			}
			if (completed) {
				await reader.cancel().catch(() => undefined)
				return
			}
			buffer += decoder.decode()
			parseRecords(true)
			if (!completed) throw new Error('ZIP operation ended before completion')
		} finally {
			reader.releaseLock()
		}
	}
}
