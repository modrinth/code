import { defineMessages } from '@modrinth/ui'

import duskBlock from '@/assets/instance-icons/dusk-block.png'
import duskBlockGold from '@/assets/instance-icons/dusk-block-gold-test.png'
import duskBlockRose from '@/assets/instance-icons/dusk-block-rose-test.png'
import duskBlockTeal from '@/assets/instance-icons/dusk-block-teal-test.png'

const names = defineMessages({
	surface: {
		id: 'instance.icon-editor.background.surface',
		defaultMessage: 'Surface',
	},
	green: {
		id: 'instance.icon-editor.background.green',
		defaultMessage: 'Green',
	},
	purple: {
		id: 'instance.icon-editor.background.purple',
		defaultMessage: 'Purple',
	},
	blue: {
		id: 'instance.icon-editor.background.blue',
		defaultMessage: 'Blue',
	},
	orange: {
		id: 'instance.icon-editor.background.orange',
		defaultMessage: 'Orange',
	},
	red: {
		id: 'instance.icon-editor.background.red',
		defaultMessage: 'Red',
	},
	duskBlock: {
		id: 'instance.icon-editor.symbol.dusk-block',
		defaultMessage: 'Dusk block',
	},
	duskBlockRose: {
		id: 'instance.icon-editor.symbol.dusk-block-rose',
		defaultMessage: 'Rose dusk block',
	},
	duskBlockGold: {
		id: 'instance.icon-editor.symbol.dusk-block-gold',
		defaultMessage: 'Gold dusk block',
	},
	duskBlockTeal: {
		id: 'instance.icon-editor.symbol.dusk-block-teal',
		defaultMessage: 'Teal dusk block',
	},
})

export const backgroundOptions = [
	{ background: { type: 'color', value: '#34363c' }, name: names.surface },
	{ background: { type: 'color', value: '#1bd96a' }, name: names.green },
	{ background: { type: 'color', value: '#c78aff' }, name: names.purple },
	{ background: { type: 'color', value: '#4f9cff' }, name: names.blue },
	{ background: { type: 'color', value: '#ffa347' }, name: names.orange },
	{ background: { type: 'color', value: '#ff496e' }, name: names.red },
] as const

export const symbolOptions = [
	{ id: 'dusk_block', name: names.duskBlock, asset: duskBlock },
	{ id: 'dusk_block_rose_test', name: names.duskBlockRose, asset: duskBlockRose },
	{ id: 'dusk_block_gold_test', name: names.duskBlockGold, asset: duskBlockGold },
	{ id: 'dusk_block_teal_test', name: names.duskBlockTeal, asset: duskBlockTeal },
] as const

export type BackgroundColor = (typeof backgroundOptions)[number]['background']['value']
export type SymbolId = (typeof symbolOptions)[number]['id']

export const DEFAULT_BACKGROUND_COLOR = '#c78aff' satisfies BackgroundColor
export const DEFAULT_SYMBOL_ID = 'dusk_block' satisfies SymbolId
