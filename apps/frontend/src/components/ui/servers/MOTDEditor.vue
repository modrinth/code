<template>
  <div class="flex items-center justify-center">
    <div class="w-full overflow-hidden">
      <div class="mb-4">
        <div
          v-for="(line, lineIndex) in motd"
          :key="lineIndex"
          class="relative mb-2 rounded bg-button-bg p-2"
        >
          <div
            class="font-minecraft text-white"
            :contenteditable="true"
            spellcheck="false"
            @input="handleInput($event, lineIndex)"
            @keydown.enter.prevent
            @paste.prevent="handlePaste($event, lineIndex)"
            @mouseup="handleSelection(lineIndex)"
            v-html="renderLine(line)"
          ></div>
          <div class="text-sm text-gray-400">
            {{ motd[lineIndex].reduce((sum, segment) => sum + segment.text.length, 0) }}/45
            characters
          </div>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div
        v-if="showPopup"
        :style="{ top: `${popupY}px`, left: `${popupX}px` }"
        class="fixed z-10 flex flex-col items-end gap-2 transition-all duration-300 ease-in-out"
      >
        <div class="rounded-xl border bg-table-alternateRow p-2 shadow-lg">
          <div class="flex space-x-2">
            <Button
              v-for="style in styles"
              :key="style.name"
              icon-only
              transparent
              @click="applyStyle({ [style.name]: !currentStyle[style.name] })"
            >
              <component :is="style.icon" class="h-4 w-4" />
            </Button>
            <div class="relative overflow-y-scroll">
              <Button icon-only transparent :class="colorPicker ?? 'hidden'" @click="pickColor">
                <PaintBrushIcon />
              </Button>
            </div>
          </div>
        </div>
        <div
          v-if="colorPicker"
          icon-only
          class="w-fit overflow-y-auto rounded-xl p-2 [&&]:bg-table-alternateRow"
        >
          <div :class="colorPicker ? `grid grid-flow-col grid-rows-4 gap-2` : '[&&]:hidden'">
            <button
              v-for="format in sortedFormatCodes()"
              :key="format.code"
              class="rounded-full p-3"
              :style="{ backgroundColor: format.color }"
              :title="format.description"
              @click="applyStyle({ color: format.color })"
            ></button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import {
  ItalicIcon,
  BoldIcon,
  StrikethroughIcon,
  UnderlineIcon,
  PaintBrushIcon,
  ChevronLeftIcon,
} from "@modrinth/assets";
import { Button } from "@modrinth/ui";

const props = defineProps({
  server: {
    type: Object,
    required: true,
  },
});

const formatCodes = [
  { code: "§f", color: "white", description: "White" },
  { code: "§7", color: "#AAAAAA", description: "Gray" },
  { code: "§8", color: "#555555", description: "Dark Gray" },
  { code: "§0", color: "#000000", description: "Black" },
  { code: "§9", color: "#5555FF", description: "Blue" },
  { code: "§1", color: "#0000AA", description: "Dark Blue" },
  { code: "§b", color: "#55FFFF", description: "Aqua" },
  { code: "§3", color: "#00AAAA", description: "Dark Aqua" },
  { code: "§a", color: "#55FF55", description: "Green" },
  { code: "§2", color: "#00AA00", description: "Dark Green" },
  { code: "§e", color: "#FFFF55", description: "Yellow" },
  { code: "§6", color: "#FFAA00", description: "Gold" },
  { code: "§c", color: "#FF5555", description: "Red" },
  { code: "§4", color: "#AA0000", description: "Dark Red" },
  { code: "§d", color: "#FF55FF", description: "Light Purple" },
  { code: "§5", color: "#AA00AA", description: "Dark Purple" },
];

const sortedFormatCodes = () => {
  const colors = formatCodes;
  if (colors[0].description === "White") {
    colors.reverse();
  }
  return colors;
};

