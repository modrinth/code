<template>
  <Card>
    <div v-html="body" class="markdown-body"/>
  </Card>
</template>

<script setup>
import { Card } from 'omorphia'

const body = "<span>\n    <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_HEADER.png\"\n         alt=\"Spirit banner\" width=\"1100\"/>\n    <center>\n        <h2>Spirit is a magic mod for Forge and Fabric 1.18.2+ that adds new features related to souls and spawning in Minecraft</h2>\n    </center>\n    <a href=\"https://bisecthosting.com/CodexAdrian\">\n        <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_PROMO.png\"\n             alt=\"Overview\" width=\"1100\"/>\n    </a>\n    <img src=\"https://cdn.discordapp.com/attachments/881367981463072809/1007044890670137465/BH_NU_BANNER1.png\"\n         alt=\"Overview\" width=\"1100\"/>\n    <center>\n        <h2>\n            Crafting in Spirit revolves around englufing objects in Soul Fire. You can use <a\n                href=\"https://curseforge.com/minecraft/mc-mods/jei\">JEI</a> or <a\n                href=\"https://curseforge.com/minecraft/mc-mods/jei\">REI</a> to view\n            all the recipes in Spirit. If you want some further explanation, this video by SirColor does an excellent job explaining what the main functions of Spirit are\n        </h2>\n    </center>\n    <div>\n        <iframe width=\"900\" height=\"570\" src=\"https://www.youtube.com/embed/lv6C3dZ7hGs\" title=\"YouTube video player\"\n                allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\"\n                allowfullscreen></iframe>\n    </div>\n    <img src=\"https://cdn.discordapp.com/attachments/881367981463072809/1007044890351390720/BH_NU_BANNER2.png\"\n         alt=\"Features\" width=\"1100\"/>\n    <h2>Spirit has a number of features designed to enhance your experience when it comes to soul magic in Minecraft</h2>\n    <blockquote>\n        <h3>Using souls you can:</h3>\n        Summon Monsters<br>\n        Empower Tools<br>\n        Transmute Mobs into other mobs<br>\n        Craft items with soul fire<br>\n    </blockquote>\n    <h2>Detailed descriptions on how you can use these various items, recipes and features can be found on our <a href=\"https://codexadrian.gitbook.io/spirit\">wiki!</a></h2>\n    <a href=\"https://codexadrian.tech/discord\">\n        <img src=\"https://www.bisecthosting.com/images/CF/Spirit/BH_NU_BANNER3.png\"\n             alt=\"Overview\" width=\"1100\"/>\n    </a>\n</span>\n";
</script>

<script>
export default {
  name: "Description"
}
</script>

<style scoped>

</style>