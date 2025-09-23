import type { Style } from '@ceereals/vue-pdf'

export const docStyles: Record<string, Style> = {
	page: {
		paddingTop: 24,
		paddingRight: 24,
		paddingBottom: 32,
		paddingLeft: 24,
		// fontFamily: 'Inter',
	},
	header: { marginBottom: 16 },
	logo: { width: 175 },
	title: { fontSize: 24, fontWeight: 700, marginTop: 8 },
	hr: { height: 1, backgroundColor: '#e5e7eb', marginTop: 12, marginBottom: 12 },
	section: { marginBottom: 16 },
	sectionTitle: { fontSize: 14, fontWeight: 600, marginBottom: 6 },
	row: { flexDirection: 'row' as const },
	col: { flex: 1 },
	colLeft: { paddingRight: 8 },
	colRight: { paddingLeft: 8 },
	text: { fontSize: 12, lineHeight: 1.4 },
	textMuted: { fontSize: 12, color: '#6b7280' },
} as const