const minecraftEmojis = [
  { char: "☺", name: "SMILING FACE" },
  { char: "☹", name: "FROWNING FACE" },
  { char: "☠", name: "SKULL AND CROSSBONES" },
  { char: "❣", name: "HEART EXCLAMATION" },
  { char: "❤", name: "RED HEART" },
  { char: "✌", name: "VICTORY HAND" },
  { char: "☝", name: "INDEX POINTING UP" },
  { char: "✍", name: "WRITING HAND" },
  { char: "♨", name: "HOT SPRINGS" },
  { char: "✈", name: "AIRPLANE" },
  { char: "⌛", name: "HOURGLASS DONE" },
  { char: "⌚", name: "WATCH" },
  { char: "☀", name: "SUN" },
  { char: "☁", name: "CLOUD" },
  { char: "☂", name: "UMBRELLA" },
  { char: "❄", name: "SNOWFLAKE" },
  { char: "☃", name: "SNOWMAN" },
  { char: "☄", name: "COMET" },
  { char: "♠", name: "SPADE SUIT" },
  { char: "♥", name: "HEART SUIT" },
  { char: "♦", name: "DIAMOND SUIT" },
  { char: "♣", name: "CLUB SUIT" },
  { char: "♟", name: "CHESS PAWN" },
  { char: "☎", name: "TELEPHONE" },
  { char: "⌨", name: "KEYBOARD" },
  { char: "✉", name: "ENVELOPE" },
  { char: "✏", name: "PENCIL" },
  { char: "✒", name: "BLACK PEN" },
  { char: "✂", name: "SCISSORS" },
  { char: "☢", name: "RADIOACTIVE" },
  { char: "☣", name: "BIOHAZARD" },
  { char: "⬆", name: "UP ARROW" },
  { char: "⬇", name: "DOWN ARROW" },
  { char: "➡", name: "RIGHT ARROW" },
  { char: "⬅", name: "LEFT ARROW" },
  { char: "↗", name: "UP-RIGHT ARROW" },
  { char: "↘", name: "DOWN-RIGHT ARROW" },
  { char: "↙", name: "DOWN-LEFT ARROW" },
  { char: "↖", name: "UP-LEFT ARROW" },
  { char: "↕", name: "UP-DOWN ARROW" },
  { char: "↔", name: "LEFT-RIGHT ARROW" },
  { char: "↩", name: "RIGHT ARROW CURVING LEFT" },
  { char: "↪", name: "LEFT ARROW CURVING RIGHT" },
  { char: "✡", name: "STAR OF DAVID" },
  { char: "☸", name: "WHEEL OF DHARMA" },
  { char: "☯", name: "YIN YANG" },
  { char: "✝", name: "LATIN CROSS" },
  { char: "☦", name: "ORTHODOX CROSS" },
  { char: "☪", name: "STAR AND CRESCENT" },
  { char: "☮", name: "PEACE SYMBOL" },
  { char: "♈", name: "ARIES" },
  { char: "♉", name: "TAURUS" },
  { char: "♊", name: "GEMINI" },
  { char: "♋", name: "CANCER" },
  { char: "♌", name: "LEO" },
  { char: "♍", name: "VIRGO" },
  { char: "♎", name: "LIBRA" },
  { char: "♏", name: "SCORPIO" },
  { char: "♐", name: "SAGITTARIUS" },
  { char: "♑", name: "CAPRICORN" },
  { char: "♒", name: "AQUARIUS" },
  { char: "♓", name: "PISCES" },
  { char: "▶", name: "PLAY BUTTON" },
  { char: "◀", name: "REVERSE BUTTON" },
  { char: "♀", name: "FEMALE SIGN" },
  { char: "♂", name: "MALE SIGN" },
  { char: "✖", name: "MULTIPLY" },
  { char: "‼", name: "DOUBLE EXCLAMATION MARK" },
  { char: "〰", name: "WAVY DASH" },
  { char: "☑", name: "CHECK BOX WITH CHECK" },
  { char: "✔", name: "CHECK MARK" },
  { char: "✳", name: "EIGHT-SPOKED ASTERISK" },
  { char: "✴", name: "EIGHT-POINTED STAR" },
  { char: "❇", name: "SPARKLE" },
  { char: "©", name: "COPYRIGHT" },
  { char: "®", name: "REGISTERED" },
  { char: "™", name: "TRADE MARK" },
  { char: "Ⓜ", name: "CIRCLED M" },
  { char: "㊗", name: 'JAPANESE "CONGRATULATIONS" BUTTON' },
  { char: "㊙", name: 'JAPANESE "SECRET" BUTTON' },
  { char: "▪", name: "BLACK SMALL SQUARE" },
  { char: "▫", name: "WHITE SMALL SQUARE" },
  { char: "☷", name: "TRIGRAM FOR EARTH" },
  { char: "☵", name: "TRIGRAM FOR WATER" },
  { char: "☶", name: "TRIGRAM FOR MOUNTAIN" },
  { char: "☋", name: "DESCENDING NODE" },
  { char: "☌", name: "CONJUNCTION" },
  { char: "♜", name: "BLACK CHESS ROOK" },
  { char: "♕", name: "WHITE CHESS QUEEN" },
  { char: "♡", name: "WHITE HEART SUIT" },
  { char: "♬", name: "BEAMED SIXTEENTH NOTES" },
  { char: "☚", name: "BLACK LEFT POINTING INDEX" },
  { char: "♮", name: "MUSIC NATURAL SIGN" },
  { char: "♝", name: "BLACK CHESS BISHOP" },
  { char: "♯", name: "SHARP" },
  { char: "☴", name: "TRIGRAM FOR WIND" },
  { char: "♭", name: "FLAT" },
  { char: "☓", name: "SALTIRE" },
  { char: "☛", name: "BLACK RIGHT POINTING INDEX" },
  { char: "☭", name: "HAMMER AND SICKLE" },
  { char: "♢", name: "WHITE DIAMOND SUIT" },
  { char: "✐", name: "UPPER RIGHT PENCIL" },
  { char: "♖", name: "WHITE CHESS ROOK" },
  { char: "☈", name: "THUNDERSTORM" },
  { char: "☒", name: "BALLOT BOX WITH X" },
  { char: "★", name: "BLACK STAR" },
  { char: "♚", name: "BLACK CHESS KING" },
  { char: "♛", name: "BLACK CHESS QUEEN" },
  { char: "✎", name: "LOWER RIGHT PENCIL" },
  { char: "♪", name: "EIGHTH NOTE" },
  { char: "☰", name: "TRIGRAM FOR HEAVEN" },
  { char: "☽", name: "FIRST QUARTER MOON" },
  { char: "☡", name: "CAUTION SIGN" },
  { char: "☼", name: "WHITE SUN WITH RAYS" },
  { char: "♅", name: "URANUS" },
  { char: "☐", name: "BALLOT BOX" },
  { char: "☟", name: "WHITE DOWN POINTING INDEX" },
  { char: "❦", name: "FLORAL HEART" },
  { char: "☊", name: "ASCENDING NODE" },
  { char: "☍", name: "OPPOSITION" },
  { char: "☬", name: "ADI SHAKTI" },
  { char: "♧", name: "WHITE CLUB SUIT" },
  { char: "☫", name: "FARSI SYMBOL" },
  { char: "☱", name: "TRIGRAM FOR LAKE" },
  { char: "☾", name: "LAST QUARTER MOON" },
  { char: "☤", name: "CADUCEUS" },
  { char: "❧", name: "ROTATED FLORAL HEART BULLET" },
  { char: "♄", name: "SATURN" },
  { char: "♁", name: "EARTH" },
  { char: "♔", name: "WHITE CHESS KING" },
  { char: "❥", name: "ROTATED HEAVY BLACK HEART BULLET" },
  { char: "☥", name: "ANKH" },
  { char: "☻", name: "BLACK SMILING FACE" },
  { char: "♤", name: "WHITE SPADE SUIT" },
  { char: "♞", name: "BLACK CHESS KNIGHT" },
  { char: "♆", name: "NEPTUNE" },
  { char: "#", name: "HASH SIGN" },
  { char: "♃", name: "JUPITER" },
  { char: "♩", name: "QUARTER NOTE" },
  { char: "☇", name: "LIGHTNING" },
  { char: "☞", name: "WHITE RIGHT POINTING INDEX" },
  { char: "♫", name: "BEAMED EIGHTH NOTES" },
  { char: "☏", name: "WHITE TELEPHONE" },
  { char: "♘", name: "WHITE CHESS KNIGHT" },
  { char: "☧", name: "CHI RHO" },
  { char: "☉", name: "SUN" },
  { char: "♇", name: "PLUTO" },
  { char: "☩", name: "CROSS OF JERUSALEM" },
  { char: "♙", name: "WHITE CHESS PAWN" },
  { char: "☜", name: "WHITE LEFT POINTING INDEX" },
  { char: "☲", name: "TRIGRAM FOR FIRE" },
  { char: "☨", name: "CROSS OF LORRAINE" },
  { char: "♗", name: "WHITE CHESS BISHOP" },
  { char: "☳", name: "TRIGRAM FOR THUNDER" },
  { char: "⚔", name: "CROSSED SWORDS" },
  { char: "⚀", name: "DICE ONE" },
];

