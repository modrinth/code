<script>
import VueTurnstile from 'vue-turnstile';

export default {
  components: { VueTurnstile },
  props: {
    modelValue: {
      type: String,
      required: true,
    },
  },
  data() {
    return {
      internalToken: this.modelValue,
    };
  },
  watch: {
    modelValue(newValue) {
      this.internalToken = newValue;
    },
  },
  methods: {
    onTokenUpdate(token) {
      console.log('Token:', token);
      this.internalToken = token;
      this.$emit('update:modelValue', token);
    },
  },
};
</script>

<template>
  <iv>
    <vue-turnstile
      site-key="0x4AAAAAAA04yTLWmQXpl2OY"
      :model-value="internalToken"
      @update:modelValue="onTokenUpdate"
    />
  </iv>
</template>