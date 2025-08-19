<template>
	<div class="overlay"></div>
	<img
		src="https://cdn-raw.modrinth.com/medal-banner-background.webp"
		class="background-pattern dark-pattern shadow-xl"
		alt=""
	/>
	<img
		src="https://cdn-raw.modrinth.com/medal-banner-background-light.webp"
		class="background-pattern light-pattern shadow-xl"
		alt=""
	/>
</template>

<script setup lang="ts"></script>
<style scoped lang="scss">
.overlay {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background: var(--medal-promotion-bg-gradient);
	z-index: 1;
	border-radius: inherit;
}

.light-mode,
.light {
	.background-pattern.dark-pattern {
		display: none;
	}
	.background-pattern.light-pattern {
		display: block;
	}
}

.background-pattern.dark-pattern {
	display: block;
}

.background-pattern.light-pattern {
	display: none;
}

.background-pattern {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	z-index: 0;
	object-fit: cover;
	object-position: bottom;
	background-color: var(--medal-promotion-bg);
	border-radius: inherit;
	color: var(--medal-promotion-bg-orange);
}
</style>
