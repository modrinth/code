# Markdown

<script setup>
import { renderHighlightedString } from 'omorphia';
import { ref } from 'vue';

const b = '`';
const body = ref('');

fetch('https://raw.githubusercontent.com/joeyespo/grip/master/tests/input/gfm-test.md')
        .then((response) => response.text())
        .then((response) => body.value = response)
</script>

:::raw
<DemoContainer>
<div style="width: 100%" class="markdown-body" v-html="renderHighlightedString(body)" />
</DemoContainer>
:::

<style lang="scss">
h1, h2, h3, h4, h5, h6 {
  line-height: 1.15;
  font-weight: revert;
  font-size: revert;
  margin: revert;
}

ul, ol {
  list-style: revert;
  margin: revert;
  padding: revert;
}

blockquote {
  margin: revert;
}
</style>
