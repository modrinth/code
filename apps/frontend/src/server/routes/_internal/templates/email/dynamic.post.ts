import { render } from '@vue-email/render'

import MarkdownDynamicEmail from '~/templates/emails/dynamic/MarkdownDynamicEmail.vue'

export default defineEventHandler(async (event) => {
	try {
		const body = await readBody<{ title: string; body: string }>(event)

		if (!body.title || !body.body) {
			throw createError({
				statusCode: 400,
				message: 'Missing required fields: title and body',
			})
		}

		const html = await render(MarkdownDynamicEmail, {
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
