import { TriangleAlertIcon } from '@modrinth/assets'

import { group, option, stage, toggle } from '../../types/node'

export default function () {
	return stage('disclosures', 'Disclosures')
		.hint('Has this project selected all proper content disclosures?')
		.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892')
		.icon(TriangleAlertIcon)
		.navigate('/settings/disclosures')
		.children(
			group().children(
				toggle('missing-disclosures', 'Disclosures Missing')
					.message('missing-disclosures/missing-disclosures-header')
					.children(
						group()
							.title('Which content disclosures are missing?')
							.children(
								toggle('ai', 'AI Usage')
									.suggestedStatus('flagged')
									.message()
									.children(
										group()
											.title('What kind of AI content is present?')
											.children(
												toggle('code', 'Code').message('ai-usages/code'),
												toggle('assets', 'Assets').message('ai-usages/assets'),
												toggle('text', 'Text').message('ai-usages/text'),
												toggle('functionality', 'Functionality').message('ai-usages/functionality'),
											),
									)
									.collect(undefined, 'ai/list-intro'),

								toggle('ads', 'Advertisements').suggestedStatus('flagged').message(),

								toggle('paid-features', 'Paid Features').suggestedStatus('flagged').message(),

								toggle('telemetry', 'Telemetry').suggestedStatus('rejected').message(),

								toggle('derivative-content', 'Derivative Content')
									.suggestedStatus('rejected')
									.message(),

								toggle('photosensitivity', 'Photosensitivity')
									.suggestedStatus('rejected')
									.message(),

								toggle('system-interactions', 'System Interactions')
									.suggestedStatus('rejected')
									.message(),

								toggle('archive', 'Archive').message(),
							),
					)
					.collect(undefined, 'missing-disclosures/list-intro'),
			),
		)
}
