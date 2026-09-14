#include <cairo.h>
#include <pango/pangocairo.h>
#include <stdbool.h>
#include <math.h>
bool raven_screenshot_hud(unsigned char *pixels, int w, int h, int scale, const char *text) {
    cairo_surface_t *surface = cairo_image_surface_create_for_data(pixels, CAIRO_FORMAT_ARGB32, w, h, w * 4);
    cairo_t *cr = cairo_create(surface);
    cairo_scale(cr, scale, scale);
    double width = (double)w / scale, height = (double)h / scale, r = 12;
    cairo_new_sub_path(cr);
    cairo_arc(cr, width-r, r, r, -G_PI/2, 0); cairo_arc(cr, width-r, height-r, r, 0, G_PI/2);
    cairo_arc(cr, r, height-r, r, G_PI/2, G_PI); cairo_arc(cr, r, r, r, G_PI, 3*G_PI/2);
    cairo_close_path(cr); cairo_set_source_rgba(cr, .07, .08, .10, .96); cairo_fill_preserve(cr);
    cairo_set_source_rgba(cr, 1, 1, 1, .20); cairo_set_line_width(cr, 1); cairo_stroke(cr);
    PangoFontMap *fonts = pango_cairo_font_map_new();
    PangoContext *context = pango_font_map_create_context(fonts);
    pango_cairo_update_context(cr, context);
    PangoLayout *layout = pango_layout_new(context);
    PangoFontDescription *font = pango_font_description_from_string("sans 11");
    pango_font_description_set_absolute_size(font, 13 * PANGO_SCALE);
    pango_layout_set_font_description(layout, font);
    pango_layout_set_text(layout, text, -1); pango_layout_set_alignment(layout, PANGO_ALIGN_CENTER);
    pango_layout_set_width(layout, (width - 24) * PANGO_SCALE);
    pango_layout_set_height(layout, (height - 16) * PANGO_SCALE);
    pango_layout_set_ellipsize(layout, PANGO_ELLIPSIZE_END);
    int tw, th; pango_layout_get_pixel_size(layout, &tw, &th);
    cairo_move_to(cr, 12, (height-th)/2); cairo_set_source_rgb(cr, .96, .97, .99); pango_cairo_show_layout(cr, layout);
    pango_font_description_free(font); g_object_unref(layout); g_object_unref(context); g_object_unref(fonts);
    cairo_surface_flush(surface);
    bool ok = cairo_status(cr) == CAIRO_STATUS_SUCCESS && cairo_surface_status(surface) == CAIRO_STATUS_SUCCESS;
    cairo_destroy(cr); cairo_surface_destroy(surface); return ok;
}
