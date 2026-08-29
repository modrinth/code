import assert from 'node:assert/strict'
import test from 'node:test'

import { francAll } from 'franc-min'

import {
	MIN_ENGLISH_SCORE,
	MIN_LANGUAGE_DETECTION_CHARACTERS,
	MIN_LANGUAGE_DETECTION_WORDS,
	validateEnglishSummaryText,
	validateEnglishText,
} from './index.ts'

test('accepts text when franc scores English above the minimum score', () => {
	const text =
		'This project adds configurable caves, useful tools, and polished world generation for every player.'
	const result = validateEnglishText(text)
	const english = result.detections.find(({ language }) => language === 'eng')
	const alternative = result.detections.find(({ language }) => language !== 'eng')

	assert.equal(result.valid, true)
	assert.ok(english)
	assert.ok(alternative)
	assert.ok(english.accuracy > MIN_ENGLISH_SCORE)
	assert.deepEqual(
		result.detections,
		francAll(text).map(([language, accuracy]) => ({ language, accuracy })),
	)
})

test('accepts mixed English and Chinese text above the minimum score', () => {
	const result = validateEnglishText(
		'A super light QQ bot for minecraft server and QQ group exchange msgs | 超轻量的QQ-MC群服插件',
	)

	assert.equal(result.valid, true)
})

test('rejects text when franc scores English below the minimum score', () => {
	for (const text of [
		'Чистый модпак для комфортной игры с друзьями, новыми заданиями и значительно улучшенной производительностью.',
		'これは新しい洞窟と構造物を追加し、すべてのプレイヤーの世界生成を改善するプロジェクトです。',
	]) {
		assert.equal(validateEnglishText(text).valid, false, text)
	}
})

test('validates production project text using the English score threshold', () => {
	const cases = [
		{
			text: 'This mod adds some new things about turtles to Minecraft这个模组为Minecraft增加了一些关于乌龟的新东西',
			valid: true,
		},
		{
			text: 'Tenhle Project má super mody ktere zlepší kvalitu hraní PVP',
			valid: true,
		},
		{
			text: '此插件修复了Authme在lophine服务端上的登录漏洞 修复了玩家退出时SQL数据库的Logged依然为1的问题',
			valid: false,
		},
		{
			text: 'Мод добавляет рубин — новый драгоценный камень. Добывайте рубиновую руду, кристаллизующуюся в толще камня,  а закалив четыре рубина четырьмя незеритовыми ломами - можно будет сделать меч, кирку, броню и крюк захвата',
			valid: false,
		},
		{
			text: 'Um modpack Fabric focado em desempenho, imersão e exploração, mantendo a experiência próxima ao Minecraft Vanilla.  O objetivo é melhorar o visual, os sons, a geração de mundo e a qualidade de vida do jogo sem adicionar sistemas complexos.',
			valid: true,
		},
		{
			text: 'A Create Tacz Warfare Modpack for the Server Create: Warfare',
			valid: true,
		},
		{
			text: '一个集成了全息字和占位符创建的插件 A Plugin Integrating Holograms and Placeholder Support',
			valid: true,
		},
		{
			text: "A modpack that adds stuff from TaCZ guns, to shaders, to curios slots, and even create! And also, Superb Warfare, in case TaCZ isn't for you!",
			valid: true,
		},
		{
			text: 'Leichtes Client-Modpack für entspannte Feierabend-Sessions, bessere Performance, praktische QoL-Mods und ein aufgeräumtes Spielgefühl ohne unnötigen Ballast.',
			valid: true,
		},
		{
			text: 'A super light QQ bot for minecraft server and QQ group exchange msgs | 超轻量的QQ-MC群服插件',
			valid: true,
		},
		{
			text: 'Integrates MCP into minecraft, made for mapmakers and complex command block logic and datapack making.',
			valid: true,
		},
	]

	for (const { text, valid } of cases) {
		assert.equal(validateEnglishText(text).valid, valid, text)
	}
})

test('skips language detection for production text below the word minimum', () => {
	for (const text of [
		'You can chat Gemini AI in Minecraft',
		'Create Mods X Zombie Apolcalypse',
		'Open-world zombie survival modpack',
		"BIG-GOOSE Minecraft server's modpack",
	]) {
		assert.deepEqual(validateEnglishText(text), { valid: true, detections: [] }, text)
	}
})

test('skips language detection for text below the character minimum', () => {
	const result = validateEnglishText('one two three four a b c d')

	assert.deepEqual(result, { valid: true, detections: [] })
	assert.equal(MIN_LANGUAGE_DETECTION_CHARACTERS, 35)
})

test('validates production text meeting both signal minimums', () => {
	const text = '𝗔𝗶𝗺𝗶𝗻𝗴 𝘁𝗼 𝗲𝗻𝗵𝗮𝗻𝗰𝗲 𝗠𝗶𝗻𝗲𝗰𝗿𝗮𝗳𝘁 𝘄𝗵𝗶𝗹𝗲 𝗿𝗲𝘁𝗮𝗶𝗻𝗶𝗻𝗴 𝘁𝗵𝗮𝘁 𝗩𝗮𝗻𝗶𝗹𝗹𝗮 𝗳𝗲𝗲𝗹!'
	const result = validateEnglishText(text)

	assert.equal(result.valid, false)
	assert.ok(result.detections.length > 0)
	assert.equal(MIN_LANGUAGE_DETECTION_WORDS, 8)
})

test('uses the same validation for summaries', () => {
	assert.equal(validateEnglishSummaryText, validateEnglishText)
})

test('allows empty text to be handled by required-field validation', () => {
	assert.deepEqual(validateEnglishText('  '), { valid: true, detections: [] })
})
