<script setup lang="ts">
import { Document, Image, Page, Text, View } from '@ceereals/vue-pdf'
import { docStyles as styles } from './pdfStyles'

const props = withDefaults(
	defineProps<{
		title?: string
		size?: string | number | number[] | { width?: number; height?: number; [key: string]: unknown }
		showLogo?: boolean
	}>(),
	{
		title: 'Document',
		size: 'A4',
		showLogo: true,
	},
)

defineSlots<{
	default: (props: { styles: typeof styles }) => any
	header: (props: { styles: typeof styles }) => any
}>()
</script>

<template>
	<Document :title="props.title">
		<Page :size="props.size" :style="styles.page">
			<slot name="header" :styles="styles">
				<View :style="styles.header">
					<Image
						v-if="props.showLogo"
						:style="styles.logo"
						src="https://cdn.modrinth.com/email/bd3357dfae4b1d266250372db3a0988f.png"
					/>
					<Text :style="styles.title">{{ props.title }}</Text>
					<View :style="styles.hr" />
				</View>
			</slot>

			<slot :styles="styles" />
		</Page>
	</Document>
</template>
