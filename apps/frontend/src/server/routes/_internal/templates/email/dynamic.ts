import { render } from '@vue-email/render'
import type { Component } from 'vue'

import emails from '~/templates/emails'

export default defineEventHandler(async (event) => {
	try {
		const body = await readBody<{ title: string; body: string }>(event)

		if (!body.title || !body.body) {
			throw createError({
				statusCode: 400,
				message: 'Missing required fields: title and body',
			})
		}

		// Hacky, but cant explicitly import vue files in server side
		const component = (await emails['dynamic']()).default as Component | undefined

		if (!component) {
			throw createError({
				statusCode: 500,
				message: 'Failed to load email template component',
			})
		}

		const html = await render(component, {
			title: body.title,
			body: body.body,
		})

		return html
	} catch (error) {
		console.error('Error rendering dynamic email template:', error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render dynamic email template',
		})
	}
})
