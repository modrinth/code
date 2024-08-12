<template>
  <button class="btn btn-transparent" @click="copyText">
    <CopyIcon />
    Copy IP
  </button>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { CopyIcon } from "@modrinth/assets";

export default defineComponent({
  name: "CopyIPButton",
  components: {
    CopyIcon,
  },
  props: {
    ip: {
      type: String,
      required: true,
    },
    port: {
      type: Number,
      required: true,
    },
  },
  setup(props) {
    const copyText = () => {
      const text = `${props.ip}:${props.port}`;
      navigator.clipboard.writeText(text);
    };

    return { copyText };
  },
});
</script>
