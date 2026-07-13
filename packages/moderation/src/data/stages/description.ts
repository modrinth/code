import { LibraryIcon } from '@modrinth/assets'

import { action, toggle, group, markdown, md, select, stage, check } from '../../types/node'

export default stage('description', 'Description')
	.hint('Is the description sufficient, accurate, and accessible?')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf080508042e70089dd787e')
	.icon(LibraryIcon)
	.navigate('/')
	.children(
		group().children(
			toggle('insufficient', 'Insufficient')
				.action(
					action()
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
						check('fork', 'Fork'),
						check('custom', 'Custom').children(
							markdown(
								'explainer',
								'How can the author improve their description?',
							).required(),
						),
					),
				),

			toggle('non_english', 'Non-english')
				.shown(({ project }) => !project.minecraft_java_server)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-english/non-english')),
				),

			toggle('non_english_server', 'Non-english')
				.shown(({ project }) => !!project.minecraft_java_server)
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-english/non-english-server')),
				),

			toggle('unfinished', 'Unfinished')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/description/unfinished')),
				),

			toggle('headers_as_body', 'Headers as body text')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/description/accessability/headers-as-body')),
				),

			toggle('image_only', 'Image-only')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/image-only')),
				),

			toggle('non_standard_text', 'Non-standard text')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/description/accessability/non-standard-text')),
				),

			toggle('clarity', 'Unclear / Misleading')
				.action(
					action()
						.suggestedStatus('rejected')
						.severity('high')
						.message(md('checklist/messages/description/clarity')),
				),
		),
	)
