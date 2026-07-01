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
| pens/brushes/fonts | planned | object table replay |
| clipping | planned | diagnostic fallback |
| raster/bitmaps | unsupported | diagnostic only |
| OpenGL / printer escapes | unsupported | diagnostic only |

Unsupported records are reported in Drawing IR diagnostics rather than silently dropped.
