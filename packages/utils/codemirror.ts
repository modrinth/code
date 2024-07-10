// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-nocheck

import { insertNewlineAndIndent } from '@codemirror/commands'
import { deleteMarkupBackward } from '@codemirror/lang-markdown'
import { getIndentation, indentString, syntaxTree } from '@codemirror/language'
import { type EditorState, type Transaction } from '@codemirror/state'
import { type EditorView, type Command, type KeyBinding } from '@codemirror/view'

const toggleBold: Command = ({ state, dispatch }) => {
  return toggleAround(state, dispatch, '**', '**')
}

const toggleItalic: Command = ({ state, dispatch }) => {
  return toggleAround(state, dispatch, '_', '_')
}

const toggleStrikethrough: Command = ({ state, dispatch }) => {
  return toggleAround(state, dispatch, '~~', '~~')
}

const toggleCodeBlock: Command = ({ state, dispatch }) => {
  const lineBreak = state.lineBreak
  const codeBlockMark = `${lineBreak}\`\`\`${lineBreak}`
  return toggleAround(state, dispatch, codeBlockMark, codeBlockMark)
}

const toggleSpoiler: Command = ({ state, dispatch }) => {
  // Insert details tag with a summary tag at the start
  const detailsTags = ['\n<details>\n<summary>Spoiler</summary>\n\n', '\n\n</details>\n\n']
  return toggleAround(state, dispatch, detailsTags[0], detailsTags[1])
}

const toggleHeader: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '# ')
}

const toggleHeader2: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '## ')
}

const toggleHeader3: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '### ')
}

const toggleHeader4: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '#### ')
}

const toggleHeader5: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '##### ')
}

const toggleHeader6: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '###### ')
}

const toggleQuote: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '> ')
}

const toggleBulletList: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '- ')
}

const toggleOrderedList: Command = ({ state, dispatch }) => {
  return toggleLineStart(state, dispatch, '1. ')
}

const yankSelection = ({ state }: EditorView): string => {
  const { from, to } = state.selection.main
  const selectedText = state.doc.sliceString(from, to)
  return selectedText
}

const replaceSelection = ({ state, dispatch }: EditorView, text: string) => {
  const { from, to } = state.selection.main
  const transaction = state.update({
    changes: { from, to, insert: text },
    selection: { anchor: from + text.length, head: from + text.length },
  })
  dispatch(transaction)
  return true
}

type Dispatch = (tr: Transaction) => void

const surroundedByText = (
  state: EditorState,
  open: string,
  close: string,
): 'inclusive' | 'exclusive' | 'none' => {
  const { from, to } = state.selection.main

  // Check for inclusive surrounding first
  const selectedText = state.doc.sliceString(from, to)
  if (selectedText.startsWith(open) && selectedText.endsWith(close)) {
    return 'inclusive'
  }

  // Then check for exclusive surrounding
  const beforeText = state.doc.sliceString(Math.max(0, from - open.length), from)
  const afterText = state.doc.sliceString(to, to + close.length)
  if (beforeText === open && afterText === close) {
    return 'exclusive'
  }

  // Return 'none' if no surrounding detected
  return 'none'
}

// TODO: Node based toggleAround so that we can support nested delimiters
const toggleAround = (
  state: EditorState,
  dispatch: Dispatch,
  open: string,
  close: string,
): boolean => {
  const { from, to } = state.selection.main

  const isSurrounded = surroundedByText(state, open, close)

  if (isSurrounded !== 'none') {
    const isInclusive = isSurrounded === 'inclusive'
    let transaction: Transaction

    if (isInclusive) {
      // Remove delimiters on the inside edges of the selected text
      transaction = state.update({
        changes: [
          { from, to: from + open.length, insert: '' },
          { from: to - close.length, to, insert: '' },
        ],
      })
    } else {
      // Remove delimiters on the outside edges of the selected text
      transaction = state.update({
        changes: [
          { from: from - open.length, to: from, insert: '' },
          { from: to, to: to + close.length, insert: '' },
        ],
      })
    }

    dispatch(transaction)
    return true
  }

  // Add delimiters around the selected text
  const transaction = state.update({
    changes: [
      { from, insert: open },
      { from: to, insert: close },
    ],
    selection: { anchor: from + open.length, head: to + open.length },
  })

  dispatch(transaction)
  return true
}

const toggleLineStart = (state: EditorState, dispatch: Dispatch, text: string): boolean => {
  const lines = state.doc.lineAt(state.selection.main.from)
  const lineBreak = state.lineBreak

  const range = {
    from: lines.from,
    to: state.selection.main.to,
  }

  const selectedText = state.doc.sliceString(range.from, range.to)
  const shouldRemove = selectedText.startsWith(text)

  let transaction: Transaction | undefined

  if (shouldRemove) {
    const numOfSelectedLinesThatNeedToBeRemoved = selectedText.split(lineBreak + text).length
    const modifiedText = selectedText.substring(text.length).replaceAll(lineBreak + text, lineBreak)
    transaction = state.update({
      changes: { from: range.from, to: range.to, insert: modifiedText },
      selection: {
        anchor: state.selection.main.from - text.length,
        head: state.selection.main.to - text.length * numOfSelectedLinesThatNeedToBeRemoved,
      },
    })
  } else {
    const modifiedText = text + selectedText.replaceAll(lineBreak, lineBreak + text)
    const lengthDiff = modifiedText.length - selectedText.length
    transaction = state.update({
      changes: { from: range.from, to: range.to, insert: modifiedText },
      selection: {
        anchor: state.selection.main.from + text.length,
        head: state.selection.main.to + lengthDiff,
      },
    })
  }

  if (!transaction) return false

  dispatch(transaction)
  return true
}

