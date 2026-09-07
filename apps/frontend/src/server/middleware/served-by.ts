import { readEnv } from '~/helpers/env'

const servedBy = Promise.all([readEnv('BUNNYNET_MC_PODID'), readEnv('BUNNYNET_MC_REGION')])
	.then((parts) => parts.filter(Boolean).join('-'))
	.catch(() => '')

export default defineEventHandler(async (event) => {
	const pod = await servedBy
	if (!pod) return

	setResponseHeader(event, 'Served-By', pod)
})
