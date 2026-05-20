export function addReportMessage(thread, report) {
	if (!thread || !report) {
		return thread
	}

	const reporterId = report.reporterUser?.id ?? report.reporter
	if (!reporterId) {
		return thread
	}

	const members = Array.isArray(thread.members) ? [...thread.members] : []
	const messages = Array.isArray(thread.messages) ? [...thread.messages] : []

	let changed = false

	if (
		!members.some((user) => {
			return user?.id === reporterId
		}) &&
		report.reporterUser
	) {
		members.push(report.reporterUser)
		changed = true
	}

	if (!messages.some((message) => message?.id === 'original')) {
		messages.push({
			id: 'original',
			author_id: reporterId,
			body: {
				type: 'text',
				body: report.body,
				private: false,
				replying_to: null,
			},
			created: report.created,
		})
		changed = true
	}

	if (!changed) {
		return thread
	}

	return {
		...thread,
		members,
		messages,
	}
}
