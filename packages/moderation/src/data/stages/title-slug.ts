import type { Labrinth } from '@modrinth/api-client'
import { BookOpenIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { button, fix, group, md, option, rawLabel, stage, text, toggle } from '../../types/node'

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

	const titleMsg = md('checklist/text/title-slug/title')
	const slugMsg = md('checklist/text/title-slug/slug')

	return stage('title-slug', 'Title & Slug')
		.hint('Are the Name and URL accurate and appropriate?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e15ee711bf0803c9660e90f0fead705',
		)
		.icon(BookOpenIcon)
		.children(
			rawLabel(async (state) => {
				const title = await titleMsg(state)
				if (!hasCustomSlug(project.value)) return title
				return title + (await slugMsg(state))
			}),

			group('title')
				.title('Title Issues?')
				.children(
					toggle('useless-info', 'Contains Useless Info')
						.suggestedStatus('flagged')
						.severity('low')
						.message(),

					toggle('minecraft-branding', 'Minecraft Title')
						.suggestedStatus('flagged')
						.severity('medium')
						.message(),

					toggle('similarities', 'Title Similarities')
						.suggestedStatus('flagged')
						.severity('medium')
						.message()
						.children(
							group()
								.title('Similarities Additional Info')
								.multiSelect('options')
								.children(
									option('modpack', 'Modpack Named After Mod')
										.shown(computed(() => project.value.project_types.includes('modpack')))
										.message(),

									option('fork', 'Forked Project')
										.shown(computed(() => !project.value?.minecraft_server))
										.message(),
								),
						)
						.collect(),
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
											text('correct-slug').initial(project.value.slug),

											button('Auto')
												.enabled(
													computed(
														() => project.value.slug !== generateUrlSlug(project.value.name),
													),
												)
												.onClick(
													(state) => (state['correct-slug'] = generateUrlSlug(project.value.name)),
												),
										),
								)
								.rawMessage(async (state) => {
									return await md('checklist/messages/title-slug/slug/misused', () => ({
										SUGGESTED_SLUG: state['correct-slug'],
									}))(state)
								})
								.fix(
									fix().project((patch, state) => {
										const slug = state['correct-slug'] as string
										if (!slug) return
										patch.slug = slug
									}),
								),
						),
				),
		)
}
