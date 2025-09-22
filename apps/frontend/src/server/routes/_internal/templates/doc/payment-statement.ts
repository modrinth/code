import { render } from '@vue-email/render'
import { getHeader, getMethod, readBody, setHeader, setResponseStatus } from 'h3'
import type { Component } from 'vue'

import PaymentStatement from '~/templates/docs/payouts/PaymentStatement.vue'

export default defineEventHandler(async (event) => {
	try {
		const method = getMethod(event)

		if (method === 'OPTIONS') {
			setHeader(event, 'Allow', 'POST, OPTIONS')
			setResponseStatus(event, 204)
			return ''
		}

		if (method !== 'POST') {
			setHeader(event, 'Allow', 'POST, OPTIONS')
			throw createError({ statusCode: 405, statusMessage: 'Method Not Allowed' })
		}

		// CSRF protection, only allow json
		const contentType = (getHeader(event, 'content-type') || '').toLowerCase()
		if (!contentType.startsWith('application/json')) {
			throw createError({ statusCode: 415, statusMessage: 'Unsupported Media Type' })
		}

		const props = (await readBody(event)) ?? {}

		const component = PaymentStatement as unknown as Component
		console.log(props)
		const html = await render(component, props, {
			pretty: true,
		})

		setHeader(event, 'Content-Type', 'text/html; charset=utf-8')
		setHeader(event, 'Cache-Control', 'no-store')

		return html
	} catch (error) {
		console.error('Error rendering document payment-statement:', error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render document template',
		})
	}
})
