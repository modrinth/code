import { createGlobalState } from "@vueuse/core";
import { type Ref, shallowRef } from "vue";

/**
 * Maximum number of console output lines to store
 * @type {number}
 */
const maxLines = 5000;
const batchTimeout = 300; // ms
const initialBatchSize = 100;

/**
 * Provides a global console output state management system
 * Allows adding, storing, and clearing console output with a maximum line limit
 *
 * @returns {Object} Console state management methods and reactive state
 * @property {Ref<string[]>} consoleOutput - Reactive array of console output lines
 * @property {function(string): void} addConsoleOutput - Method to add a new console output line
 * @property {function(): void} clear - Method to clear all console output
 */
export const usePyroConsole = createGlobalState(() => {
  /**
   * Reactive array storing console output lines
   * @type {Ref<string[]>}
   */
  const output: Ref<string[]> = shallowRef<string[]>([]);

  let lineBuffer: string[] = [];
  let batchTimer: NodeJS.Timeout | null = null;
  let isProcessingInitialBatch = false;

  const flushBuffer = () => {
    if (lineBuffer.length === 0) return;

    const processedLines = lineBuffer.flatMap((line) => line.split("\n").filter(Boolean));

    if (isProcessingInitialBatch && processedLines.length >= initialBatchSize) {
      isProcessingInitialBatch = false;
      output.value = processedLines.slice(-maxLines);
      lineBuffer = [];
      batchTimer = null;
      return;
    }

    const newOutput = [...output.value, ...processedLines];
    output.value = newOutput.slice(-maxLines);

    lineBuffer = [];
    batchTimer = null;
  };

  /**
   * Adds a new output line to the console output
   * Automatically removes the oldest line if max output is exceeded
   *
   * @param {string} line - The console output line to add
   */
  const addLine = (line: string): void => {
    lineBuffer.push(line);

    if (!batchTimer) {
      batchTimer = setTimeout(flushBuffer, batchTimeout);
    }
  };

  /**
   * Adds multiple output lines to the console output
   * Automatically removes the oldest lines if max output is exceeded
   *
   * @param {string[]} lines - The console output lines to add
   * @returns {void}
   */
  const addLines = (lines: string[]): void => {
    if (output.value.length === 0 && lines.length >= initialBatchSize) {
      isProcessingInitialBatch = true;
      lineBuffer = lines;
      flushBuffer();
      return;
    }

    lineBuffer.push(...lines);

    if (!batchTimer) {
      batchTimer = setTimeout(flushBuffer, batchTimeout);
    }
  };

  /**
   * Clears all console output lines
   */
  const clear = (): void => {
    output.value = [];
    lineBuffer = [];
    isProcessingInitialBatch = false;
    if (batchTimer) {
      clearTimeout(batchTimer);
      batchTimer = null;
    }
  };

  return {
    output,
    addLine,
    addLines,
    clear,
  };
});
