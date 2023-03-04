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
    margin-left: 1rem;
  }

  #username {
    margin-bottom: 5px;
    font-size: 18px;
    font-weight: 400;
    line-height: 19px;
  }

  a {
    font-size: 14px;
    font-weight: 400;
  }
}

.dark-mode {
  .user-section {
    a {
      color: #c0c0c0;
    }
  }
}
</style>
