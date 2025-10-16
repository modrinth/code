export const useNotificationRightwards = () => {
	const isVisible = useState('moderation-checklist-notifications', () => false)

	const setVisible = (visible: boolean) => {
		isVisible.value = visible
	}

	return {
		isVisible: readonly(isVisible),
		setVisible,
	}
}
