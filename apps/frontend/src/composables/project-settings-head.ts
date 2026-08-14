import {
	defineMessages,
	injectProjectPageContext,
	type MessageDescriptor,
	useVIntl,
} from '@modrinth/ui'

const messages = defineMessages({
	headTitle: {
		id: 'project.settings.head-title',
		defaultMessage: '⚙️ {section} - {project}',
	},
})

export function useProjectSettingsHeadTitle(section: MessageDescriptor) {
	const { formatMessage } = useVIntl()
	const { projectV2: project } = injectProjectPageContext()

	useHead({
		title: () =>
			`${formatMessage(messages.headTitle, {
				section: formatMessage(section),
				project: project.value.title,
			})}`,
	})
}
