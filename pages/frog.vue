<template>
  <div class="normal-page">
    <div class="card">
      <h1>Frog</h1>
      <p>You've been frogged! 🐸</p>
      <img
        src="https://cdn.modrinth.com/frog.png"
        alt="a photorealistic painting of a frog labyrinth"
      />
    </div>
  </div>
</template>

<script>
export default {
  auth: false,
}
</script>

<style lang="scss" scoped>
.card {
  width: 100%;
  text-align: center;
}

img {
  margin-block: 0 1.5rem;
}
</style>
