#include <gdk-pixbuf/gdk-pixbuf.h>
#include <stdbool.h>
bool raven_screenshot_png(const unsigned char *pixels, int w, int h, char **data, size_t *length) {
    if (w <= 0 || h <= 0 || w > 65536 || h > 65536 || (long long)w * h * 4 > 256LL * 1024 * 1024) return false;
    GdkPixbuf *image = gdk_pixbuf_new_from_data(pixels, GDK_COLORSPACE_RGB, TRUE, 8, w, h, w * 4, NULL, NULL);
    if (!image) return false;
    bool ok = gdk_pixbuf_save_to_buffer(image, data, length, "png", NULL, "compression", "6", NULL);
    g_object_unref(image);
    return ok;
}
const char *raven_screenshot_pictures(void) { return g_get_user_special_dir(G_USER_DIRECTORY_PICTURES); }
