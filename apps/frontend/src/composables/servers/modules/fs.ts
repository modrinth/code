import type {
	DirectoryResponse,
	FilesystemOp,
	FileUploadQuery,
	FSQueuedOp,
	JWTAuth,
} from '@modrinth/utils'
import { ModrinthServerError } from '@modrinth/utils'

import { useServersFetch } from '../servers-fetch.ts'
import { ServerModule } from './base.ts'

export class FSModule extends ServerModule {
	auth!: JWTAuth
	ops: FilesystemOp[] = []
	queuedOps: FSQueuedOp[] = []
	opsQueuedForModification: string[] = []

	async fetch(): Promise<void> {
		this.auth = await useServersFetch<JWTAuth>(`servers/${this.serverId}/fs`, {}, 'fs')
		this.ops = []
		this.queuedOps = []
		this.opsQueuedForModification = []
	}

	private async retryWithAuth<T>(
		requestFn: () => Promise<T>,
		ignoreFailure: boolean = false,
	): Promise<T> {
		try {
			return await requestFn()
		} catch (error) {
			if (error instanceof ModrinthServerError && error.statusCode === 401) {
				console.debug('Auth failed, refreshing JWT and retrying')
				await this.fetch() // Refresh auth
				return await requestFn()
			}

			const available = await this.server.testNodeReachability()
			if (!available && !ignoreFailure) {
				this.server.moduleErrors.general = {
					error: new ModrinthServerError(
						'Unable to reach node. FS operation failed and subsequent ping test failed.',
						500,
						error as Error,
						'fs',
					),
					timestamp: Date.now(),
				}
			}

			throw error
		}
	}

	listDirContents(
		path: string,
		page: number,
		pageSize: number,
		ignoreFailure: boolean = false,
	): Promise<DirectoryResponse> {
		return this.retryWithAuth(async () => {
			const encodedPath = encodeURIComponent(path)
			return await useServersFetch(`/list?path=${encodedPath}&page=${page}&page_size=${pageSize}`, {
				override: this.auth,
				retry: false,
			})
		}, ignoreFailure)
	}

	createFileOrFolder(path: string, type: 'file' | 'directory'): Promise<void> {
		return this.retryWithAuth(async () => {
			const encodedPath = encodeURIComponent(path)
			await useServersFetch(`/create?path=${encodedPath}&type=${type}`, {
				method: 'POST',
				contentType: 'application/octet-stream',
				override: this.auth,
			})
		})
	}

	uploadFile(path: string, file: File): FileUploadQuery {
		const encodedPath = encodeURIComponent(path)
		const progressSubject = new EventTarget()
		const abortController = new AbortController()

		const uploadPromise = new Promise((resolve, reject) => {
			const xhr = new XMLHttpRequest()

			xhr.upload.addEventListener('progress', (e) => {
				if (e.lengthComputable) {
					const progress = (e.loaded / e.total) * 100
					progressSubject.dispatchEvent(
						new CustomEvent('progress', {
							detail: { loaded: e.loaded, total: e.total, progress },
						}),
					)
				}
			})

			xhr.onload = () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					resolve(xhr.response)
				} else {
					reject(new Error(`Upload failed with status ${xhr.status}`))
				}
			}

			xhr.onerror = () => reject(new Error('Upload failed'))
			xhr.onabort = () => reject(new Error('Upload cancelled'))

			xhr.open('POST', `https://${this.auth.url}/create?path=${encodedPath}&type=file`)
			xhr.setRequestHeader('Authorization', `Bearer ${this.auth.token}`)
			xhr.setRequestHeader('Content-Type', 'application/octet-stream')
			xhr.send(file)

			abortController.signal.addEventListener('abort', () => xhr.abort())
		})

		return {
			promise: uploadPromise,
			onProgress: (
				callback: (progress: { loaded: number; total: number; progress: number }) => void,
			) => {
				progressSubject.addEventListener('progress', ((e: CustomEvent) => {
					callback(e.detail)
				}) as EventListener)
			},
			cancel: () => abortController.abort(),
		} as FileUploadQuery
	}

	renameFileOrFolder(path: string, name: string): Promise<void> {
		const pathName = path.split('/').slice(0, -1).join('/') + '/' + name
		return this.retryWithAuth(async () => {
			await useServersFetch(`/move`, {
				method: 'POST',
				override: this.auth,
				body: { source: path, destination: pathName },
			})
		})
	}

	updateFile(path: string, content: string): Promise<void> {
		const octetStream = new Blob([content], { type: 'application/octet-stream' })
		return this.retryWithAuth(async () => {
			await useServersFetch(`/update?path=${path}`, {
				method: 'PUT',
				contentType: 'application/octet-stream',
				body: octetStream,
				override: this.auth,
			})
		})
	}

	moveFileOrFolder(path: string, newPath: string): Promise<void> {
		return this.retryWithAuth(async () => {
			await this.server.createMissingFolders(newPath.substring(0, newPath.lastIndexOf('/')))
			await useServersFetch(`/move`, {
				method: 'POST',
				override: this.auth,
				body: { source: path, destination: newPath },
			})
		})
	}

	deleteFileOrFolder(path: string, recursive: boolean): Promise<void> {
		const encodedPath = encodeURIComponent(path)
		return this.retryWithAuth(async () => {
			await useServersFetch(`/delete?path=${encodedPath}&recursive=${recursive}`, {
				method: 'DELETE',
				override: this.auth,
			})
		})
	}

	downloadFile(path: string, raw: boolean = false, ignoreFailure: boolean = false): Promise<any> {
		return this.retryWithAuth(async () => {
			const encodedPath = encodeURIComponent(path)
			const fileData = await useServersFetch(`/download?path=${encodedPath}`, {
				override: this.auth,
			})

			if (fileData instanceof Blob) {
				return raw ? fileData : await fileData.text()
			}
			return fileData
		}, ignoreFailure)
	}

	extractFile(
		path: string,
		override = true,
		dry = false,
		silentQueue = false,
	): Promise<{ modpack_name: string | null; conflicting_files: string[] }> {
		return this.retryWithAuth(async () => {
			const encodedPath = encodeURIComponent(path)

			if (!silentQueue) {
				this.queuedOps.push({ op: 'unarchive', src: path })
				setTimeout(() => this.removeQueuedOp('unarchive', path), 4000)
			}

			try {
				return await useServersFetch(
					`/unarchive?src=${encodedPath}&trg=/&override=${override}&dry=${dry}`,
					{
						method: 'POST',
						override: this.auth,
						version: 1,
					},
					undefined,
					'Error extracting file',
				)
			} catch (err) {
				this.removeQueuedOp('unarchive', path)
				throw err
			}
		})
	}

	modifyOp(id: string, action: 'dismiss' | 'cancel'): Promise<void> {
		return this.retryWithAuth(async () => {
			await useServersFetch(
				`/ops/${action}?id=${id}`,
				{
					method: 'POST',
					override: this.auth,
					version: 1,
				},
				undefined,
				`Error ${action === 'dismiss' ? 'dismissing' : 'cancelling'} filesystem operation`,
			)

			this.opsQueuedForModification = this.opsQueuedForModification.filter((x: string) => x !== id)
			this.ops = this.ops.filter((x: FilesystemOp) => x.id !== id)
		})
	}

	removeQueuedOp(op: FSQueuedOp['op'], src: string): void {
		this.queuedOps = this.queuedOps.filter((x: FSQueuedOp) => x.op !== op || x.src !== src)
	}

	clearQueuedOps(): void {
		this.queuedOps = []
	}
}
