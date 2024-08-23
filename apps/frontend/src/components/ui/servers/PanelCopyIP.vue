<template>
  <button class="btn btn-transparent" @click="copyText">
    <CopyIcon />
    Copy IP
  </button>
</template>

<script lang="ts">
import { defineComponent } from "vue";
import { CopyIcon } from "@modrinth/assets";

const app = useNuxtApp();

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
    subdomain: {
      type: String,
    },
  },
  setup(props) {
    const copyText = () => {
      const text = props.subdomain ? `${props.subdomain}` : `${props.ip}:${props.port}`;
      navigator.clipboard.writeText(text);

      // @ts-ignore
      app.$notify({
        group: "server",
        title: `Copied IP`,
        text: `Your server's IP has been copied to your clipboard`,
        type: "success",
      });
    };

    return { copyText };
  },
});
</script>