const rawMotd = ref(props.server.general?.motd ?? "");

const motd = computed(() => {
  const lines = rawMotd.value.split("\n");
  return lines.map((line) => {
    const segments = [];
    let currentSegment = { text: "", color: "White" };
    let i = 0;
    while (i < line.length) {
      if (line[i] === "§") {
        if (currentSegment.text) {
          segments.push({ ...currentSegment });
          currentSegment = { text: "", color: "White" };
        }
        const formatCode = line.substr(i, 2);
        const format = formatCodes.find((f) => f.code === formatCode);
        console.log(format);
        console.log(formatCode);
        if (format) {
          currentSegment.color = format.color;
          i += 2;
          continue;
        } else if (formatCode === "§l") {
          currentSegment.bold = true;
          i += 2;
          continue;
        } else if (formatCode === "§o") {
          currentSegment.italic = true;
          i += 2;
          continue;
        } else if (formatCode === "§n") {
          currentSegment.underline = true;
          i += 2;
          continue;
        } else if (formatCode === "§m") {
          currentSegment.strikethrough = true;
          i += 2;
          continue;
        }
      }
      currentSegment.text += line[i];
      i++;
    }
    if (currentSegment.text) {
      segments.push(currentSegment);
    }
    return segments;
  });
});

const styles = [
  {
    name: "bold",
    icon: BoldIcon,
  },
  {
    name: "italic",
    icon: ItalicIcon,
  },
  {
    name: "underline",
    icon: UnderlineIcon,
  },
  {
    name: "strikethrough",
    icon: StrikethroughIcon,
  },
];

