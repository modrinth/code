import { CopyrightIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { action, check, group, markdown, stage, text, toggle } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	const isServerModpack = computed(
		() =>
			!!project.value.minecraft_server &&
			project.value.minecraft_java_server?.content?.kind === 'modpack' &&
			project.value.minecraft_java_server?.content?.project_id === project.value.id,
	)

	return stage('reupload', 'Reupload')
		.hint('Does the author have proper permissions to post this project?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080d1a0a2cda3ff2ce997',
		)
		.icon(CopyrightIcon)
		.children(
			group().children(
				toggle('reupload', 'Re-upload')
					.shown(computed(() => !project.value.minecraft_server))
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('high')
							.message((state) => ({
								ORIGINAL_PROJECT: state.originalProject,
								ORIGINAL_AUTHOR: state.originalAuthor,
							})),
					)
					.children(
						text('originalProject').title('Original Project Title').required(),
						text('originalAuthor').title('Original project Author').required(),
					),

				toggle('unclearFork', 'Unclear Fork')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('insufficientFork', 'Insufficient Fork')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('requestProof', 'Proof of permissions').action(
					action().suggestedStatus('rejected').severity('high').message(),
				),

				toggle('identityVerification', 'Verify Identity')
					.shown(computed(() => !project.value.minecraft_server))
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('high')
							.message((state) => ({
								PLATFORM: state.platform,
							})),
					)
					.children(text('platform').title('Where else can the project be found?').required()),

				toggle('identityVerification-server', 'Verify Identity')
					.shown(computed(() => !!project.value.minecraft_server))
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('high')
							.message((state) => ({
								CONTACT: state.contact,
							})),
					)
					.children(text('contact').title('Known public contact method').required()),

				toggle('requestProof-server', 'Reuploaded pack')
					.shown(isServerModpack)
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('customPackVerification', 'Override verification')
					.shown(isServerModpack)
					.action(action().suggestedStatus('rejected').severity('high').message())
					.children(
						check('list', 'List overrides?')
							.action(
								action().message((state) => ({
									OVERRIDES: state.overrides,
								})),
							)
							.children(markdown('overrides').title('Add list of overrides.')),
					),

				toggle('customPackProhibited', 'Forbidden Overrides')
					.shown(isServerModpack)
					.action(
						action()
							.suggestedStatus('rejected')
							.severity('high')
							.message((state) => ({
								OVERRIDES: state.overrides,
							})),
					)
					.children(markdown('overrides').title('Forbidden overrides list').required()),
			),
		)
}
