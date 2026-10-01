import rough from "roughjs";
import { getStroke } from "perfect-freehand";
import type { RenderElement } from "./types";
import { type RenderAssets } from "./assets";
import { archivedImage, escapeXml, imageTag, textImage } from "./raster";

function arrowhead(
  kind: RenderElement["end_arrowhead"],
  from: [number, number],
  to: [number, number],
  color: string,
  size: number,
): string {
  if (!kind) {
    return "";
  }
  const angle = (Math.atan2(to[1] - from[1], to[0] - from[0]) * 180) / Math.PI;
  let shape: string;
  switch (kind) {
    case "dot":
      shape = `<circle r="${size / 3}" fill="${color}"/>`;
      break;
    case "diamond":
      shape = `<path d="M0 0 L${-size / 2} ${size / 3} L${-size} 0 L${-size / 2} ${-size / 3} Z" fill="${color}"/>`;
      break;
    case "bar":
      shape = `<path d="M0 ${-size / 2} L0 ${size / 2}"/>`;
      break;
    default:
      shape = `<path d="M${-size} ${-size / 2} L0 0 L${-size} ${size / 2}${kind === "triangle" ? " Z" : ""}" fill="${kind === "triangle" ? color : "none"}"/>`;
  }
  return `<g transform="translate(${to.join(" ")}) rotate(${angle})" stroke="${color}">${shape}</g>`;
}

export async function drawElement(element: RenderElement, assets: RenderAssets): Promise<string> {
  const { width, height } = element;
  const generator = rough.generator();
  const options = {
    seed: element.seed,
    roughness: element.roughness,
    stroke: element.stroke,
    strokeWidth: element.stroke_width,
    fill: element.fill === "transparent" ? undefined : element.fill,
    fillStyle: element.fill_style,
    strokeLineDash: { dashed: [8, 8], dotted: [2, 6], solid: undefined }[element.stroke_style],
  };
  let body = "";
  if (element.type === "text") {
    const text = await textImage(
      `<span foreground="${element.stroke}">${escapeXml(element.text ?? "")}</span>`,
      Math.max(1, width),
      element.font_size ?? 20,
      element.align,
    );
    body = imageTag(text.data, 0, 0, text.info.width, text.info.height);
  } else if (element.type === "image" && element.asset) {
    body = imageTag(
      await archivedImage(
        await assets.read(element.asset),
        Math.max(1, width),
        Math.max(1, height),
        "contain",
      ),
      0,
      0,
      width,
      height,
    );
  } else if (element.type === "freedraw") {
    const points = getStroke(element.points ?? [], {
      size: Math.max(1, element.stroke_width * 2),
      simulatePressure: true,
    });
    body = `<path d="${points.map((point, index) => `${index ? "L" : "M"}${point.join(" ")}`).join(" ")} Z" fill="${element.stroke}"/>`;
  } else {
    const points = element.points ?? [];
    let shape;
    switch (element.type) {
      case "ellipse":
        shape = generator.ellipse(width / 2, height / 2, width, height, options);
        break;
      case "diamond":
        shape = generator.polygon(
          [
            [width / 2, 0],
            [width, height / 2],
            [width / 2, height],
            [0, height / 2],
          ],
          options,
        );
        break;
      case "line":
      case "arrow":
        shape = generator.linearPath(points, options);
        break;
      default:
        shape = generator.rectangle(0, 0, width, height, options);
    }
    body = generator
      .toPaths(shape)
      .map(
        (path) =>
          `<path d="${path.d}" stroke="${path.stroke}" stroke-width="${path.strokeWidth}" fill="${path.fill}"${options.strokeLineDash ? ` stroke-dasharray="${options.strokeLineDash.join(" ")}"` : ""}/>`,
      )
      .join("");
    if (element.type === "arrow" && points.length >= 2) {
      const size = Math.max(10, element.stroke_width * 6);
      body += arrowhead(element.start_arrowhead, points[1], points[0], element.stroke, size);
      body += arrowhead(
        element.end_arrowhead,
        points[points.length - 2],
        points[points.length - 1],
        element.stroke,
        size,
      );
    }
  }
  return `<g opacity="${element.opacity}" transform="translate(${element.x} ${element.y}) rotate(${(element.angle * 180) / Math.PI} ${width / 2} ${height / 2})">${body}</g>`;
}
