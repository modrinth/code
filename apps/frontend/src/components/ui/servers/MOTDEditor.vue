<template>
  <div class="flex flex-col gap-4 rounded-xl bg-bg p-4">
    <div class="mcbg flex gap-4 p-4 text-white">
      <slot />
      <div class="font-minecraft mb-2 flex w-full flex-col gap-1 text-2xl">
        <div class="flex items-center justify-between">Minecraft Server</div>
        <div
          v-for="(line, lineIndex) in motd"
          :key="lineIndex"
          class="relative mb-2 rounded bg-gray-700 p-2"
        >
          <div class="minecraft-font text-white" @mouseup="handleSelection(lineIndex)">
            <span
              v-for="(segment, segIndex) in line"
              :key="segIndex"
              :class="getSegmentStyle(segment)"
              :contenteditable="true"
              @input="handleInput($event, lineIndex, segIndex)"
              @keydown.enter.prevent
              @paste.prevent="handlePaste($event, lineIndex, segIndex)"
              >{{ segment.text }}</span
            >
          </div>
        </div>
      </div>
    </div>
    <div class="text-sm text-gray-400">{{ totalCharacters }}/90</div>

    <div
      v-if="showPopup"
      class="fixed z-10 rounded border border-gray-600 bg-gray-800 p-2 shadow-lg"
      :style="{ top: `${popupY}px`, left: `${popupX}px` }"
    >
      <div class="mb-2 flex space-x-1">
        <button
          v-for="color in formatCodes"
          :key="color.description"
          v-tooltip="color.description"
          class="h-6 w-6 rounded-full p-2"
          :class="color.color"
          @click="applyStyle(color.code)"
        ></button>
      </div>
      <div class="mb-2 flex space-x-2">
        <button
          v-for="style in styles"
          :key="style"
          :class="{ 'bg-gray-600': currentSegment[style] }"
          class="rounded px-2 py-1 text-sm text-white"
          @click="applyStyle({ [style]: !currentSegment[style] })"
        >
          {{ style }}
        </button>
      </div>
      <button class="w-full rounded bg-gray-700 px-2 py-1 text-sm text-white" @click="insertEmoji">
        Insert Emoji
      </button>
    </div>
  </div>
</template>

<script setup>
const motd = ref([
  [{ text: "Welcome to our Minecraft server!", color: "white" }],
  [{ text: "Enjoy your stay and have fun!", color: "yellow" }],
]);

const formatCodes = [
  { code: "§f", color: "bg-white", description: "White" },
  { code: "§7", color: "bg-[#AAAAAA]", description: "Gray" },
  { code: "§8", color: "bg-[#555555]", description: "Dark Gray" },
  { code: "§0", color: "bg-[#000000]", description: "Black" },
  { code: "§9", color: "bg-[#5555FF]", description: "Blue" },
  { code: "§1", color: "bg-[#0000AA]", description: "Dark Blue" },
  { code: "§b", color: "bg-[#55FFFF]", description: "Aqua" },
  { code: "§3", color: "bg-[#00AAAA]", description: "Dark Aqua" },
  { code: "§a", color: "bg-[#55FF55]", description: "Green" },
  { code: "§2", color: "bg-[#00AA00]", description: "Dark Green" },
  { code: "§e", color: "bg-[#FFFF55]", description: "Yellow" },
  { code: "§6", color: "bg-[#FFAA00]", description: "Gold" },
  { code: "§c", color: "bg-[#FF5555]", description: "Red" },
  { code: "§4", color: "bg-[#AA0000]", description: "Dark Red" },
  { code: "§d", color: "bg-[#FF55FF]", description: "Light Purple" },
  { code: "§5", color: "bg-[#AA00AA]", description: "Dark Purple" },
];

const styles = ["bold", "italic", "underline", "strikethrough"];

const showPopup = ref(false);
const popupX = ref(0);
const popupY = ref(0);
const currentLineIndex = ref(0);
const currentSegmentIndex = ref(0);
const selectionStart = ref(0);
const selectionEnd = ref(0);

const totalCharacters = computed(() => {
  return motd.value.reduce((sum, line) => {
    return sum + line.reduce((lineSum, segment) => lineSum + segment.text.length, 0);
  }, 0);
});

const minecraftFormat = computed(() => {
  return motd.value
    .map((line) => {
      return line
        .map((segment) => {
          let format = "§";
          if (segment.color) format += getColorCode(segment.color);
          if (segment.bold) format += "l";
          if (segment.italic) format += "o";
          if (segment.underline) format += "n";
          if (segment.strikethrough) format += "m";
          return format + segment.text;
        })
        .join("");
    })
    .join("\n");
});

const currentSegment = computed(() => {
  return motd.value[currentLineIndex.value][currentSegmentIndex.value];
});

function getColorCode(color) {
  const colorCodes = Object.fromEntries(formatCodes.map((code) => [code.code, code.color]));
  return colorCodes[color] || "f";
}

