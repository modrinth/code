import { TriangleAlertIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { group, markdown, md, option, stage, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('disclosures', 'Disclosures')
		.hint('Has this project selected all proper content disclosures?')
		.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892')
		.icon(TriangleAlertIcon)
		.navigate('/settings/disclosures')
		.children(
			group().children(
				toggle('missing-disclosures', 'Disclosures Missing')
					.rawMessage(async (state) => {
						if (state?.['missing-disclosures'])
							return md(
								'checklist/messages/disclosures/missing-disclosures/missing-disclosures-header',
								() => ({
									DISCLOSURES_SELECTED: md(
										'checklist/messages/disclosures/missing-disclosures/disclosures-selected',
									),
								}),
							)(state)

						return md(
							'checklist/messages/disclosures/missing-disclosures/missing-disclosures-header',
							() => ({
								DISCLOSURES_SELECTED: '',
							}),
						)(state)
					})

					.children(
						group()
							.title('Which content disclosures are missing?')
							.children(
								option('ai', 'AI Usage')
									.suggestedStatus('flagged')
									.message()
									.children(
										group()
											.title('What kind of AI content is present?')
											.children(
												option('code', 'Code').message(),
												option('assets', 'Assets').message(),
												option('text', 'Text').message(),
												option('functionality', 'Functionality').message(),
											),
									),

								option('ads', 'Advertisements').suggestedStatus('flagged').message(),

								option('paid-features', 'Paid Features').suggestedStatus('flagged').message(),

								option('telemetry', 'Telemetry').suggestedStatus('rejected').message(),

								option('derivative-content', 'Derivative Content')
									.suggestedStatus('rejected')
									.message(),

								option('photosensitivity', 'Photosensitivity')
									.suggestedStatus('rejected')
									.message(),

								option('system-interactions', 'System Interactions')
									.suggestedStatus('rejected')
									.message(),

								option('archive', 'Archive').message(),
							),
					)
					.collect(),
			),
		)
}
