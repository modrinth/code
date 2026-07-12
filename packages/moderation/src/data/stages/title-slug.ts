import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'

import { action, button, chips, group, label, md, stage, toggle } from '../../types/node'

function hasCustomSlug(project: Labrinth.Projects.v3.Project): boolean {
	return (
		project.slug !==
		project.name
			.trim()
			.toLowerCase()
			.replaceAll(' ', '-')
			.replaceAll(/[^a-zA-Z0-9!@$()`.+,_"-]/g, '')
			.replaceAll(/--+/gm, '-')
	)
}

export default stage(
	'title-slug',
	'Title & Slug',
	'Are the Name and URL accurate and appropriate?',
	'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
)
	.icon(BookOpenIcon)
	.children(
		label(async (ctx) => {
			const title = await md('checklist/text/title-slug/title')(ctx)
			if (!hasCustomSlug(ctx.project)) return title
			return title + (await md('checklist/text/title-slug/slug')(ctx))
		}),

		group('title').children(
			button('useless_info', 'Contains Useless Info')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('low')
						.message(md('checklist/messages/title/useless-info')),
				),

			button('minecraft_branding', 'Minecraft Title')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/title/minecraft-branding')),
				),

			button('similarities', 'Title Similarities')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/title/similarities')),
				)
				.children(
					chips('options', 'Similarities Additional Info').children(
						toggle('modpack_named_after_mod', 'Modpack Named After Mod')
							.shown(({ project }) => project.project_types.includes('modpack'))
							.action(
								action()
									.message(md('checklist/messages/title/similarities-modpack')),
							),

						toggle('forked_project', 'Forked Project')
							.shown(({ project }) => !project?.minecraft_server)
							.action(
								action()
									.message(md('checklist/messages/title/similarities-fork')),
							),
					),
				),
		),

		group('slug')
			.layout('column')
			.shown(({ project }) => hasCustomSlug(project))
			.children(
				chips('options', 'Slug Issues?')
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('low'),
					)
					.children(
						toggle('misused', 'Misused')
							.action(
								action()
									.message(md('checklist/messages/slug/misused')),
							),
					),
			),
	)
