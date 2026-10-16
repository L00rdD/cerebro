// One shared animation clock for every scene, capped at 10 fps and paused while hidden.

type Frame = (seconds: number) => void;

const FRAME_MS = 100;
const subscribers = new Set<Frame>();
let running = true;
let handle: number | undefined;
let last = 0;

function loop(now: number) {
  handle = undefined;
  if (!running || subscribers.size === 0) return;
  if (now - last >= FRAME_MS) {
    last = now;
    const seconds = Date.now() / 1000;
    subscribers.forEach((draw) => draw(seconds));
  }
  handle = requestAnimationFrame(loop);
}

function kick() {
  if (running && handle === undefined && subscribers.size > 0) handle = requestAnimationFrame(loop);
}

/** Registers a draw callback; returns the unsubscribe function. */
export function onFrame(draw: Frame): () => void {
  subscribers.add(draw);
  draw(Date.now() / 1000);
  kick();
  return () => subscribers.delete(draw);
}

export function setAnimating(on: boolean) {
  running = on;
  if (!on && handle !== undefined) {
    cancelAnimationFrame(handle);
    handle = undefined;
  }
  kick();
}
