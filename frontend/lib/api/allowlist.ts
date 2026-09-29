const SESSION_PATH =
  /^v1\/sessions\/[0-9a-fA-F-]{36}(\/(roster|capabilities|access|admission|start|end|cancel|participation|recordings(\/commands)?|invite-links|lobby(\/[0-9a-fA-F-]{36}\/(admit|reject))?|collaboration(\/(events|grants(\/[0-9a-fA-F-]{36})?|presentation|viewport|(notes|whiteboard)\/images))?))?$/;
const RESOURCE_PATH = /^v1\/(auth|rooms|files|echo|status|health|invite-links)(\/.*)?$/;

export function isAllowedBackendPath(path: string): boolean {
  return path === "v1/ws/tickets" || RESOURCE_PATH.test(path) || SESSION_PATH.test(path);
}
