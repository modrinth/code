import { AbstractModule } from '../../../core/abstract-module'
import type { Paste } from '../types'

const PASTE_BASE_URL = 'https://paste.modrinth.com'

export class PastePastesV0Module extends AbstractModule {
	public getModuleID(): string {
		return 'paste_pastes_v0'
	}

	/**
	 * Upload plain text to paste.modrinth.com.
	 *
	 * @param content - Text to upload
	 * @param language - Language id used for highlighting, sent as `text/<language>`
	 * @returns The key of the new paste
	 */
	public async create(
		content: string,
		language = 'plain',
	): Promise<Paste.Pastes.v0.CreateResponse> {
		return this.client.request<Paste.Pastes.v0.CreateResponse>('/post', {
			api: PASTE_BASE_URL,
			version: '',
			method: 'POST',
			body: content,
			headers: { 'Content-Type': `text/${language}` },
			skipAuth: true,
		})
	}

	/**
	 * Get the shareable URL for a paste.
	 *
	 * @param key - Key returned by `create`
	 */
	public getUrl(key: string): string {
		return `${PASTE_BASE_URL}/${key}`
	}
}
