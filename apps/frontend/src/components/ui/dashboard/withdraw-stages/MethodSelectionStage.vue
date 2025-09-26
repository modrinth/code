<template>
	<div class="flex flex-col gap-4">
		<p>Method Selection Stage - Region and Payment Provider Selection</p>
		<!-- TBD: Implement region selection and payment provider selection based on Figma design -->
		<!-- This stage should allow users to select their country/region and then show available payment providers -->
		<!-- Payment providers: Tremendous (gift card, PayPal, Venmo) and Muralpay (bank transfer, crypto) -->
	</div>
</template>

<script setup lang="ts">
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const withdrawContext = useWithdrawContext()

// TBD: Implement region selection logic
// TBD: Implement payment provider filtering based on selected region
// TBD: Update withdrawContext.withdrawData with selections
</script>
