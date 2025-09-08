import { render } from '@vue-email/render'
import fs from 'fs'
import path from 'path'

export default defineEventHandler(async (event) => {
	console.log('Email route hit:', event.node.req.url)

	const template = getRouterParam(event, 'template')?.replace('.html', '')
	console.log('Template param:', template)

	if (!template) {
		throw createError({
			statusCode: 404,
			message: 'Email template not found',
		})
	}

	try {
		const emailPath = path.resolve(`./src/emails/${template}.vue`)
		console.log('Looking for email at:', emailPath)

		if (!fs.existsSync(emailPath)) {
			console.log('Email file not found at:', emailPath)
			throw createError({
				statusCode: 404,
				message: `Template ${template} not found`,
			})
		}

		console.log('Importing component...')
		// Fix the import path - use absolute path from the project root
		const component = await import(
			path.resolve(process.cwd(), 'apps/frontend/src/emails', `${template}.vue`)
		)
		console.log('Component imported:', !!component.default)

		const html = await render(component.default, {})
		console.log('Rendered HTML length:', html.length)

		return html
	} catch (error) {
		console.error(`Error rendering email template ${template}:`, error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render email template',
		})
	}
})
