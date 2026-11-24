const VIDEO_ID = /^[a-zA-Z0-9_-]{11}$/
const TIME_PARAMS = ['start', 'end'] as const
const FRAME_ANCESTORS = [
	'tauri://localhost',
	'http://tauri.localhost',
	'https://tauri.localhost',
	'http://localhost:1420',
]

export default defineEventHandler((event) => {
	const id = getRouterParam(event, 'id') ?? ''
	if (!VIDEO_ID.test(id)) {
		throw createError({ statusCode: 404, message: 'Video not found' })
	}

	const query = getQuery(event)
	const params = new URLSearchParams()
	for (const key of TIME_PARAMS) {
		const value = String(query[key] ?? '')
		if (/^\d+$/.test(value)) {
			params.set(key, value)
		}
	}

	const search = params.size > 0 ? `?${params}` : ''
	const src = `https://www.youtube-nocookie.com/embed/${id}${search}`

	setResponseHeaders(event, {
		'content-type': 'text/html; charset=utf-8',
		'content-security-policy': [
			"default-src 'none'",
			"style-src 'unsafe-inline'",
			'frame-src https://www.youtube-nocookie.com',
			`frame-ancestors ${FRAME_ANCESTORS.join(' ')}`,
		].join('; '),
		'referrer-policy': 'strict-origin-when-cross-origin',
		'cache-control': 'public, max-age=86400',
	})

	return `<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1" />
		<title>YouTube video</title>
		<style>
			html,
			body,
			iframe {
				margin: 0;
				width: 100%;
				height: 100%;
				border: 0;
				background: #000;
				overflow: hidden;
			}
		</style>
	</head>
	<body>
		<iframe
			src="${src}"
			title="YouTube video player"
			allow="autoplay; encrypted-media; picture-in-picture; fullscreen"
			allowfullscreen
			referrerpolicy="strict-origin-when-cross-origin"
		></iframe>
	</body>
</html>`
})
