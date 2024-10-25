type SelectionRange = {
  start: number;
  end: number;
};

export function useTerminalSelection() {
  const selections = ref<SelectionRange[]>([]);
  const selectionStart = ref<number | null>(null);
  const isSelecting = ref(false);

  const isSelected = (index: number) => {
    return selections.value.some((range) => index >= range.start && index <= range.end);
  };

  const clearSelections = () => {
    selections.value = [];
    selectionStart.value = null;
    isSelecting.value = false;
  };

  const handleLineClick = (index: number, event: MouseEvent) => {
    if (!event.shiftKey) {
      if (isSelected(index)) {
        clearSelections();
      } else {
        clearSelections();
        selections.value = [{ start: index, end: index }];
        selectionStart.value = index;
        isSelecting.value = true;
      }
    } else if (selectionStart.value !== null) {
      const start = Math.min(selectionStart.value, index);
      const end = Math.max(selectionStart.value, index);
      selections.value = [{ start, end }];
      isSelecting.value = true;
    } else {
      selections.value = [{ start: index, end: index }];
      selectionStart.value = index;
      isSelecting.value = true;
    }
  };

  const handleScroll = () => {
    isSelecting.value = false;
  };

  const getSelectedText = (consoleOutput: string[]) => {
    if (!selections.value.length) return "";

    const range = selections.value[0];
    return consoleOutput.slice(range.start, range.end + 1).join("\n");
  };

  return {
    selections,
    selectionStart,
    isSelecting,
    isSelected,
    clearSelections,
    handleLineClick,
    handleScroll,
    getSelectedText,
  };
}
