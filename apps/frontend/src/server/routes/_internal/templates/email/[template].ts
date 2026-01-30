import { render } from '@vue-email/render'
import type { Component } from 'vue'

import emails from '~/templates/emails'

export default defineEventHandler(async (event) => {
	const template = event.context.params?.template as string

	if (template === 'dynamic') {
		throw createError({
			statusCode: 404,
			message: 'Email template not found',
		})
	}

	try {
		const component = (await emails[template]()).default as Component | undefined

		if (!component) {
			throw createError({
				statusCode: 404,
				message: 'Email template not found',
			})
		}

		const html = await render(component, {})

		return html
	} catch (error) {
		console.error(`Error rendering email template ${template}:`, error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render email template',
		})
	}
})