const continueNodeTypes = ['ListItem', 'Blockquote']
const blackListedNodeTypes = ['CodeBlock']

const getListStructure = (state: EditorState, head: number) => {
  const tree = syntaxTree(state)
  const headNode = tree.resolve(head, -1)
  const stack = []

  let node: typeof headNode.parent = headNode
  while (node) {
    if (continueNodeTypes.includes(node.name)) {
      stack.push(node)
    }
    if (blackListedNodeTypes.includes(node.name)) {
      return null
    }
    if (node.name === 'Document') {
      break
    }

    node = node.parent
  }

  return stack
}

const insertNewlineContinueMark: Command = (view): boolean => {
  const { state, dispatch } = view
  const {
    selection: {
      main: { head },
    },
  } = state

  // Get the current list structure to examine
  const stack = getListStructure(state, head)
  if (!stack || stack.length === 0) {
    // insert a newline as normal so that mobile works
    return insertNewlineAndIndent(view)
  }

  const lastNode = stack[stack.length - 1]

  // Get the necessary indentation
  const indentation = getIndentation(state, head)
  const indentStr = indentation ? indentString(state, indentation) : ''

  // Initialize a transaction variable
  let transaction: Transaction | undefined

  const lineContent = state.doc.lineAt(head).text

  // Identify the patterns that should cancel the list continuation
  // TODO: Implement Node based cancellation
  const cancelPatterns = ['```', '# ', '> ']

  const listMark = lastNode.getChild('ListMark')
  if (listMark) {
    cancelPatterns.push(`${state.doc.sliceString(listMark.from, listMark.to)} `)
  }

  // Skip if current line matches any of the cancel patterns
  if (cancelPatterns.includes(lineContent)) {
    transaction = createSimpleTransaction(state)
    dispatch(transaction)
    return true
  }

  switch (lastNode.name) {
    case 'ListItem':
      if (!listMark) return false
      transaction = createListTransaction(state, indentStr, listMark.from, listMark.to)
      break
    case 'Blockquote':
      transaction = createBlockquoteTransaction(state, indentStr)
      break
  }

  if (transaction) {
    dispatch(transaction)
    return true
  }

  return false
}

// Creates a transaction for a simple line break
const createSimpleTransaction = (state: EditorState) => {
  const {
    lineBreak,
    selection: {
      main: { head },
    },
  } = state
  const line = state.doc.lineAt(head)
  return state.update({
    changes: {
      from: line.from,
      to: line.to,
      insert: lineBreak,
    },
  })
}

// Creates a transaction for continuing a list item
const createListTransaction = (state: EditorState, indentStr: string, from: number, to: number) => {
  const {
    lineBreak,
    selection: {
      main: { head },
    },
  } = state
  const listMarkContent = state.doc.sliceString(from, to)
  const insert = `${lineBreak}${indentStr}${incrementMark(listMarkContent)} `
  return state.update({
    changes: { from: head, insert },
    selection: { anchor: head + insert.length, head: head + insert.length },
  })
}

// Creates a transaction for continuing a blockquote
const createBlockquoteTransaction = (state: EditorState, indentStr: string) => {
  const {
    lineBreak,
    selection: {
      main: { head },
    },
  } = state
  const insert = `${lineBreak}${indentStr}> `
  return state.update({
    changes: { from: head, insert },
    selection: { anchor: head + insert.length, head: head + insert.length },
  })
}

const incrementMark = (mark: string): string => {
  const numberedListRegex = /^(\d+)\.$/
  const match = numberedListRegex.exec(mark)
  if (match) {
    const number = parseInt(match[1])
    return `${(number + 1).toString()}.`
  }
  return mark
}

const commands = {
  toggleBold,
  toggleItalic,
  toggleStrikethrough,
  toggleCodeBlock,
  toggleSpoiler,
  toggleHeader,
  toggleHeader2,
  toggleHeader3,
  toggleHeader4,
  toggleHeader5,
  toggleHeader6,
  toggleQuote,
  toggleBulletList,
  toggleOrderedList,
  insertNewlineContinueMark,

  // Utility
  yankSelection,
  replaceSelection,
}

export const markdownCommands = commands
export const modrinthMarkdownEditorKeymap: KeyBinding[] = [
  { key: 'Enter', run: insertNewlineContinueMark },
  { key: 'Backspace', run: deleteMarkupBackward },
  { key: 'Mod-b', run: toggleBold },
  { key: 'Mod-i', run: toggleItalic },
  { key: 'Mod-e', run: toggleCodeBlock },
  { key: 'Mod-s', run: toggleStrikethrough },
  { key: 'Mod-Shift-.', run: toggleQuote },
]
