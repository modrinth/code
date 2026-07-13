import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'

import {
	action,
	toggle,
	chips,
	fix,
	group,
	label,
	md,
	stage,
	text,
	check,
	button,
} from '../../types/node'

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

export default stage('title-slug', 'Title & Slug')
	.hint('Are the Name and URL accurate and appropriate?')
	.guidance('https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705')
	.icon(BookOpenIcon)
	.children(
		label(async (ctx) => {
			const title = await md('checklist/text/title-slug/title')(ctx)
			if (!hasCustomSlug(ctx.project)) return title
			return title + (await md('checklist/text/title-slug/slug')(ctx))
		}),

		group('title').children(
			toggle('useless_info', 'Contains Useless Info').action(
				action()
					.suggestedStatus('flagged')
					.severity('low')
					.message(md('checklist/messages/title/useless-info')),
			),

			toggle('minecraft_branding', 'Minecraft Title').action(
				action()
					.suggestedStatus('flagged')
					.severity('medium')
					.message(md('checklist/messages/title/minecraft-branding')),
			),

			toggle('similarities', 'Title Similarities')
				.action(
					action()
						.suggestedStatus('flagged')
						.severity('medium')
						.message(md('checklist/messages/title/similarities')),
				)
				.children(
					chips('options', 'Similarities Additional Info').children(
						check('modpack_named_after_mod', 'Modpack Named After Mod')
							.shown(({ project }) => project.project_types.includes('modpack'))
							.action(action().message(md('checklist/messages/title/similarities-modpack'))),

						check('forked_project', 'Forked Project')
							.shown(({ project }) => !project?.minecraft_server)
							.action(action().message(md('checklist/messages/title/similarities-fork'))),
					),
				),
		),

		group('slug')
			.shown(({ project }) => hasCustomSlug(project))
			.children(
				chips('options', 'Slug Issues?').children(
					check('misused', 'Misused')
						.children(
								text('correct_slug', 'Correct Slug')
									.initial((ctx) => ctx.project.slug),
								button('Auto')
									.onClick(ctx => {}))
						.action(
							action()
								.message(md('checklist/messages/slug/misused'))
								.fix(
									fix().project((patch, ctx) => {
										const slug = ctx.state.correct_slug as string
										if (!slug) return
										patch.slug = slug
									}),
								),
						),
				),
			),
	)
