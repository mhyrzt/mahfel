export function fitScale(
  containerWidth: number,
  containerHeight: number,
  contentWidth: number,
  contentHeight: number,
): number {
  if (containerWidth <= 0 || containerHeight <= 0 || contentWidth <= 0 || contentHeight <= 0) {
    return 1;
  }
  return Math.min(containerWidth / contentWidth, containerHeight / contentHeight);
}
