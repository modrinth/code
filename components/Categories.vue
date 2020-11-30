<template>
  <div class="categories">
    <p v-if="categories.includes('fabric')">
      <FabricLoader />
      Fabric
    </p>
    <p v-if="categories.includes('forge')">
      <ForgeLoader />
      Forge
    </p>
    <p v-if="categories.includes('technology')">
      <TechCategory />
      Technology
    </p>
    <p v-if="categories.includes('adventure')">
      <AdventureCategory />
      Adventure
    </p>
    <p v-if="categories.includes('magic')">
      <MagicCategory />
      Magic
    </p>
    <p v-if="categories.includes('utility')">
      <UtilityCategory />
      Utility
    </p>
    <p v-if="categories.includes('decoration')">
      <DecorationCategory />
      Decoration
    </p>
    <p v-if="categories.includes('library')">
      <LibraryCategory />
      Library
    </p>
    <p v-if="categories.includes('cursed')">
      <CursedCategory />
      Cursed
    </p>
    <p v-if="categories.includes('worldgen')">
      <WorldGenCategory />
      Worldgen
    </p>
    <p v-if="categories.includes('storage')">
      <StorageCategory />
      Storage
    </p>
    <p v-if="categories.includes('food')">
      <FoodCategory />
      Food
    </p>
    <p v-if="categories.includes('equipment')">
      <EquipmentCategory />
      Equipment
    </p>
    <p v-if="categories.includes('misc')">
      <MiscCategory />
      Misc
    </p>
  </div>
</template>

<script>
import TechCategory from '~/assets/images/categories/tech.svg?inline'
import AdventureCategory from '~/assets/images/categories/adventure.svg?inline'
import CursedCategory from '~/assets/images/categories/cursed.svg?inline'
import DecorationCategory from '~/assets/images/categories/decoration.svg?inline'
import EquipmentCategory from '~/assets/images/categories/equipment.svg?inline'
import FoodCategory from '~/assets/images/categories/food.svg?inline'
import LibraryCategory from '~/assets/images/categories/library.svg?inline'
import MagicCategory from '~/assets/images/categories/magic.svg?inline'
import MiscCategory from '~/assets/images/categories/misc.svg?inline'
import StorageCategory from '~/assets/images/categories/storage.svg?inline'
import UtilityCategory from '~/assets/images/categories/utility.svg?inline'
import WorldGenCategory from '~/assets/images/categories/worldgen.svg?inline'

import ForgeLoader from '~/assets/images/categories/forge.svg?inline'
import FabricLoader from '~/assets/images/categories/fabric.svg?inline'

export default {
  name: 'Categories',
  components: {
    TechCategory,
    AdventureCategory,
    CursedCategory,
    DecorationCategory,
    EquipmentCategory,
    FoodCategory,
    LibraryCategory,
    MagicCategory,
    MiscCategory,
    StorageCategory,
    UtilityCategory,
    WorldGenCategory,
    ForgeLoader,
    FabricLoader,
  },
  props: {
    categories: {
      type: Array,
      default() {
        return []
      },
    },
  },
}
</script>

<style lang="scss" scoped>
.categories {
  @extend %row;
  flex-wrap: wrap;

  p {
    display: flex;
    align-items: center;
    flex-direction: row;
    background-color: var(--color-category-bg);
    border-radius: var(--size-rounded-max);
    color: var(--color-category-text);
    margin-top: 0.25em;
    margin-bottom: 0.25em;
    margin-right: 0.5em;
    padding: 0.4em 0.7em;
    font-size: var(--font-size-sm);
    height: 1em;
    svg {
      width: 15px;
      margin-right: 5px;
    }
  }
}
</style>
