<template>
	<div class="flex flex-col gap-4">
		<div class="flex flex-col gap-2.5">
			<span class="text-contrast font-semibold">Region
				<UnknownIcon
v-tooltip="'Some payout methods are not available in certain regions.'"
					class="size-3 text-secondary" />
			</span>
		</div>
	</div>
</template>

<script setup lang="ts">
import { UnknownIcon } from '@modrinth/assets';

import { useWithdrawContext } from '@/providers/creator-withdraw.ts';

const withdrawContext = useWithdrawContext()

// TBD: Implement region selection logic
// TBD: Implement payment provider filtering based on selected region
// TBD: Update withdrawContext.withdrawData with selections
</script>
