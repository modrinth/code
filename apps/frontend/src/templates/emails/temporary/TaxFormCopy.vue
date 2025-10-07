<script setup lang="ts">
import { Heading, Text, Link as VLink } from '@vue-email/components';

import StyledEmail from '../shared/StyledEmail.vue';
</script>

<template>
	<StyledEmail title="Your { taxform.type } tax form is available to download" :manual-links="[
		{ link: 'https://modrinth.com/dashboard/revenue', label: 'Revenue dashboard' },
		{ link: 'https://support.modrinth.com', label: 'Support Portal' },
	]">
		<Heading as="h1" class="mb-2 text-2xl font-bold">
			Your { taxform.type } tax form is available to download
		</Heading>

		<Text class="text-base">Hi { user.name },</Text>

		<Text class="text-base">
			We recently introduced a way for creators to download their tax forms after they've filled
			them out on the Modrinth revenue dashboard.
		</Text>

		<Text class="text-base">
			Since you filled out your form before this was available, we are emailing you with a copy of
			it.
		</Text>

		<Text class="text-base">
			Attached to this email you can find the PDF copy of your { taxform.type } form.
		</Text>

		<Text class="text-base">
			If you have any questions about the creator rewards program, please contact support through
			the <VLink href="https://support.modrinth.com" class="text-green underline">Support Portal</VLink>.
		</Text>

		<Text class="text-base">Thanks for publishing to Modrinth!</Text>
	</StyledEmail>
</template>
