import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'

import {
	action,
	button,
	fix,
	group,
	label,
	md,
	option,
	stage,
	stageFn,
	text,
	toggle,
} from '../../types/node'

//TODO: make this not a copy of frontend/src/utils/slugs.generateUrlSlug
// (as in move the other one so we can use it here)
function generateUrlSlug(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replaceAll(' ', '-')
		.replaceAll(/[^a-zA-Z0-9._-]/g, '')
		.replaceAll(/--+/gm, '-')
}

function hasCustomSlug(project: Labrinth.Projects.v3.Project) {
	return generateUrlSlug(project.name) !== project.slug
}

//TODO:some sort of input somewhere had placeholders but i forgot so probably figure out which that one was

export default stageFn((project) =>
	stage('title-slug', 'Title & Slug')
		.hint('Are the Name and URL accurate and appropriate?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
		)
		.icon(BookOpenIcon)
		.children(
			label(async (ctx) => {
				const title = await md('checklist/text/title-slug/title')(ctx)
				if (!hasCustomSlug(ctx.project)) return title
				return title + (await md('checklist/text/title-slug/slug')(ctx))
			}),

			group('title')
				.title('Title Issues?')
				.children(
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
							group('options')
								.title('Similarities Additional Info')
								.multiSelect()
								.children(
									option('modpack_named_after_mod', 'Modpack Named After Mod')
										.shown(project.project_types.includes('modpack'))
										.action(action().message(md('checklist/messages/title/similarities-modpack'))),

									option('forked_project', 'Forked Project')
										.shown(!project?.minecraft_server)
										.action(action().message(md('checklist/messages/title/similarities-fork'))),
								),
						),
				),

			group('slug')
				.title('Slug Issues')
				.shown(hasCustomSlug(project))
				.children(
					group('options')
						.multiSelect()
						.children(
							option('misused', 'Misused')
								.children(
									group()
										.title('Correct Slug')
										.children(
                      //TODO: probably make this reset to current slug if you clear it?
											text('correct_slug')
                        .initial(project.slug),
											button('Auto')
												.enabled(project.slug !== generateUrlSlug(project.name))
												.onClick((ctx) => (ctx.state.correct_slug = generateUrlSlug(project.name))),
										),
								)
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
		),
)
