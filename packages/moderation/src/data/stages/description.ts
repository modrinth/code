import { LibraryIcon } from '@modrinth/assets'

import { action, button, group, markdown, md, select, stage, toggle } from '../../types/node'

export default stage(
	'description',
	'Description',
	'Is the description sufficient, accurate, and accessible?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e',
)
	.icon(LibraryIcon)
	.navigate('/')
	.children(
		group().children(
			button('insufficient', 'Insufficient')
				.action(
					action()
						.weight(400)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(async (ctx) => {
							const reason = ctx?.state.reason as string | undefined
							if (reason === 'custom') {
								return md('checklist/messages/description/insufficient/custom', (c) => ({
									EXPLAINER: c.state.explainer,
								}))(ctx)
							}
							if (reason === 'fork') {
								const header = await md('checklist/messages/description/insufficient/header')(ctx)
								const detail = await md('checklist/messages/description/insufficient/fork')(ctx)
								return `${header}\n\n${detail}`
							}
							return md(
								`checklist/messages/description/insufficient/${ctx?.project?.minecraft_java_server ? 'servers' : ctx?.project?.project_types?.includes('modpack') ? 'packs' : 'projects'}`,
							)(ctx)
						}),
				)
				.children(
					select('reason', 'Specific reason?').children(
						toggle('fork', 'Fork'),
						toggle('custom', 'Custom').children(
							markdown(
								'explainer',
								'How can the author improve their description?',
							).required(),
						),
					),
				),

			button('non_english', 'Non-english')
				.shown(({ project }) => !project.minecraft_java_server)
				.action(
					action()
						.weight(402)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-english/non-english')),
				),

			button('non_english_server', 'Non-english')
				.shown(({ project }) => !!project.minecraft_java_server)
				.action(
					action()
						.weight(402)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-english/non-english-server')),
				),

			button('unfinished', 'Unfinished')
				.action(
					action()
						.weight(403)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/description/unfinished')),
				),

			button('headers_as_body', 'Headers as body text')
				.action(
					action()
						.weight(404)
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/description/accessability/headers-as-body')),
				),

			button('image_only', 'Image-only')
				.action(
					action()
						.weight(405)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/image-only')),
				),

			button('non_standard_text', 'Non-standard text')
				.action(
					action()
						.weight(406)
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-standard-text')),
				),

			button('clarity', 'Unclear / Misleading')
				.action(
					action()
						.weight(407)
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/description/clarity')),
				),
		),
	)
