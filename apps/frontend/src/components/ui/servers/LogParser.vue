<template>
  <div class="parsed-log w-fit whitespace-nowrap px-6 py-1" v-html="parsedLog"></div>
</template>

<script lang="ts">
import { defineComponent, computed } from "vue";
import type { PropType } from "vue";
import Convert from "ansi-to-html";

export default defineComponent({
  name: "LogParser",
  props: {
    log: {
      type: String as PropType<string>,
      required: true,
    },
  },
  setup(props) {
    const colors = {
      30: "#101010",
      31: "#EFA6A2",
      32: "#80C990",
      33: "#A69460",
      34: "#A3B8EF",
      35: "#E6A3DC",
      36: "#50CACD",
      37: "#808080",
      90: "#454545",
      91: "#E0AF85",
      92: "#5ACCAF",
      93: "#C8C874",
      94: "#CCACED",
      95: "#F2A1C2",
      96: "#74C3E4",
      97: "#C0C0C0",
    };

    const convert = new Convert({
      fg: "#FFF",
      bg: "#000",
      newline: false,
      escapeXML: false,
      stream: false,
      colors,
    });

    const parsedLog = computed(() => {
      return convert.toHtml(props.log);
    });

    return {
      parsedLog,
    };
  },
});
</script>

<style scoped>
html.light-mode .parsed-log:hover {
  background-color: #ccc;
}

html.dark-mode .parsed-log:hover {
  background-color: #333;
}

html.oled-mode .parsed-log:hover {
  background-color: #333;
}
</style>
