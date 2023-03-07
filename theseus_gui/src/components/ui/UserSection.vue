<script setup>
import { Avatar } from 'omorphia'
</script>

<template>
  <section class="user-section">
    <Avatar size="sm" circle src="https://cdn.modrinth.com/data/AANobbMI/icon.png" />
    <section>
      <p id="username">OreoViking</p>
      <a href="#">Manage accounts</a>
    </section>
  </section>
</template>

<style lang="scss" scoped>
.user-section {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  width: 100%;
  height: 70px;
  margin-left: 3rem;

  section {
    display: flex;
    flex-direction: column;
    justify-content: flex-start;
    text-align: left;
    margin-left: 0.5rem;
  }

  #username {
    margin-bottom: 0.3rem;
    font-size: 1.1rem;
    font-weight: 400;
    line-height: 19px;
  }

  a {
    font-size: 0.75rem;
    font-weight: 400;
    color: var(--color-secondary);
  }
}
</style>
