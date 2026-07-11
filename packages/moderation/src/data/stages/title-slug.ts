import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'

import { button, chips, group, mdMsg, mdText, prose, stage, toggle } from '../../types/node'

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
	[
		prose(async (ctx) => {
			const title = await mdText('title-slug/title')(ctx)
			if (!hasCustomSlug(ctx.project)) return title
			return title + await mdText('title-slug/slug')(ctx)
		}),

		group('title').children(
			button('useless_info', 'Contains Useless Info')
				.weight(100)
				.suggestedStatus('flagged')
				.severity('low')
				.message(mdMsg('title/useless-info')),

			button('minecraft_branding', 'Minecraft Title')
				.weight(100)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(mdMsg('title/minecraft-branding')),

			button('similarities', 'Title Similarities')
				.weight(110)
				.suggestedStatus('flagged')
				.severity('medium')
				.message(mdMsg('title/similarities'))
				.children(
					chips('options', 'Similarities Additional Info')
						.children(
							toggle('modpack_named_after_mod', 'Modpack Named After Mod')
								.shown(({ project }) => project.project_types.includes('modpack'))
								.weight(111)
								.message(mdMsg('title/similarities-modpack')),

							toggle('forked_project', 'Forked Project')
								.shown(({ project }) => !project?.minecraft_server)
								.weight(112)
								.message(mdMsg('title/similarities-fork')),
						),
				),
		),

		group('slug')
			.column()
			.shown(({ project }) => hasCustomSlug(project))
			.children(
				chips('options', 'Slug Issues?')
					.suggestedStatus('rejected')
					.severity('low')
					.children(
						toggle('misused', 'Misused')
							.weight(200)
							.message(mdMsg('slug/misused')),
					),
			),
	],
	{ icon: BookOpenIcon },
)
