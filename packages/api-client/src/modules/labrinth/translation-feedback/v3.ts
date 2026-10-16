import { AbstractModule } from '../../../core/abstract-module'
import type { Labrinth } from '../types'

export class LabrinthTranslationFeedbackV3Module extends AbstractModule {
	public getModuleID(): string {
		return 'labrinth_translation_feedback_v3'
	}

	/**
	 * Send feedback about a translated string to the translators
	 *
	 * @param data - The reported string and the player's feedback
	 *
	 * @example
	 * ```typescript
	 * await client.labrinth.translation_feedback_v3.create({
	 *   product: 'website',
	 *   locale: 'de-DE',
	 *   key: 'settings.language.title',
	 *   kind: 'typo',
	 *   seen_text: 'Sprahce',
	 *   suggestion: 'Sprache',
	 * })
	 * ```
	 */
	public async create(
		data: Labrinth.TranslationFeedback.v3.CreateTranslationFeedbackRequest,
	): Promise<void> {
		return this.client.request(`/translation-feedback`, {
			api: 'labrinth',
			version: 3,
			method: 'POST',
			body: data,
		})
	}
}
