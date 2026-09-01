import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ref } from 'vue'

import MarkdownBody from '../../components/base/MarkdownBody.vue'
import MarkdownEditor from '../../components/base/MarkdownEditor.vue'

const meta = {
	title: 'Base/ModrinthFlavoredMarkdown',
	component: MarkdownBody,
} satisfies Meta<typeof MarkdownBody>

export default meta

function md(source: string, heightClass = 'h-96'): StoryObj {
	return {
		render: () => ({
			components: { MarkdownBody, MarkdownEditor },
			setup() {
				const content = ref(source.trim())
				return { content, heightClass }
			},
			template: /*html*/ `
				<div :class="heightClass">
					<MarkdownEditor v-model="content" />
					<MarkdownBody :source="content" />
				</div>
			`,
		}),
	}
}


export const Modrinth: StoryObj = md(`
<project/sodium>

<@modrinth>

<org/modrinth>

<collection/wL9BPICg>
`)

export const Headers: StoryObj = md(`
# Header 1
## Header 2
### Header 3
#### Header 4
##### Header 5
###### Header 6
`)

export const Inline: StoryObj = md(`
**bold**

__also bold__

*italics*

_also italics_

~~strikethrough~~

\`snippet\`

^Superscript^
~Subscript~

||Boo!||

!!Also Scary!!

==Marked==

en--dash

em---dash

...

+-

:cheese:
`)

export const Link: StoryObj = md(`
https://example.com

[A Link](example.com)

[No way a Title?](example.com "Damn, that's cool")

[This one uses a reference][1]

[This one is referenced by name]

[1]: https://example.com
[This one is referenced by name]: https://example.com
`)

export const List: StoryObj = md(`
### Bullets

- One
- Two
    - Nested One

### Numbered

1. One
2. Two
    1. Nested One

### Task List

- [x] Completed
- [ ] Pending
    - [x] Nested
`)

export const CodeBlock: StoryObj = md(`
\`\`\`js [test.js] {1}
function hello() {
    console.log("Hello")
}
\`\`\`

\`\`\`json
{
  "foo": "bar"
}
\`\`\`
`)

export const Table: StoryObj = md(`
| Column A | Column B |
| -------- | -------- |
| foo      | bar      |
| baz      | qux      |
`)

export const Latex: StoryObj = md(`
Inline math $x^2 + y^2 = z^2$ right in a sentence.

$$
E = mc^2
$$
`)

export const Mermaid: StoryObj = md(`
\`\`\`mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C(Rounded step)
    B -->|No| D([Stadium step])
    C --> E[[Subroutine]]
    D --> E
    E --> F[End]

    subgraph Group
        G[Inside group]
    end

    F --> G
\`\`\`
`)

export const BlockQuote: StoryObj = md(`
> Single Line

>Multi
>Line

>We even
>
>have
>
>blank lines!
`)

export const DetailsRegion: StoryObj = md(`
+++ Summary
Details
+++

++> Pre-Opened
Amazing!
++>

+++ No Details here
+++

++++ Nested
++>
Get Spoiled!
++>
++++
`)

export const Alert: StoryObj = md(`
> [!TIP]
> We have alerts now?

> [!IMPORTANT]
> Sure do!

> [!WARNING]
> They don't even need content

> [!CAUTION]

> [!NOTE]+ They can even be collapsible?
> > [!TIP]- And They can Nest?
> > > [!IMPORTANT] Nesting doesn't look very good tho
> > > Wait a minute, they support custom titles!
`)

export const Footnote: StoryObj = md(`
We support Footnotes![^1]

[^1]: This is true.
`)
