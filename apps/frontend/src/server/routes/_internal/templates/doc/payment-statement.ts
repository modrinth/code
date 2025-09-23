import { renderToBuffer } from '@ceereals/vue-pdf'
import { getHeader, getMethod, readBody, setHeader, setResponseStatus } from 'h3'
import { h, type Component } from 'vue'

import PaymentStatement from '~/templates/docs/payouts/PaymentStatement.vue'

export default defineEventHandler(async (event) => {
	try {
		const method = getMethod(event)

		if (method === 'OPTIONS') {
			setHeader(event, 'Allow', 'GET, POST, OPTIONS')
			setResponseStatus(event, 204)
			return ''
		}

		if (method !== 'POST' && method !== 'GET' && import.meta.dev) {
			setHeader(event, 'Allow', 'GET, POST, OPTIONS')
			throw createError({ statusCode: 405, statusMessage: 'Method Not Allowed' })
		}

		let props: any
		if (method === 'POST') {
			// CSRF protection, only allow json
			const contentType = (getHeader(event, 'content-type') || '').toLowerCase()
			if (!contentType.startsWith('application/json')) {
				throw createError({ statusCode: 415, statusMessage: 'Unsupported Media Type' })
			}

			props = (await readBody(event)) ?? {}
		} else {
			// For testing purposes.
			props = {
				payer: {
					name: 'Rinth, Inc.',
					address: '800 N King St, Suite 304 #3133, Wilmington, DE 19801',
					email: 'support@modrinth.com',
				},
				recipient: {
					name: 'Alex Doe',
					email: 'alex@example.com',
					address: '123 Main St, Anytown, CA 94000',
				},
				payment: {
					id: 'PS-2025-09-001',
					date: '2025-09-02',
					currency: 'USD',
					gross: 135.79,
					fees: 1.5,
					net: 134.29,
				},
			}
		}

		const component = PaymentStatement as unknown as Component
		console.log(props)

		// registerPdfFonts()

		const vnode = h(component, props)
		const buffer = await renderToBuffer(vnode)

		const fileNameBase =
			(typeof (props as any)?.payment?.id === 'string' && (props as any).payment.id.trim()) ||
			'payment-statement'

		setHeader(event, 'Content-Disposition', `inline; filename="${fileNameBase}.pdf"`)
		setHeader(event, 'Content-Type', 'application/pdf')
		setHeader(event, 'Cache-Control', 'no-store')

		return buffer
	} catch (error) {
		console.error('Error rendering document payment-statement:', error)
		throw createError({
			statusCode: 500,
			message: 'Failed to render document template',
		})
	}
})
