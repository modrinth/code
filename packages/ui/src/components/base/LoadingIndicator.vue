<template>
  <div class="w-full flex items-center justify-center flex-col gap-2">
    <div class="title">Loading</div>
    <div class="placeholder"></div>
    <div class="placeholder"></div>
    <div class="placeholder"></div>
  </div>
</template>

<script setup></script>
<style scoped>
.title {
  position: absolute;
  z-index: 1;
  font-weight: bold;
  color: var(--color-contrast);

  &::after {
    content: '';
    animation: dots 2s infinite;
  }
}

@keyframes dots {
  25% {
    content: '';
  }
  50% {
    content: '.';
  }
  75% {
    content: '..';
  }
  0%,
  100% {
    content: '...';
  }
}

.placeholder {
  border-radius: var(--radius-lg);
  width: 100%;
  height: 4rem;
  opacity: 0.25;
  position: relative;
  overflow: hidden;
  background-color: var(--color-raised-bg);
  animation: pop 4s ease-in-out infinite;
  border: 1px solid transparent;

  &::before {
    content: '';
    position: absolute;
    inset: 0;
    background-image: linear-gradient(
      -45deg,
      transparent 30%,
      rgba(196, 217, 237, 0.075) 50%,
      transparent 70%
    );
    animation: shimmer 4s ease-in-out infinite;
  }

  &:nth-child(2)::before {
    animation-delay: 0s;
  }

  &:nth-child(3)::before {
    animation-delay: 0.3s;
  }

  &:nth-child(4)::before {
    animation-delay: 0.6s;
  }

  &:nth-child(2) {
    animation-delay: 0s;
  }

  &:nth-child(3) {
    animation-delay: 0.3s;
  }

  &:nth-child(4) {
    animation-delay: 0.6s;
  }
}

@keyframes pop {
  from {
    opacity: 0.25;
    border-color: transparent;
  }
  50% {
    opacity: 0.5;
    border-color: var(--color-button-bg);
  }
  to {
    opacity: 0.25;
    border-color: transparent;
  }
}

@keyframes shimmer {
  from {
    transform: translateX(-80%);
  }
  50%,
  to {
    transform: translateX(80%);
  }
}
</style>
