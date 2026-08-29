import assert from 'node:assert/strict'
import test from 'node:test'

import { francAll } from 'franc-min'

import {
	analyzeLanguageChunks,
	LANGUAGE_CHUNK_STRIDE_WORDS,
	LANGUAGE_CHUNK_WORDS,
	MIN_ENGLISH_CHUNK_PERCENTAGE,
	MIN_ENGLISH_SCORE,
	MIN_ENGLISH_SUMMARY_SCORE,
	MIN_LANGUAGE_DETECTION_CHARACTERS,
	MIN_LANGUAGE_DETECTION_WORDS,
	validateEnglishSummaryText,
	validateEnglishText,
	validateEnglishTextBlocks,
} from './index.ts'

const english =
	'This project adds useful tools, configurable settings, and clear documentation for every player.'
const russian =
	'Этот проект добавляет новые инструменты и значительно улучшает игровой процесс для всех игроков.'

test('accepts English text and retains whole-text language diagnostics', () => {
	const result = validateEnglishText(english)
	const englishDetection = result.detections.find(({ language }) => language === 'eng')

	assert.equal(result.valid, true)
	assert.ok(englishDetection)
	assert.ok(englishDetection.accuracy > MIN_ENGLISH_SCORE)
	assert.deepEqual(result.reasons, [])
	assert.deepEqual(
		result.detections,
		francAll(english).map(([language, accuracy]) => ({ language, accuracy })),
	)
})

test('rejects text containing only confidently non-English chunks', () => {
	for (const text of [
		russian,
		'これは新しい洞窟と構造物を追加し、すべてのプレイヤーの世界生成を改善するプロジェクトです。',
		'Um modpack focado em desempenho, imersão e exploração, mantendo a experiência próxima ao jogo original.',
	]) {
		const result = validateEnglishText(text)

		assert.equal(result.valid, false, text)
		assert.deepEqual(result.reasons, ['insufficient-english-chunk-coverage'])
	}
})

test('accepts bilingual text when English chunks are 30% of classified chunks', () => {
	const blocks = [
		english,
		english,
		english,
		russian,
		russian,
		russian,
		russian,
		russian,
		russian,
		russian,
	]
	const analysis = analyzeLanguageChunks(blocks)
	const result = validateEnglishTextBlocks(blocks)

	assert.equal(analysis.englishChunks, 3)
	assert.equal(analysis.nonEnglishChunks, 7)
	assert.equal(analysis.englishChunkPercentage, 0.3)
	assert.equal(MIN_ENGLISH_CHUNK_PERCENTAGE, 0.3)
	assert.equal(result.valid, true)
	assert.deepEqual(result.reasons, [])
})

test('rejects mixed-language text when English chunks are below 30% of classified chunks', () => {
	const blocks = [english, english, russian, russian, russian, russian, russian]
	const analysis = analyzeLanguageChunks(blocks)
	const result = validateEnglishTextBlocks(blocks)

	assert.equal(analysis.englishChunks, 2)
	assert.equal(analysis.nonEnglishChunks, 5)
	assert.ok((analysis.englishChunkPercentage ?? 0) < MIN_ENGLISH_CHUNK_PERCENTAGE)
	assert.equal(result.valid, false)
	assert.deepEqual(result.reasons, ['insufficient-english-chunk-coverage'])
})

test('uses overlapping 24-word windows with a 12-word stride', () => {
	const text = Array.from({ length: 48 }, (_, index) => `word${index}`).join(' ')
	const analysis = analyzeLanguageChunks([text])

	assert.equal(LANGUAGE_CHUNK_WORDS, 24)
	assert.equal(LANGUAGE_CHUNK_STRIDE_WORDS, 12)
	assert.equal(analysis.totalChunks, 4)
})

test('skips chunks below the minimum word count', () => {
	for (const text of [
		'Minecraft',
		'Minecraft server',
		'This description has only seven English words',
	]) {
		assert.deepEqual(validateEnglishText(text), { valid: true, detections: [], reasons: [] })
	}

	assert.equal(MIN_LANGUAGE_DETECTION_WORDS, 8)
})

test('skips chunks below the minimum character count', () => {
	assert.deepEqual(validateEnglishText('a b c d e f g h'), {
		valid: true,
		detections: [],
		reasons: [],
	})
	assert.equal(MIN_LANGUAGE_DETECTION_CHARACTERS, 35)
})

test('accepts summaries with an English score of at least 50% without changing description logic', () => {
	const text =
		'Um modpack Fabric focado em desempenho, imersão e exploração, mantendo a experiência próxima ao Minecraft Vanilla.'
	const summaryResult = validateEnglishSummaryText(text)
	const englishDetection = summaryResult.detections.find(({ language }) => language === 'eng')

	assert.equal(MIN_ENGLISH_SUMMARY_SCORE, 0.5)
	assert.ok(englishDetection)
	assert.ok(englishDetection.accuracy >= MIN_ENGLISH_SUMMARY_SCORE)
	assert.equal(summaryResult.valid, true)
	assert.equal(validateEnglishText(text).valid, false)
})

test('skips summary detection below its word or character minimum', () => {
	for (const text of ['This summary has only seven English words', 'a b c d e f g h']) {
		assert.deepEqual(validateEnglishSummaryText(text), {
			valid: true,
			detections: [],
			reasons: [],
		})
	}

	assert.equal(MIN_LANGUAGE_DETECTION_WORDS, 8)
	assert.equal(MIN_LANGUAGE_DETECTION_CHARACTERS, 35)
})

test('allows empty text to be handled by required-field validation', () => {
	assert.deepEqual(validateEnglishText('  '), { valid: true, detections: [], reasons: [] })
})