const showPopup = ref(false);
const popupX = ref(0);
const popupY = ref(0);
const currentLineIndex = ref(0);
const selectionStart = ref(0);
const selectionEnd = ref(0);
const colorPicker = ref(false);

const pickColor = () => {
  colorPicker.value = !colorPicker.value;
};

const totalCharacters = computed(() => {
  return motd.value.reduce((sum, line) => {
    return Math.max(
      sum,
      line.reduce((lineSum, segment) => lineSum + segment.text.length, 0),
    );
  }, 0);
});

const minecraftFormat = computed(() => {
  return motd.value
    .map((line) => {
      return line
        .map((segment) => {
          let format = getColorCode(segment.color);
          if (segment.bold) format += "§l";
          if (segment.italic) format += "§o";
          if (segment.underline) format += "§n";
          if (segment.strikethrough) format += "§m";
          return format + segment.text;
        })
        .join("");
    })
    .join("\n");
});

const currentStyle = computed(() => {
  const line = motd.value[currentLineIndex.value];
  if (!line) return {};

  let start = 0;
  for (const segment of line) {
    if (start + segment.text.length > selectionStart.value) {
      return {
        color: segment.color || "White",
        bold: segment.bold || false,
        italic: segment.italic || false,
        underline: segment.underline || false,
        strikethrough: segment.strikethrough || false,
      };
    }
    start += segment.text.length;
  }
  return {};
});

