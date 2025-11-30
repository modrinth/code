<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
	defineProps<{
		progress: number
		max?: number
		color?: 'brand' | 'green' | 'red' | 'orange' | 'blue' | 'purple' | 'gray'
		waiting?: boolean
		fullWidth?: boolean
		striped?: boolean
		gradientBorder?: boolean
	}>(),
	{
		max: 1,
		color: 'brand',
		waiting: false,
		fullWidth: false,
		striped: false,
		gradientBorder: true,
	},
)

const colors = {
	brand: {
		fg: 'bg-brand',
		bg: 'bg-brand-highlight',
	},
	green: {
		fg: 'bg-green',
		bg: 'bg-bg-green',
	},
	red: {
		fg: 'bg-red',
		bg: 'bg-bg-red',
	},
	orange: {
		fg: 'bg-orange',
		bg: 'bg-bg-orange',
	},
	blue: {
		fg: 'bg-blue',
		bg: 'bg-bg-blue',
	},
	purple: {
		fg: 'bg-purple',
		bg: 'bg-bg-purple',
	},
	gray: {
		fg: 'bg-gray',
		bg: 'bg-bg-gray',
	},
}

const percent = computed(() => props.progress / props.max)
</script>
<template>
	<div
		class="flex w-full h-2 rounded-full overflow-hidden"
		:class="[colors[props.color].bg, fullWidth ? '' : 'max-w-[15rem]']"
	>
		<div
			class="rounded-full progress-bar"
			:class="[
				colors[props.color].fg,
				{ 'progress-bar--waiting': waiting },
				{ 'progress-bar--gradient-border': gradientBorder },
				striped ? `progress-bar--striped--${color}` : '',
			]"
			:style="!waiting ? { width: `${percent * 100}%` } : {}"
		></div>
	</div>
</template>
<style scoped lang="scss">
.progress-bar {
	transition: width 0.2s ease-in-out;
}

.progress-bar--waiting {
	animation: progress-bar-waiting 1s linear infinite;
	position: relative;
}

@keyframes progress-bar-waiting {
	0% {
		left: -50%;
		width: 20%;
	}
	50% {
		width: 60%;
	}
	100% {
		left: 100%;
		width: 20%;
	}
}

.progress-bar--gradient-border {
	position: relative;

	&::after {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(to bottom, rgba(255, 255, 255, 0.3), transparent);
		border-radius: inherit;
		mask:
			linear-gradient(#fff 0 0) content-box,
			linear-gradient(#fff 0 0);
		mask-composite: xor;
		padding: 2px;
		pointer-events: none;
	}
}

%progress-bar--striped-common {
	background-attachment: scroll;
	background-position: 0 0;
	background-size: 9.38px 9.38px;
}

@mixin striped-background($color-variable) {
	background-image: linear-gradient(
		135deg,
		$color-variable 11.54%,
		transparent 11.54%,
		transparent 50%,
		$color-variable 50%,
		$color-variable 61.54%,
		transparent 61.54%,
		transparent 100%
	);
}

.progress-bar--striped--brand {
	@include striped-background(var(--color-brand));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--green {
	@include striped-background(var(--color-green));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--red {
	@include striped-background(var(--color-red));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--orange {
	@include striped-background(var(--color-orange));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--blue {
	@include striped-background(var(--color-blue));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--purple {
	@include striped-background(var(--color-purple));
	@extend %progress-bar--striped-common;
}

.progress-bar--striped--gray {
	@include striped-background(var(--color-divider-dark));
	@extend %progress-bar--striped-common;
}
</style>
