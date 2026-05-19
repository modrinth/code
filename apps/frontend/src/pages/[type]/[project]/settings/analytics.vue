<template>
	<div>
		<div class="universal-card">
			<h2>Analytics</h2>

			<p>
				This page shows you the analytics for your project,
				<strong>{{ project.title }}</strong
				>. You can see the number of downloads, page views and revenue earned for your project, as
				well as the total downloads and page views for {{ project.title }} by country.
			</p>
		</div>

		<ChartDisplay :projects="[project]" />
	</div>
</template>

<script setup>
import { injectProjectPageContext } from '@modrinth/ui'

import ChartDisplay from '~/components/ui/charts/ChartDisplay.vue'

const { projectV2: project } = injectProjectPageContext()
</script>

<style scoped lang="scss">
.markdown-body {
	margin-bottom: var(--gap-md);
}
</style>
