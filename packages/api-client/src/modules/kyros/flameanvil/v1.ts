import { AbstractModule } from '../../../core/abstract-module'
import type { UploadHandle, UploadProgress } from '../../../types/upload'
import { getNodeBaseUrl } from '../../../utils/node-url'
import type { Archon } from '../../archon/types'
import type { Kyros } from '../types'

export type FlameAnvilAuthOptions = {
	auth?: Archon.Servers.v0.JWTAuth
}

export type FlameAnvilUploadOptions = FlameAnvilAuthOptions & {
	onProgress?: (progress: UploadProgress) => void
}

function getSessionPath(worldId: string) {
	return `/worlds/${worldId}/content/flameanvil/session`
}

function nodeRequestOptions(auth?: Archon.Servers.v0.JWTAuth) {
	return {
		api: auth ? getNodeBaseUrl(auth.url) : '',
		version: 'v1' as const,
		...(auth
			? { headers: { Authorization: `Bearer ${auth.token}` }, skipAuth: true }
			: { useNodeAuth: true }),
	}
}

/**
 * Installs CurseForge modpacks, either from a CurseForge link or from uploaded client and server packs.
 */
export class KyrosFlameAnvilV1Module extends AbstractModule {
	public getModuleID(): string {
		return 'kyros_flameanvil_v1'
	}

	/** POST /v1/worlds/:world_id/content/flameanvil/link/query */
	public async queryLink(
		worldId: string,
		data: Kyros.FlameAnvil.v1.LinkQueryRequest,
		options?: FlameAnvilAuthOptions,
	): Promise<Kyros.FlameAnvil.v1.LinkQueryResponse> {
		return this.client.request<Kyros.FlameAnvil.v1.LinkQueryResponse>(
			`/worlds/${worldId}/content/flameanvil/link/query`,
			{
				...nodeRequestOptions(options?.auth),
				method: 'POST',
				body: data,
			},
		)
	}

	/**
	 * GET /v1/worlds/:world_id/content/flameanvil/projects/:project_id/files
	 *
	 * Lists a page of the modpack's client pack files, most recent first.
	 */
	public async listProjectFiles(
		worldId: string,
		projectId: number,
		params?: Kyros.FlameAnvil.v1.ProjectFilesRequest,
		options?: FlameAnvilAuthOptions,
	): Promise<Kyros.FlameAnvil.v1.ProjectFilesResponse> {
		return this.client.request<Kyros.FlameAnvil.v1.ProjectFilesResponse>(
			`/worlds/${worldId}/content/flameanvil/projects/${projectId}/files`,
			{
				...nodeRequestOptions(options?.auth),
				method: 'GET',
				params,
			},
		)
	}

	/** GET /v1/worlds/:world_id/content/flameanvil/session */
	public async getSession(
		worldId: string,
		options?: FlameAnvilAuthOptions,
	): Promise<Kyros.FlameAnvil.v1.GetSessionResponse> {
		return this.client.request<Kyros.FlameAnvil.v1.GetSessionResponse>(getSessionPath(worldId), {
			...nodeRequestOptions(options?.auth),
			method: 'GET',
		})
	}

	/** POST /v1/worlds/:world_id/content/flameanvil/session */
	public async createSession(
		worldId: string,
		options?: FlameAnvilAuthOptions,
	): Promise<Kyros.FlameAnvil.v1.SessionResponse> {
		return this.client.request<Kyros.FlameAnvil.v1.SessionResponse>(getSessionPath(worldId), {
			...nodeRequestOptions(options?.auth),
			method: 'POST',
		})
	}

	/** DELETE /v1/worlds/:world_id/content/flameanvil/session/:upload_id */
	public async cancelSession(
		worldId: string,
		uploadId: string,
		options?: FlameAnvilAuthOptions,
	): Promise<Kyros.FlameAnvil.v1.SessionResponse> {
		return this.client.request<Kyros.FlameAnvil.v1.SessionResponse>(
			`${getSessionPath(worldId)}/${uploadId}`,
			{
				...nodeRequestOptions(options?.auth),
				method: 'DELETE',
			},
		)
	}

	/**
	 * POST /v1/worlds/:world_id/content/flameanvil/session/:upload_id/upload/player
	 *
	 * Uploads the client modpack. Uploading again replaces it.
	 */
	public uploadPlayerPack(
		worldId: string,
		uploadId: string,
		file: File | Blob,
		options?: FlameAnvilUploadOptions,
	): UploadHandle<Kyros.FlameAnvil.v1.SessionResponse> {
		return this.uploadPack(worldId, uploadId, 'player', file, options)
	}

	/**
	 * POST /v1/worlds/:world_id/content/flameanvil/session/:upload_id/upload/server
	 *
	 * Uploads the server pack. Uploading again replaces it.
	 */
	public uploadServerPack(
		worldId: string,
		uploadId: string,
		file: File | Blob,
		options?: FlameAnvilUploadOptions,
	): UploadHandle<Kyros.FlameAnvil.v1.SessionResponse> {
		return this.uploadPack(worldId, uploadId, 'server', file, options)
	}

	/**
	 * POST /v1/worlds/:world_id/content/flameanvil/session/:upload_id/execute
	 *
	 * Queues the installation, either from `data.link` or from the uploaded packs.
	 */
	public async executeSession(
		worldId: string,
		uploadId: string,
		data: Kyros.FlameAnvil.v1.ExecuteSessionRequest,
		options?: FlameAnvilAuthOptions,
	): Promise<void> {
		return this.client.request<void>(`${getSessionPath(worldId)}/${uploadId}/execute`, {
			...nodeRequestOptions(options?.auth),
			method: 'POST',
			body: data,
		})
	}

	private uploadPack(
		worldId: string,
		uploadId: string,
		side: 'player' | 'server',
		file: File | Blob,
		options?: FlameAnvilUploadOptions,
	): UploadHandle<Kyros.FlameAnvil.v1.SessionResponse> {
		const formData = new FormData()
		formData.append('file', file, file instanceof File ? file.name : 'file')

		return this.client.upload<Kyros.FlameAnvil.v1.SessionResponse>(
			`${getSessionPath(worldId)}/${uploadId}/upload/${side}`,
			{
				...nodeRequestOptions(options?.auth),
				formData,
				onProgress: options?.onProgress,
			},
		)
	}
}
