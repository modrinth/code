<template>
  <div class="flex gap-2">
    <ButtonStyled circular>
      <a
        v-tooltip="`Share on Bluesky`"
        :href="`https://bsky.app/intent/compose?text=${encodedUrl}`"
        target="_blank"
      >
        <BlueskyIcon />
      </a>
    </ButtonStyled>
    <ButtonStyled circular>
      <a
        v-tooltip="`Share on Mastodon`"
        :href="`https://tootpick.org/#text=${encodedUrl}`"
        target="_blank"
      >
        <MastodonIcon />
      </a>
    </ButtonStyled>
    <ButtonStyled circular>
      <a
        v-tooltip="`Share on X`"
        :href="`https://www.x.com/intent/post?url=${encodedUrl}`"
        target="_blank"
      >
        <TwitterIcon />
      </a>
    </ButtonStyled>
    <ButtonStyled circular>
      <a
        v-tooltip="`Share via email`"
        :href="`mailto:${encodedTitle ? `?subject=${encodedTitle}&` : `?`}body=${encodedUrl}`"
        target="_blank"
      >
        <MailIcon />
      </a>
    </ButtonStyled>
    <ButtonStyled circular>
      <button
        v-tooltip="copied ? `Copied to clipboard` : `Copy link`"
        :disabled="copied"
        class="relative grid place-items-center overflow-hidden"
        @click="copyToClipboard(url)"
      >
        <CheckIcon
          class="absolute transition-all ease-in-out"
          :class="copied ? 'translate-y-0' : 'translate-y-7'"
        />
        <LinkIcon
          class="absolute transition-all ease-in-out"
          :class="copied ? '-translate-y-7' : 'translate-y-0'"
        />
      </button>
    </ButtonStyled>
  </div>
</template>

<script setup lang="ts">
import {
  BlueskyIcon,
  CheckIcon,
  LinkIcon,
  MailIcon,
  MastodonIcon,
  TwitterIcon,
} from "@modrinth/assets";
import { ButtonStyled } from "@modrinth/ui";

const props = defineProps<{
  title?: string;
  url: string;
}>();

const copied = ref(false);
const encodedUrl = computed(() => encodeURIComponent(props.url));
const encodedTitle = computed(() => (props.title ? encodeURIComponent(props.title) : undefined));

async function copyToClipboard(text: string) {
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => {
    copied.value = false;
  }, 3000);
}
</script>
