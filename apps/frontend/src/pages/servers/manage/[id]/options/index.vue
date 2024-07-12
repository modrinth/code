<template>
  <div>
    <section class="card">
      <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
      <label for="username-field">
        <span class="label__title">{{ formatMessage(messages.servernameTitle) }}</span>
        <span class="label__description">{{ formatMessage(messages.servernameDescription) }}</span>
      </label>
      <input id="servername-field" type="text" value="BetterAdventures++ Server" />
    </section>
  </div>
</template>

<script setup lang="ts">
const { formatMessage } = useVIntl();
const messages = defineMessages({
  title: {
    id: "server.options.general.title",
    defaultMessage: "General",
  },
  description: {
    id: "server.options.general.description",
    defaultMessage: "Your server settings.",
  },
  servernameTitle: {
    id: "server.options.general.servername.title",
    defaultMessage: "Server Name",
  },
  servernameDescription: {
    id: "server.options.general.servername.description",
    defaultMessage: "A name to help identify your server.",
  },
});
</script>
