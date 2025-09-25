<template>
	<div class="flex flex-col gap-4">
		<p>Muralpay Details Stage</p>
		<!-- TBD: Implement bank account or crypto wallet details form -->
		<!-- Show different fields based on selected method (bank transfer vs crypto) -->
		<!-- Reference: https://www.figma.com/design/9bI55sqgicwUezmRXBo9bd/Design-System?node-id=295-14876&m=dev -->
	</div>
</template>

<script setup lang="ts">
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const withdrawContext = useWithdrawContext()

// TBD: Implement bank account details form (for bank transfer method)
// TBD: Implement crypto wallet details form (for crypto method)
// TBD: Update withdrawContext.withdrawData.accountDetails with form data
</script>
