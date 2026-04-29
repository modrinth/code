export const getReportPath = (type: string, id: string) => {
	const prefill = new URLSearchParams()
	prefill.set('item', type)
	prefill.set('itemID', id)
	return '/report?' + prefill.toString()
}

const startReport = (type: string, id: string) => {
	navigateTo(getReportPath(type, id))
}

export const reportProject = (id: string) => {
	return startReport('project', id)
}

export const reportVersion = (id: string) => {
	return startReport('version', id)
}

export const reportUser = (id: string) => {
	return startReport('user', id)
}
