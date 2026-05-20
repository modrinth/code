import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import type { Kyros } from '../types'

export type UploadSessionFile = {
	file: File | Blob
	filename: string
}

export class KyrosUploadSessionsV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_upload_sessions_v1'
	}

	public async create(
		scope: Kyros.UploadSessions.v1.Scope,
		worldId: string,
	): Promise<Kyros.UploadSessions.v1.UploadSessionResponse> {
		return this.client.request<Kyros.UploadSessions.v1.UploadSessionResponse>(
			`/worlds/${worldId}/${scope}/upload-session`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				useNodeAuth: true,
			},
		)
	}

	public async get(
		scope: Kyros.UploadSessions.v1.Scope,
		worldId: string,
	): Promise<Kyros.UploadSessions.v1.GetUploadSessionResponse> {
		return this.client.request<Kyros.UploadSessions.v1.GetUploadSessionResponse>(
			`/worlds/${worldId}/${scope}/upload-session`,
			{
				api: '',
				version: 'v1',
				method: 'GET',
				useNodeAuth: true,
			},
		)
	}

	public uploadFiles(
		scope: Kyros.UploadSessions.v1.Scope,
		worldId: string,
		uploadId: string,
		files: UploadSessionFile[],
		options?: {
			onProgress?: (progress: UploadProgress) => void
			retry?: boolean | number
		},
	): UploadHandle<Kyros.UploadSessions.v1.UploadSessionResponse> {
		const formData = new FormData()
		for (const { file, filename } of files) {
			formData.append('file', file, filename)
		}

		return this.client.upload<Kyros.UploadSessions.v1.UploadSessionResponse>(
			`/worlds/${worldId}/${scope}/upload-session/${uploadId}/files`,
			{
				api: '',
				version: 'v1',
				formData,
				onProgress: options?.onProgress,
				retry: options?.retry,
				useNodeAuth: true,
			},
		)
	}

	public async finalize(
		scope: Kyros.UploadSessions.v1.Scope,
		worldId: string,
		uploadId: string,
	): Promise<Kyros.UploadSessions.v1.UploadSessionResponse> {
		return this.client.request<Kyros.UploadSessions.v1.UploadSessionResponse>(
			`/worlds/${worldId}/${scope}/upload-session/${uploadId}/finalize`,
			{
				api: '',
				version: 'v1',
				method: 'POST',
				useNodeAuth: true,
			},
		)
	}

	public async cancel(
		scope: Kyros.UploadSessions.v1.Scope,
		worldId: string,
		uploadId: string,
	): Promise<Kyros.UploadSessions.v1.UploadSessionResponse> {
		return this.client.request<Kyros.UploadSessions.v1.UploadSessionResponse>(
			`/worlds/${worldId}/${scope}/upload-session/${uploadId}`,
			{
				api: '',
				version: 'v1',
				method: 'DELETE',
				useNodeAuth: true,
			},
		)
	}
}
