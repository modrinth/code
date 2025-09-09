<!-- EmailCardSlots.vue -->
<script setup lang="ts">
import {
	Button,
	Column,
	Container,
	Head,
	Heading,
	Html,
	Img,
	Row,
	Section,
	Tailwind,
	Text,
	Link as VLink,
} from '@vue-email/components'

const props = defineProps<{
	link?: string
	buttonText?: string
}>()
</script>

<template>
	<Html lang="en">
		<Head>
			<title><slot name="title" /></title>
			<meta http-equiv="X-UA-Compatible" content="IE=edge" />
			<meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
			<meta name="viewport" content="width=device-width, initial-scale=1" />
			<link
				href="https://fonts.googleapis.com/css?family=Inter:700,400"
				rel="stylesheet"
				type="text/css"
			/>
		</Head>

		<Tailwind
			:config="{
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
						},
						fontFamily: {
							sans: ['Inter', 'Arial', 'sans-serif'],
						},
					},
				},
			}"
		>
			<div class="font-sans">
				<Section class="bg-bg px-4 py-4">
					<Container class="max-w-[600px]">
						<Row>
							<Column>
								<Img
									src="https://cdn.modrinth.com/email/f740e2decee8764a4629bff677a284f9.png"
									width="29"
									alt=""
									class="block h-auto"
								/>
							</Column>
						</Row>
					</Container>
				</Section>

				<Section class="bg-white px-8 py-8">
					<Container class="max-w-[600px]">
						<Heading as="h1" class="mb-2 text-2xl font-bold">
							<slot name="title" />
						</Heading>

						<Text class="text-muted mb-2 text-base">
							<slot name="subtitle" />
						</Text>

						<div class="text-muted text-base">
							<slot />
						</div>

						<div v-if="link && buttonText" class="mt-4">
							<Button
								:href="link"
								target="_blank"
								class="text-accentContrast inline-block rounded-[12px] bg-brand px-4 py-3 text-[14px] font-bold"
							>
								{{ buttonText }}
							</Button>

							<VLink :href="link">
								<Text class="text-muted mt-2 break-words text-xs font-bold">
									{{ link }}
								</Text>
							</VLink>
						</div>
					</Container>
				</Section>

				<Section class="bg-bg px-4 py-4">
					<Container class="max-w-[600px]">
						<Row>
							<Column class="align-middle">
								<VLink href="https://modrinth.com" aria-label="Modrinth">
									<Img
										src="https://cdn.modrinth.com/email/bd3357dfae4b1d266250372db3a0988f.png"
										width="102"
										alt="modrinth logo"
										class="block h-auto"
									/>
								</VLink>

								<Text class="mt-2 text-xs" :style="{ color: '#4d4d4d' }"> Rinth, Inc. </Text>

								<div class="mt-2">
									<Text class="text-xs text-secondary">410 N Scottsdale Road</Text>
									<Text class="text-xs text-secondary">Suite 1000</Text>
									<Text class="text-xs text-secondary">Tempe, AZ 85281</Text>
								</div>
							</Column>

							<Column class="text-right align-middle">
								<div class="inline-block">
									<VLink href="https://discord.modrinth.com" class="mr-4 inline-block">
										<Img
											width="20"
											height="20"
											alt="Discord"
											src="https://cdn.modrinth.com/email/e089a3a07be91c2940beff1fb191b247.png"
										/>
									</VLink>
									<VLink href="https://twitter.com/modrinth" class="mr-4 inline-block">
										<Img
											width="20"
											height="20"
											alt="Twitter/X"
											src="https://cdn.modrinth.com/email/363985aad91cab53854276e12f267b0b.png"
										/>
									</VLink>
									<VLink href="https://floss.social/@modrinth" class="mr-4 inline-block">
										<Img
											width="20"
											height="20"
											alt="Mastodon"
											src="https://cdn.modrinth.com/email/e25c30d5707744f31dcf18651a67d3d5.png"
										/>
									</VLink>
									<VLink href="https://github.com/modrinth/" class="mr-4 inline-block">
										<Img
											width="20"
											height="20"
											alt="GitHub"
											src="https://cdn.modrinth.com/email/45993023966d64e6138fd65a530a5d03.png"
										/>
									</VLink>
									<VLink href="https://www.youtube.com/@modrinth" class="inline-block">
										<Img
											width="20"
											height="20"
											alt="YouTube"
											src="https://cdn.modrinth.com/email/a15afae9fc94e105caeb1bb4d33a0a13.png"
										/>
									</VLink>
								</div>
							</Column>
						</Row>
					</Container>
				</Section>
			</div>
		</Tailwind>
	</Html>
</template>
