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
								ORIGINAL_PROJECT: state.original_project,
								ORIGINAL_AUTHOR: state.original_author,
							})),
					)
					.children(
						text('original_project').title('Original Project Title').required(),
						text('original_author').title('Original project Author').required(),
					),

				toggle('unclear_fork', 'Unclear Fork')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('insufficient_fork', 'Insufficient Fork')
					.shown(computed(() => !project.value.minecraft_server))
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('request_proof', 'Proof of permissions').action(
					action().suggestedStatus('rejected').severity('high').message(),
				),

				toggle('identity_verification', 'Verify Identity')
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

				toggle('identity_verification_server', 'Verify Identity')
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

				toggle('request_proof_server', 'Reuploaded pack')
					.shown(isServerModpack)
					.action(action().suggestedStatus('rejected').severity('high').message()),

				toggle('custom_pack_verification', 'Override verification')
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

				toggle('custom_pack_prohibited', 'Forbidden Overrides')
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
