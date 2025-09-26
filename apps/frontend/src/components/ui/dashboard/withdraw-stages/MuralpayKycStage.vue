<template>
	<div class="flex flex-col gap-4">
		<p>Muralpay KYC Stage</p>
		<!-- TBD: Implement KYC form fields for Muralpay -->
		<!-- Fields: Full Name, Address, Phone Number, etc. -->
		<!-- Reference: https://www.figma.com/design/9bI55sqgicwUezmRXBo9bd/Design-System?node-id=522-2998&m=dev -->
	</div>
</template>

<script setup lang="ts">
import { useWithdrawContext } from '@/providers/creator-withdraw.ts'

const withdrawContext = useWithdrawContext()

// TBD: Implement KYC form fields
// TBD: Implement form validation
// TBD: Update withdrawContext.withdrawData.kycData with form data
</script>
