#include "artwork.h"
#include <stdint.h>
static void image(cairo_t *cr, GdkPixbuf *pixbuf, double x, double y, double scale) {
    int w = gdk_pixbuf_get_width(pixbuf), h = gdk_pixbuf_get_height(pixbuf);
    int channels = gdk_pixbuf_get_n_channels(pixbuf), stride = gdk_pixbuf_get_rowstride(pixbuf);
    const guchar *source = gdk_pixbuf_read_pixels(pixbuf);
    cairo_surface_t *surface = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, w, h);
    unsigned char *data = cairo_image_surface_get_data(surface);
    int dest_stride = cairo_image_surface_get_stride(surface);
    for (int row = 0; row < h; row++) for (int col = 0; col < w; col++) {
        const guchar *pixel = source + row * stride + col * channels;
        uint32_t a = channels == 4 ? pixel[3] : 255;
        ((uint32_t *)(data + row * dest_stride))[col] = (a << 24) | (((pixel[0] * a + 127) / 255) << 16) | (((pixel[1] * a + 127) / 255) << 8) | ((pixel[2] * a + 127) / 255);
    }
    cairo_surface_mark_dirty(surface);
    cairo_save(cr); cairo_translate(cr, x, y); cairo_scale(cr, 1 / scale, 1 / scale);
    cairo_set_source_surface(cr, surface, 0, 0); cairo_paint(cr);
    cairo_restore(cr); cairo_surface_destroy(surface);
}
bool raven_switcher_paint(void *data, uint8_t *pixels, int width, int height, int scale,
        int slot, int icon, int selected, int count, const char *const *ids,
        const char *const *titles, bool before, bool after) {
    RavenArt *art = data;
    if (!art || !pixels || width <= 0 || height <= 0 || scale <= 0 || count <= 0) return false;
    cairo_surface_t *surface = cairo_image_surface_create_for_data(pixels, CAIRO_FORMAT_ARGB32, width, height, width * 4);
    cairo_t *cr = cairo_create(surface);
    cairo_scale(cr, scale, scale);
    double w = (double)width / scale, h = (double)height / scale;
    // Soft external shadow around the opaque dark panel.
    for (int i = 12; i > 0; i--) {
        raven_round(cr, 12 - i * .65, 12 - i * .4, w - 24 + i * 1.3, h - 24 + i * 1.1, 36 + i * .65);
        cairo_set_source_rgba(cr, 0, 0, 0, .018); cairo_fill(cr);
    }
    raven_round(cr, 12, 12, w - 24, h - 24, 36);
    cairo_pattern_t *glass = cairo_pattern_create_linear(0, 12, 0, h - 12);
    cairo_pattern_add_color_stop_rgba(glass, 0, .17, .17, .18, 1);
    cairo_pattern_add_color_stop_rgba(glass, 1, .10, .10, .11, 1);
    cairo_set_source(cr, glass); cairo_fill_preserve(cr); cairo_pattern_destroy(glass);
    cairo_set_source_rgba(cr, 1, 1, 1, .16); cairo_set_line_width(cr, 1); cairo_stroke(cr);
    for (int i = 0; i < count; i++) {
        double x = 24 + i * slot, center = x + slot / 2.0;
        if (i == selected) {
            raven_round(cr, center - (icon + 16) / 2.0, 28, icon + 16, icon + 16, 25);
            cairo_set_source_rgba(cr, 1, 1, 1, .18); cairo_fill(cr);
        }
        RavenApp *app = raven_app(art, ids[i], titles[i]);
        GdkPixbuf *pixbuf = raven_icon(art, app->icon, icon * scale);
        if (pixbuf) {
            double iw = (double)gdk_pixbuf_get_width(pixbuf) / scale;
            double ih = (double)gdk_pixbuf_get_height(pixbuf) / scale;
            image(cr, pixbuf, center - iw / 2, 36 + (icon - ih) / 2, scale);
        } else {
            raven_round(cr, center - icon / 2.0, 36, icon, icon, 20);
            cairo_set_source_rgba(cr, .3, .32, .36, 1); cairo_fill(cr);
            char *initial = g_utf8_substring(app->name, 0, 1);
            cairo_set_source_rgba(cr, .94, .94, .96, 1);
            raven_text(art, cr, initial, center - icon / 2.0, 36 + icon * .2, icon, icon * .45);
            g_free(initial);
        }
        if (i == selected) {
            cairo_set_source_rgba(cr, .96, .96, .97, 1);
            raven_text(art, cr, app->name, x - 2, icon + 49, slot + 4, 13);
        }
    }
    cairo_set_source_rgba(cr, 1, 1, 1, .6);
    if (before) raven_text(art, cr, "‹", 12, h / 2 - 13, 14, 22);
    if (after) raven_text(art, cr, "›", w - 26, h / 2 - 13, 14, 22);
    bool ok = cairo_status(cr) == CAIRO_STATUS_SUCCESS;
    cairo_destroy(cr); cairo_surface_flush(surface); cairo_surface_destroy(surface);
    return ok;
}
