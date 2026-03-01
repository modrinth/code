<template>
	<div class="grid grid-cols-[min-content_1fr_auto] gap-4">
		<slot name="icon" />
		<div class="flex flex-col gap-1.5">
			<slot name="title" />
			<slot name="summary" />
			<slot name="stats" />
		</div>
		<slot name="actions" />
	</div>
</template>