function getSegmentStyle(segment) {
  const styleClasses = [];
  if (segment.color)
    styleClasses.push(`${formatCodes.find((code) => code.color === segment.color).code}`);
  if (segment.bold) styleClasses.push("font-bold");
  if (segment.italic) styleClasses.push("italic");
  if (segment.underline) styleClasses.push("underline");
  if (segment.strikethrough) styleClasses.push("line-through");

  const styleClassesString = styleClasses.join(" ");
  return styleClassesString;
}

function handleSelection(lineIndex) {
  const selection = window.getSelection();
  if (selection.toString().length > 0) {
    const range = selection.getRangeAt(0);
    const rect = range.getBoundingClientRect();
    popupX.value = rect.left;
    popupY.value = rect.bottom + window.scrollY;
    showPopup.value = true;
    currentLineIndex.value = lineIndex;

    // Find the segment where the selection starts
    let charCount = 0;
    for (let i = 0; i < motd.value[lineIndex].length; i++) {
      const segmentLength = motd.value[lineIndex][i].text.length;
      if (charCount + segmentLength >= selection.anchorOffset) {
        currentSegmentIndex.value = i;
        selectionStart.value = selection.anchorOffset - charCount;
        selectionEnd.value = selection.focusOffset - charCount;
        break;
      }
      charCount += segmentLength;
    }
  } else {
    showPopup.value = false;
  }
}

function applyStyle(style) {
  const segment = motd.value[currentLineIndex.value][currentSegmentIndex.value];
  const newSegment = { ...segment, ...style };
  const selectedText = segment.text.slice(selectionStart.value, selectionEnd.value);

  if (selectedText.length === segment.text.length) {
    // If the entire segment is selected, just update it
    motd.value[currentLineIndex.value][currentSegmentIndex.value] = newSegment;
  } else {
    // Split the segment and apply the style only to the selected part
    const beforeText = segment.text.slice(0, selectionStart.value);
    const afterText = segment.text.slice(selectionEnd.value);

    motd.value[currentLineIndex.value].splice(
      currentSegmentIndex.value,
      1,
      { ...segment, text: beforeText },
      { ...newSegment, text: selectedText },
      { ...segment, text: afterText },
    );
  }

  showPopup.value = false;
}

function insertEmoji() {
  // In a real implementation, you would open an emoji picker here
  // For this example, we'll just insert a simple smiley face
  const segment = motd.value[currentLineIndex.value][currentSegmentIndex.value];
  const newText =
    segment.text.slice(0, selectionStart.value) + "☺" + segment.text.slice(selectionEnd.value);
  if (totalCharacters.value + 1 <= 90) {
    motd.value[currentLineIndex.value][currentSegmentIndex.value].text = newText;
  }
  showPopup.value = false;
}

function handleInput(event, lineIndex, segmentIndex) {
  const selection = window.getSelection();
  const cursorPosition = selection.focusOffset;

  const newText = event.target.textContent;
  const oldText = motd.value[lineIndex][segmentIndex].text;
  const diff = newText.length - oldText.length;

  if (totalCharacters.value + diff <= 90) {
    motd.value[lineIndex][segmentIndex].text = newText;

    // Use Vue.nextTick to ensure the DOM has updated before setting the cursor position
    Vue.nextTick(() => {
      const range = document.createRange();
      const sel = window.getSelection();
      range.setStart(event.target.childNodes[0], cursorPosition);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    });
  } else {
    // If the new text would exceed the limit, revert the change
    event.target.textContent = oldText;

    // Set cursor to the end of the text
    Vue.nextTick(() => {
      const range = document.createRange();
      const sel = window.getSelection();
      range.selectNodeContents(event.target);
      range.collapse(false);
      sel.removeAllRanges();
      sel.addRange(range);
    });
  }
}

function handlePaste(event, lineIndex, segmentIndex) {
  const pastedText = (event.clipboardData || window.clipboardData).getData("text");
  const selection = window.getSelection();
  const cursorPosition = selection.focusOffset;

  const segment = motd.value[lineIndex][segmentIndex];
  const newText =
    segment.text.slice(0, cursorPosition) + pastedText + segment.text.slice(cursorPosition);
  const diff = newText.length - segment.text.length;

  if (totalCharacters.value + diff <= 90) {
    motd.value[lineIndex][segmentIndex].text = newText;

    Vue.nextTick(() => {
      const range = document.createRange();
      const sel = window.getSelection();
      range.setStart(event.target.childNodes[0], cursorPosition + pastedText.length);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    });
  }
}
</script>

<style scoped>
@font-face {
  font-family: "Monocraft";
  src: url("/Monocraft.ttf") format("truetype");
}

.font-minecraft {
  font-family: "Monocraft", monospace;
}

.mcbg {
  background: url("@/assets/images/servers/minecraft-background-dark.png") repeat center center;
}
</style>
