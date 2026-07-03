# EMF Support Matrix

| Record | Status | Notes |
| --- | --- | --- |
| `EMR_HEADER` | supported | structural validation |
| `EMR_EOF` | supported | required terminator |
| `EMR_RECTANGLE` | supported | mapped to IR rectangle |
| `EMR_POLYLINE` / `EMR_POLYLINE16` | supported | mapped to IR polyline |
| `EMR_POLYGON` / `EMR_POLYGON16` | supported | closed polyline |
| `EMR_MOVETOEX` / `EMR_LINETO` | partial | line/path replay |
| `EMR_EXTTEXTOUTA` / `EMR_EXTTEXTOUTW` | partial | basic text extraction |
| `EMR_CREATEPEN` / `EMR_EXTCREATEPEN` / `EMR_SELECTOBJECT` | partial | pen table + stroke replay |
| `EMR_SETMAPMODE` / world transforms | partial | scale factors applied to coordinates |
| brushes / fills | planned | not replayed |
| clipping | planned | diagnostic fallback |
| raster/bitmaps | unsupported | diagnostic only |
| OpenGL / printer escapes | unsupported | diagnostic only |

Unsupported records are reported in Drawing IR diagnostics rather than silently dropped.
