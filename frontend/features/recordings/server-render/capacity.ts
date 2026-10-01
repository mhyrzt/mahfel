import { RenderError } from "./types";

let active = 0;

export async function withRenderCapacity<T>(operation: () => Promise<T>): Promise<T> {
  if (active >= 2) {
    throw new RenderError("renderer_busy", 503);
  }
  active += 1;
  try {
    return await operation();
  } finally {
    active -= 1;
  }
}
