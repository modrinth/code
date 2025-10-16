<script setup lang="ts">
import { Body, Head, Html, Section, Style, Tailwind } from '@vue-email/components'
import type { Config } from 'tailwindcss'

defineProps<{
	title?: string
}>()

const tailwindConfig = {
	theme: {
		extend: {
			colors: {
				bg: { DEFAULT: '#ebebeb', raised: '#ffffff', super: '#e9e9e9' },
				divider: { DEFAULT: '#babfc5', dark: '#c8cdd3' },
				base: '#2c2e31',
				secondary: '#484d54',
				contrast: '#1a202c',
				accentContrast: '#ffffff',
				red: '#cb2245',
				orange: '#e08325',
				green: '#00af5c',
				blue: '#1f68c0',
				purple: '#8e32f3',
				gray: '#595b61',
				brand: {
					DEFAULT: '#00af5c',
					highlight: 'rgba(0, 175, 92, 0.25)',
					shadow: 'rgba(0, 175, 92, 0.7)',
				},
				highlight: {
					red: 'rgba(203, 34, 69, 0.25)',
					orange: 'rgba(224, 131, 37, 0.25)',
					green: 'rgba(0, 175, 92, 0.25)',
					blue: 'rgba(31, 104, 192, 0.25)',
					purple: 'rgba(142, 50, 243, 0.25)',
					gray: 'rgba(89, 91, 97, 0.25)',
				},
				tint: {
					red: 'rgba(203, 34, 69, 0.1)',
					orange: 'rgba(224, 131, 37, 0.1)',
					green: 'rgba(0, 175, 92, 0.1)',
					blue: 'rgba(31, 104, 192, 0.1)',
					purple: 'rgba(142, 50, 243, 0.1)',
				},
				link: { DEFAULT: '#1f68c0', hover: '#1f68c0', active: '#1f68c0' },
				muted: '#777777',
				footerText: '#4d4d4d',
			},
			fontSize: {
				base: ['16px', { lineHeight: '24px' }],
				'2xl': ['28px', { lineHeight: '32px' }],
				xs: ['13px', { lineHeight: '16px' }],
				'2xs': ['11px', { lineHeight: '16px' }],
			},
			fontFamily: {
				sans: ['Inter', 'Arial', 'sans-serif'],
			},
		},
	},
} as Partial<Config>
</script>

<template>
	<Html lang="en">
		<Head>
			<title>{{ title }}</title>
			<meta http-equiv="X-UA-Compatible" content="IE=edge" />
			<meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
			<meta name="viewport" content="width=device-width, initial-scale=1" />
			<link
				href="https://fonts.googleapis.com/css?family=Inter:700,400"
				rel="stylesheet"
				type="text/css"
			/>
			<Style>
				/* Outlook.com centering + line-height fixes */ .ExternalClass { width:100%; }
				.ExternalClass p, .ExternalClass span, .ExternalClass font, .ExternalClass td {
				line-height:100%; } table { border-collapse:separate; } a, a:link, a:visited {
				text-decoration:none; color:#1f68c0; } a:hover { text-decoration:underline; }
				h1,h2,h3,h4,h5,h6 { color:#000 !important; margin:0; mso-line-height-rule:exactly; }
			</Style>
		</Head>

		<Body>
			<Tailwind :config="tailwindConfig">
				<Section class="m-0 pb-0 pl-0 pr-0 pt-0 font-sans">
					<slot />
				</Section>
			</Tailwind>
		</Body>
	</Html>
</template>
