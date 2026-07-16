import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import {
	action,
	button,
	fix,
	group,
	label,
	md,
	option,
	stage,
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

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const titleMsg = md('checklist/text/titleSlug/title')
	const slugMsg = md('checklist/text/titleSlug/slug')

	return stage('titleSlug', 'Title & Slug')
		.hint('Are the Name and URL accurate and appropriate?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
		)
		.icon(BookOpenIcon)
		.children(
			label(async (state) => {
				const title = await titleMsg(state)
				if (!hasCustomSlug(project.value)) return title
				return title + (await slugMsg(state))
			}),

			group('title')
				.title('Title Issues?')
				.children(
					toggle('uselessInfo', 'Contains Useless Info').action(
						action().suggestedStatus('flagged').severity('low').message(),
					),

					toggle('minecraftBranding', 'Minecraft Title').action(
						action().suggestedStatus('flagged').severity('medium').message(),
					),

					toggle('similarities', 'Title Similarities')
						.action(action().suggestedStatus('flagged').severity('medium').message())
						.children(
							group()
								.title('Similarities Additional Info')
								.multiSelect('options')
								.children(
									option('modpack', 'Modpack Named After Mod')
										.shown(computed(() => project.value.project_types.includes('modpack')))
										.action(action().message()),

									option('fork', 'Forked Project')
										.shown(computed(() => !project.value?.minecraft_server))
										.action(action().message()),
								),
						),
				),

			group('slug')
				.title('Slug Issues')
				.shown(computed(() => hasCustomSlug(project.value)))
				.children(
					group()
						.multiSelect('issues')
						.children(
							toggle('misused', 'Misused')
								.children(
									group()
										.title('Correct Slug')
										.children(
											//TODO: chyz probably make this reset to current slug if you clear it?
											//TODO: chyz make this validate slugs are free
											text('correctSlug').initial(project.value.slug),

											button('Auto')
												.enabled(
													computed(
														() => project.value.slug !== generateUrlSlug(project.value.name),
													),
												)
												.onClick(
													(state) => (state.correctSlug = generateUrlSlug(project.value.name)),
												),
										),
								)
								.action(
									action()
										//TODO: coolbot variant for when we suggest a better slug (also used when quick fix is applied)
										.message()
										.fix(
											fix().project((patch, state) => {
												const slug = state.correctSlug as string
												if (!slug) return
												patch.slug = slug
											}),
										),
								),
						),
				),
		)
}