function getColorCode(color) {
  const format = formatCodes.find((f) => f.description === color);
  return format ? format.code : "§f";
}

function renderLine(line) {
  return line
    .map((segment) => {
      let style = `color: ${segment.color};`;
      if (segment.bold) style += "font-weight: 900;";
      if (segment.italic) style += "font-style: italic;";
      if (segment.underline) style += "text-decoration: underline;";
      if (segment.strikethrough) style += "text-decoration: line-through;";
      return `<span style="${style}">${segment.text}</span>`;
    })
    .join("");
}

function handleSelection(lineIndex) {
  const selection = window.getSelection();
  if (selection.toString().length > 0) {
    const range = selection.getRangeAt(0);
    const rect = range.getBoundingClientRect();

    popupX.value = rect.left;
    popupY.value = rect.bottom;
    showPopup.value = true;
    currentLineIndex.value = lineIndex;

    const lineElement = document.querySelectorAll("[contenteditable]")[lineIndex];
    const rangeClone = range.cloneRange();
    rangeClone.selectNodeContents(lineElement);
    rangeClone.setEnd(range.startContainer, range.startOffset);
    selectionStart.value = rangeClone.toString().length;
    selectionEnd.value = selectionStart.value + range.toString().length;
  } else {
    showPopup.value = false;
    colorPicker.value = false;
  }
}

function applyStyle(newStyle) {
  const line = motd.value[currentLineIndex.value];
  const newLine = [];
  let currentPos = 0;

  for (const segment of line) {
    if (currentPos + segment.text.length <= selectionStart.value) {
      newLine.push(segment);
    } else if (currentPos >= selectionEnd.value) {
      newLine.push(segment);
    } else {
      const beforeSelection = segment.text.slice(0, Math.max(0, selectionStart.value - currentPos));
      const inSelection = segment.text.slice(
        Math.max(0, selectionStart.value - currentPos),
        Math.min(segment.text.length, selectionEnd.value - currentPos),
      );
      const afterSelection = segment.text.slice(
        Math.min(segment.text.length, selectionEnd.value - currentPos),
      );
      console.log(beforeSelection);
      console.log(inSelection);
      console.log(afterSelection);

      if (beforeSelection) newLine.push({ ...segment, text: beforeSelection });
      if (inSelection) {
        const mergedStyle = { ...segment, ...newStyle };
        for (const key in newStyle) {
          if (newStyle[key] === false) {
            delete mergedStyle[key];
          }
        }
        newLine.push({ ...mergedStyle, text: inSelection });
      }
      if (afterSelection) newLine.push({ ...segment, text: afterSelection });
    }
    currentPos += segment.text.length;
  }

  motd.value[currentLineIndex.value] = newLine;
  showPopup.value = false;
  colorPicker.value = false;

  // Rerender the line to reflect the changes
  nextTick(() => {
    const lineElement = document.querySelectorAll("[contenteditable]")[currentLineIndex.value];
    lineElement.innerHTML = renderLine(newLine);
  });
}

function insertEmoji() {
  const emoji = "☺";
  if (totalCharacters.value + emoji.length <= 90) {
    applyStyle({ text: emoji });
  }
}

