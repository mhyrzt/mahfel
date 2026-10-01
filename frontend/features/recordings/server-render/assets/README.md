# Native text font

`Vazirmatn-Regular.ttf` is the unmodified upstream Vazirmatn v33.003 font:

https://github.com/rastikerdar/vazirmatn/blob/v33.003/fonts/ttf/Vazirmatn-Regular.ttf

SHA-256: `b69fd4c680b8f3f225feabcc655a2c585d97627b8f5f5c0f9985e894069f3a56`

The accompanying `OFL.txt` is its upstream SIL Open Font License. Sharp/Pango requires a native
font file; the browser's WOFF2 asset is not sufficient for the container's font loader. Next.js
output tracing includes these private assets in the standalone web application.
