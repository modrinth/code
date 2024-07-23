<template>
  <div v-html="formattedLog"></div>
</template>

<script lang="ts">
import { defineComponent, computed } from "vue";

export default defineComponent({
  name: "Log",
  props: {
    logLine: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    const colorMap: { [key: string]: string } = {
      "30": "#141414",
      "31": "#F15050",
      "32": "#71D871",
      "33": "#FCE66A",
      "34": "#72A0F7",
      "35": "#F778F7",
      "36": "#79FFFF",
      "37": "#E7E6E6",
      "90": "#7F7F7F",
    };

    const formattedLog = computed(() => {
      let log = props.logLine.replace(/\u001b\[([0-9;]+)m/g, (match, p1) => {
        const codes = p1.split(";");
        const colorCode = codes.find((code: string) => colorMap[code]);
        if (colorCode) {
          return `<span style="color: ${colorMap[colorCode]}">`;
        }
        return "</span>";
      });

      log = log.replace(/\[m$/, "");
      log = log.replace(/\u001b\[m$/, "");
      log = log.replace(/\u001b\[0m$/, "");
      log = log.replace(/\u001b$/, "");

      return log + "</span>";
    });

    return {
      formattedLog,
    };
  },
});
</script>

<style scoped>
span {
  display: inline;
}
</style>