function handleInput(event, lineIndex) {
  const newText = event.target.textContent;
  const oldText = motd.value[lineIndex].reduce((acc, segment) => acc + segment.text, "");
  const diff = newText.length - oldText.length;

  if (newText.length <= 45) {
    const selection = window.getSelection();
    const range = selection.getRangeAt(0);
    const cursorOffset = getCursorOffset(event.target, range);

    const newLine = [];
    let currentPos = 0;
    for (const segment of motd.value[lineIndex]) {
      const segmentEnd = currentPos + segment.text.length;
      const newSegmentText = newText.slice(currentPos, Math.min(segmentEnd, newText.length));
      if (newSegmentText) {
        newLine.push({ ...segment, text: newSegmentText });
      }
      currentPos = segmentEnd;
      if (currentPos >= newText.length) break;
    }
    if (currentPos < newText.length) {
      newLine.push({ text: newText.slice(currentPos), color: "White" });
    }
    motd.value[lineIndex] = newLine;

    nextTick(() => {
      const lineElement = event.target;
      lineElement.innerHTML = renderLine(newLine);

      const newRange = document.createRange();
      const sel = window.getSelection();
      const { node, offset } = getCursorNodeAndOffset(lineElement, cursorOffset);

      if (node) {
        newRange.setStart(node, offset);
        newRange.collapse(true);
        sel.removeAllRanges();
        sel.addRange(newRange);
      }
    });
  } else {
    event.target.innerHTML = renderLine(motd.value[lineIndex]);
  }
}

// Helper function to get cursor offset considering styled spans
function getCursorOffset(element, range) {
  let offset = 0;
  const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT, null, false);
  let node;

  while ((node = walker.nextNode())) {
    if (node === range.startContainer) {
      return offset + range.startOffset;
    }
    offset += node.length;
  }
  return offset;
}

// Helper function to find the node and offset for cursor placement
function getCursorNodeAndOffset(element, targetOffset) {
  let currentOffset = 0;
  const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT, null, false);
  let node;

  while ((node = walker.nextNode())) {
    if (currentOffset + node.length >= targetOffset) {
      return { node, offset: targetOffset - currentOffset };
    }
    currentOffset += node.length;
  }

  // If we've gone past the end, return the last possible position
  const lastTextNode = element.lastChild?.lastChild;
  return { node: lastTextNode, offset: lastTextNode?.length || 0 };
}

function handlePaste(event, lineIndex) {
  event.preventDefault();
  const pastedText = (event.clipboardData || window.clipboardData).getData("text");
  const selection = window.getSelection();
  const range = selection.getRangeAt(0);
  const startOffset = range.startOffset;

  const currentText = motd.value[lineIndex].reduce((acc, segment) => acc + segment.text, "");
  const newText = currentText.slice(0, startOffset) + pastedText + currentText.slice(startOffset);

  if (newText.length <= 45) {
    // Preserve existing styles by matching new text with old segments
    const newLine = [];
    let currentPos = 0;
    for (const segment of motd.value[lineIndex]) {
      if (currentPos < startOffset) {
        const segmentEnd = Math.min(currentPos + segment.text.length, startOffset);
        newLine.push({ ...segment, text: newText.slice(currentPos, segmentEnd) });
        currentPos = segmentEnd;
      } else if (currentPos >= startOffset + pastedText.length) {
        newLine.push({ ...segment, text: newText.slice(currentPos) });
        break;
      }
    }
    // Insert pasted text as a new segment
    if (currentPos < startOffset + pastedText.length) {
      newLine.push({
        text: newText.slice(currentPos, startOffset + pastedText.length),
        color: "White",
      });
    }
    motd.value[lineIndex] = newLine;

    nextTick(() => {
      const lineElement = document.querySelectorAll("[contenteditable]")[lineIndex];
      lineElement.innerHTML = renderLine(newLine);
      const newRange = document.createRange();
      const sel = window.getSelection();
      newRange.setStart(lineElement.childNodes[0], startOffset + pastedText.length);
      newRange.collapse(true);
      sel.removeAllRanges();
      sel.addRange(newRange);
    });
  }
}
</script>

<style scoped>
.minecraft-font {
  font-family: "Minecraft", monospace;
  font-size: 16px;
  line-height: 1.5;
}

[contenteditable] {
  outline: none;
}
</style>

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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease-in-out;
}
.fade-enter, .fade-leave-to /* .fade-leave-active in <2.1.8 */ {
  opacity: 0;
}
</style>
