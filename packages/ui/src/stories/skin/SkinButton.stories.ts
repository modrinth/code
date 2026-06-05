import { EditIcon, TrashIcon } from '@modrinth/assets'
import type { Meta, StoryObj } from '@storybook/vue3-vite'

import ButtonStyled from '../../components/base/ButtonStyled.vue'
import SkinButton from '../../components/skin/SkinButton.vue'

const frontImage = `data:image/svg+xml,${encodeURIComponent(`
<svg width="114" height="176" viewBox="0 0 114 176" fill="none" xmlns="http://www.w3.org/2000/svg">
	<rect x="43" y="4" width="28" height="30" fill="#5A3828"/>
	<rect x="36" y="18" width="42" height="38" fill="#B97A57"/>
	<rect x="43" y="28" width="8" height="6" fill="#1B1B20"/>
	<rect x="63" y="28" width="8" height="6" fill="#1B1B20"/>
	<rect x="50" y="40" width="14" height="5" fill="#684432"/>
	<rect x="28" y="57" width="58" height="60" fill="#2693C7"/>
	<rect x="18" y="63" width="18" height="54" fill="#35A8D8"/>
	<rect x="78" y="63" width="18" height="54" fill="#1E7FAC"/>
	<rect x="22" y="117" width="14" height="43" fill="#B97A57"/>
	<rect x="78" y="117" width="14" height="43" fill="#A96A4D"/>
	<rect x="36" y="117" width="18" height="55" fill="#263F5E"/>
	<rect x="60" y="117" width="18" height="55" fill="#1D334F"/>
</svg>
)}`)}`

const backImage = `data:image/svg+xml,${encodeURIComponent(`
<svg width="114" height="176" viewBox="0 0 114 176" fill="none" xmlns="http://www.w3.org/2000/svg">
	<rect x="36" y="10" width="42" height="46" fill="#5A3828"/>
	<rect x="28" y="57" width="58" height="60" fill="#1E7FAC"/>
	<rect x="18" y="63" width="18" height="54" fill="#2693C7"/>
	<rect x="78" y="63" width="18" height="54" fill="#19759E"/>
	<rect x="22" y="117" width="14" height="43" fill="#A96A4D"/>
	<rect x="78" y="117" width="14" height="43" fill="#A96A4D"/>
	<rect x="36" y="117" width="18" height="55" fill="#1D334F"/>
	<rect x="60" y="117" width="18" height="55" fill="#162B45"/>
</svg>
)}`)}`

const meta = {
	title: 'Skin/SkinButton',
	component: SkinButton,
	args: {
		forwardImageSrc: frontImage,
		backwardImageSrc: backImage,
		selected: false,
		active: false,
		tooltip: 'Steve',
	},
	render: (args) => ({
		components: { SkinButton },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="w-[156px]">
				<SkinButton v-bind="args" />
			</div>
		`,
	}),
} satisfies Meta<typeof SkinButton>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const Selected: Story = {
	args: {
		selected: true,
	},
}

export const ActiveUnselected: Story = {
	args: {
		active: true,
	},
}

export const WithActions: Story = {
	render: (args) => ({
		components: { ButtonStyled, EditIcon, SkinButton, TrashIcon },
		setup() {
			return { args }
		},
		template: /* html */ `
			<div class="w-[156px]">
				<SkinButton v-bind="args">
					<template #overlay-buttons>
						<ButtonStyled color="brand">
							<button class="pointer-events-auto">
								<EditIcon /> Edit
							</button>
						</ButtonStyled>
						<ButtonStyled circular color="red">
							<button class="pointer-events-auto" aria-label="Delete skin">
								<TrashIcon />
							</button>
						</ButtonStyled>
					</template>
				</SkinButton>
			</div>
		`,
	}),
}
