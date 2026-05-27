import { CheckIcon, EyeIcon, RotateCounterClockwiseIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import SkinPreviewRenderer from '../../components/skin/SkinPreviewRenderer.vue'

const steveSkin =
	'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAEAAAABACAYAAACqaXHeAAAE5klEQVR4Xu1aMWsUQRgVVJCAoIIgglaJBG2UGIKCOU0hJHZKijRBsAnaWSjaiNjESgtTpT0bmxQWNvkJ+U9n3ube8ubNt7e5xOzdxX3wmNmZb/fmvflmdtm9M2dqcOfmxR44N325KFnn8ecXDwbSrzdxoODO7auleNTdgHfP5hKeKgNUsJff1zqZeBJ9fr2Jg4vW9Ef5Xxiga9+XAkRWLYFTYYDOuGYDxIMU6psf2r6tL54OA3wPUEIkBFeVfr2Jg4r2DRGk2OWZSwnZ7tcbO2hqU9j0tQtJyjMLlGqCnqPX0WtEcSh9PI0jGiBIIVpX8V7yvEg04/z6aPPxNA4V4QNm2/ar5d7vD+u9P1/eFOWvt2tF2/2ZK1ksyGO2aZ0msM3H0zg4IB2oziSEQjCEUzyJvkhUdE3WGc9jH0/j0IGxxC0M4iACQoHnj76WolEHcIwYtOGc6Fo0NDJoLAzwGaIgJWf89eO7BTUDlBTlM6/7gxqCYx9P4+BAUHLmeQ//8XKxfKDBre390nzBpRtTSQxvecwEvaYb4Mc+nsbBpzYV7yaghGiYwJKiaZCey+vpk+Gt61MJ2e7jGTke3vvaUy4sLBScnZ0t6PEZ9vbK5aHGog19fn2nXy7Dzk6vZLebLNHiN44LHxCF0wiPz7AvUsWrCSdhgO5H+B0PHxo+oKNkwEkbMLW7mxkwVhlwoktgX3S0BI6cAU/mt3sgfhzl04c/S7KNJan957a2EurMYFCl+P7Aq+LBpF2FiuCSbPv0KaWfUwcKPKwB2g9isOc3NwuWgvpCmZbeHsUnIr2N7S4ehOiNjQPSAI2rwyCBnh3eD0LI2Y8fC6Je/nC/9DaPdxPcrEwQ2T3ImNAAPacOgwRWGaB1iiHrBHl8IoyiVISnuJPi1QRlHVwgj6M27SNdUDLjZoDOfmRAtva7B5scyQ1Vy4h6juvN4Ab4DEeG6LGndGlAX4gb4PGeIaX4/rHe4nTH1zYVrfHF5luHyABfBoP6KYr0FNZZ9dgo3s+NRKkRkQFad70ZKKxKYJ0BGKSKKTJAZ5TZ0Bfk8aEBEh8ZoEaoASg9xvVm8PRW4UqPIylIxVG4i2OMxyUGiHjE6Ky6OBqgRniM682wsrLSA31zczLOqaIiM1Sgx5YGiGgVD6pYFxeZ4P2uNwMebzudTiJqdXU1E4oYj0Vdl4gapplDIyjMY3T5aAwyscoAzQw3QGNdbwY+41OcC6TwKA51NYDUZcSSM+rxvtcwhu0qxkWCfmv0GNfbokWLFi1atJgc4AVoFf3xmU+dSUyLFi1atGjRokVT8FdpQ39c1XeE+/S3QB4+dnADhv68bgbwTc+hX3GNGm7AUTOAj7qaARNpwLEyYL+uGTCWS0Df/vpLTu/TfsZkHzODbwS6JDKOGocRyWzwPjD5wqsGDDJB20YNNwClG6D9dQaUH0uqBLsho4YLdJHer22FAXXf9+uoWdI3xt8bVPGfGOjieRy1aR/p3/b9Q4d/8FCiz8fTOKoE+2xrv8arGL3f687vt8Kxui1GAuuWgNINGCRc+1j38TQOF4gyMiAyB4xS3YW7AUofT+Pw9PeZ1vZomURr3EUOygofT+Pgl2QKrKJ/bifrxFcJHxsD+LlcRQ3z/4LIABfsx9rm42kc0f8GVCCFR3GoQ0jVPuBZEfX5eIbFX3srPNN8aUvJAAAAAElFTkSuQmCC'

function svgDataUrl(svg: string) {
	return `data:image/svg+xml,${encodeURIComponent(svg)}`
}

const greenCape = svgDataUrl(
	'<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect width="64" height="32" fill="#1f8f2f"/><rect x="1" y="1" width="10" height="16" fill="#59d957"/><rect x="3" y="4" width="6" height="10" fill="#11551d"/></svg>',
)
const pinkCape = svgDataUrl(
	'<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32"><rect width="64" height="32" fill="#d94fa3"/><rect x="1" y="1" width="10" height="16" fill="#ff9bd4"/><rect x="4" y="3" width="3" height="12" fill="#fff0fa"/></svg>',
)

const meta = {
	title: 'Skin/SkinPreviewRenderer',
	component: SkinPreviewRenderer,
	args: {
		textureSrc: steveSkin,
		variant: 'CLASSIC',
		initialRotation: Math.PI / 8,
	},
	render: (args) => ({
		components: { CheckIcon, EyeIcon, RotateCounterClockwiseIcon, SkinPreviewRenderer },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="h-[80vh] min-h-[32rem] max-h-[48rem] w-[22rem]">
				<SkinPreviewRenderer v-bind="args" nametag="Steve">
					<template #nametag-badge>
						<div class="flex items-center justify-center gap-1.5 rounded-full border border-solid border-brand-blue bg-bg-blue px-3 py-1 text-base font-semibold leading-6 text-brand-blue">
							<EyeIcon class="size-5 shrink-0" /> Previewing
						</div>
					</template>
					<template #subtitle>
						<div class="flex items-center gap-2">
							<button class="flex h-10 items-center justify-center gap-2 rounded-[14px] border-0 bg-surface-4 px-4 py-2.5 text-base font-semibold leading-5 text-contrast shadow-md">
								<RotateCounterClockwiseIcon class="size-5" /> Reset
							</button>
							<button class="flex h-10 items-center justify-center gap-2 rounded-[14px] border-0 bg-brand px-4 py-2.5 text-base font-semibold leading-5 text-[rgba(0,0,0,0.9)] shadow-md">
								<CheckIcon class="size-5" /> Apply
							</button>
						</div>
					</template>
				</SkinPreviewRenderer>
			</div>
		`,
	}),
} satisfies Meta<typeof SkinPreviewRenderer>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const RepeatInteract: Story = {
	render: (args) => ({
		components: { SkinPreviewRenderer },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="h-[80vh] min-h-[32rem] max-h-[48rem] w-[22rem]">
				<SkinPreviewRenderer v-bind="args" />
			</div>
		`,
	}),
}

export const ResponsiveFit: Story = {
	args: {
		lockFit: false,
	},
}

export const LayerSeparation: Story = {
	args: {
		fitZoom: 1.5,
		framing: 'modal',
		initialRotation: Math.PI / 10,
	},
	render: (args) => ({
		components: { SkinPreviewRenderer },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="h-[28rem] w-[18rem]">
				<SkinPreviewRenderer v-bind="args" />
			</div>
		`,
	}),
}

export const FramingModes: Story = {
	render: () => ({
		components: { CheckIcon, SkinPreviewRenderer },
		setup() {
			return { steveSkin }
		},
		template: /* html */ `
			<div class="flex flex-wrap items-start gap-8">
				<div class="h-[32rem] w-[18rem]">
					<SkinPreviewRenderer :texture-src="steveSkin" variant="CLASSIC" framing="page" nametag="Steve">
						<template #subtitle>
							<button class="flex h-10 items-center justify-center gap-2 rounded-[14px] border-0 bg-brand px-4 py-2.5 text-base font-semibold leading-5 text-[rgba(0,0,0,0.9)] shadow-md">
								<CheckIcon class="size-5" /> Apply
							</button>
						</template>
					</SkinPreviewRenderer>
				</div>
				<div class="h-[25rem] w-[16rem]">
					<SkinPreviewRenderer :texture-src="steveSkin" variant="CLASSIC" framing="modal" />
				</div>
			</div>
		`,
	}),
}

export const CapeSwitching: Story = {
	render: () => ({
		components: { SkinPreviewRenderer },
		setup() {
			const capeSrc = ref<string | undefined>()
			return { capeSrc, greenCape, pinkCape, steveSkin }
		},
		template: /* html */ `
			<div class="flex items-start gap-6">
				<div class="h-[25rem] w-[16rem]">
					<SkinPreviewRenderer
						:texture-src="steveSkin"
						:cape-src="capeSrc"
						variant="CLASSIC"
						framing="modal"
						:initial-rotation="Math.PI / 8"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<button class="rounded-lg border-0 bg-surface-4 px-4 py-2 text-primary" @click="capeSrc = undefined">
						None
					</button>
					<button class="rounded-lg border-0 bg-surface-4 px-4 py-2 text-primary" @click="capeSrc = greenCape">
						Green cape
					</button>
					<button class="rounded-lg border-0 bg-surface-4 px-4 py-2 text-primary" @click="capeSrc = pinkCape">
						Pink cape
					</button>
				</div>
			</div>
		`,
	}),
}
