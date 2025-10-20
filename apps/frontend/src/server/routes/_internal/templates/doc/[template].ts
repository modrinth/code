import { render } from '@vue-email/render'
import type { Component } from 'vue'

import docs from '~/templates/docs'

export default defineEventHandler(async (event) => {
	const template = event.context.params?.template as string
	try {
		const component = (await docs[template]()).default as Component | undefined

		if (!component) {
			throw createError({
				statusCode: 404,
				message: 'Document template not found',
			})
		}

		const html = await render(component, {})

		return html
	} catch (error) {
		console.error(`Error rendering document template ${template}:`, error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render document template',
		})
	}
})
