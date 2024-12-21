let pyroConsole: typeof import("@pyro/tartaros").PyroConsole.prototype | null = null;
let ctx: OffscreenCanvasRenderingContext2D | null = null;
const mod = import("@pyro/tartaros");

onmessage = async (e) => {
  try {
    switch (e.data.type) {
      case "init": {
        const { canvas } = e.data;
        ctx = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D;
        const { PyroConsole } = await mod;
        pyroConsole = new PyroConsole(canvas);
        pyroConsole.init();
        postMessage({ type: "init", height: ctx.canvas.height });
        break;
      }

      case "line": {
        if (!pyroConsole) {
          throw new Error("PyroConsole not initialized");
        }
        const { text } = e.data;
        pyroConsole.add_line(text);
        break;
      }

      case "destroy": {
        pyroConsole?.destroy();
        pyroConsole?.free();
        break;
      }

      case "resize": {
        if (!ctx) {
          throw new Error("Canvas not initialized");
        }

        const { width } = e.data;
        ctx.canvas.width = width;
        pyroConsole?.redraw();
        postMessage({ type: "resize", width });
        break;
      }

      case "clear": {
        pyroConsole?.clear();
        break;
      }

      case "mousedown": {
        if (!pyroConsole) return;
        const { x, y, clientWidth, clientHeight } = e.data;
        pyroConsole.mouse_down(x, y, clientWidth, clientHeight);
        break;
      }

      case "mouseup": {
        if (!pyroConsole) return;
        pyroConsole.mouse_up();
        break;
      }

      case "mousemove": {
        if (!pyroConsole) return;
        const { y, clientHeight } = e.data;
        pyroConsole.mouse_move(y, clientHeight);
        break;
      }

      case "wheel": {
        if (!pyroConsole) return;
        const { deltaY } = e.data;
        pyroConsole.wheel(deltaY);
        break;
      }
    }
  } catch (e) {
    console.error(e);
  }
};
