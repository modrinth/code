import { injectProjectPageContext, type MessageDescriptor, useVIntl } from '@modrinth/ui'

export function useProjectSettingsHeadTitle(section: MessageDescriptor) {
	const { formatMessage } = useVIntl()
	const { projectV2: project } = injectProjectPageContext()

	useHead({
		title: () => `${formatMessage(section)} - ${project.value.title}`,
	})
}
