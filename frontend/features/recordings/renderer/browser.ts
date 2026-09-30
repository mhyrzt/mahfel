import { createWorkspaceRenderer } from "./workspace";

const renderer = createWorkspaceRenderer(document.body);
declare global {
  interface Window {
    recordingRenderer: ReturnType<typeof createWorkspaceRenderer>;
  }
}
window.recordingRenderer = renderer;
