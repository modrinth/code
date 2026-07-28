import { ListBulletedIcon } from '@modrinth/assets'
import { injectProjectPageContext } from '@modrinth/ui'
import { computed } from 'vue'

import { toggle, group, markdown, stage } from '../../types/node'

export default function () {
	const { projectV3: project } = injectProjectPageContext()

	return stage('rules', 'Rule Following')
		.hint('Does this project violate the rules?')
		.guidance(
			'https://www.notion.so/2e15ee711bf080e4a41df61bbab49892#2e35ee711bf080709084f6269835607f',
		)
		.icon(ListBulletedIcon)
		.navigate('/moderation')
		.children(
			group().children(
				toggle('paid-access-server', 'Paid access server')
					.shown(computed(() => !!project.value.minecraft_server))
					.suggestedStatus('rejected')
					.severity('critical')
					.message(),

				toggle('prohibited-content', 'Prohibited Content')
					.suggestedStatus('rejected')
					.severity('critical')
					.message('prohibited-content-header')
					.collect()
					.children(
						group()
							.title('Which Prohibited Content rules does this project violate?')
							.children(
								toggle('objectionable', 'Objectionable').message(),
								toggle('discriminatory', 'Discriminatory or Explicit').message(),
								toggle('ip-infringement', 'IP Infringement').message(),
								toggle('legal-rights', 'Rights Violation').message(),
								toggle('illegal-activity', 'Illegal Activity').message(),
								toggle('harmful', 'Harmful or Deceptive').message(),
								toggle('misleading', 'Misleading claims').message(),
								toggle('impersonation', 'Impersonation').message(),
								toggle('false-endorsement', 'False Endorsement').message(),
								toggle('profanity', 'Profanity').message(),
								toggle('undisclosed-upload', 'Undisclosed Data Upload').message(),
								toggle('mojang-bypass', 'Mojang Bypass').message(),
							),
					),

				toggle('cheat-or-hack-advertising', 'Hacks')
					.suggestedStatus('rejected')
					.severity('critical')
					.message(),

				toggle('server-side-opt-out', 'Opt-out')
					.suggestedStatus('flagged')
					.severity('high')
					.message(),

				toggle('server-side-opt-in', 'Opt-in')
					.suggestedStatus('flagged')
					.severity('high')
					.message('server-side-opt-in-header')
					.collect()
					.children(
						group()
							.title('Which features require a Server-side Opt-in?')
							.children(
								toggle('x-ray', 'X-ray').message(),
								toggle('aim-bot', 'Aim Assist').message(),
								toggle('movement', 'Movement').message(),
								toggle('pvp', 'PvP').message(),
								toggle('hiding-mods', 'Anti 3.x').message(),
								toggle('item-duplication', 'Dupe').message(),
							),
					),

				toggle('excessive-languages', 'Excessive languages')
					.shown(
						computed(
							() =>
								!!project.value.minecraft_server &&
								!!project.value.minecraft_server?.languages?.length &&
								project.value.minecraft_server.languages.length > 4,
						),
					)
					.suggestedStatus('flagged')
					.severity('low')
					.message(),

				toggle('rule-breaking-other', 'Other')
					.suggestedStatus('rejected')
					.severity('critical')
					.message(undefined, (state) => ({ MESSAGE: state.message }))
					.children(
						markdown('message').title('Explain how it infringes on content rules.').required(),
					),
			),
		)
}
