#pragma once
#include <stdbool.h>
#include <cairo.h>
#include <gio/gdesktopappinfo.h>
#include <gdk-pixbuf/gdk-pixbuf.h>
#include <pango/pangocairo.h>
typedef struct { char *name; GIcon *icon; } RavenApp;
typedef struct { GList *apps; GHashTable *metadata; GHashTable *images; PangoFontMap *fonts; char *theme; } RavenArt;
GIcon *raven_companion_icon(GList *apps, GAppInfo *selected);
RavenApp *raven_app(RavenArt *art, const char *id, const char *title);
GdkPixbuf *raven_icon(RavenArt *art, GIcon *icon, int size);
void raven_text(RavenArt *art, cairo_t *cr, const char *text, double x, double y, double width, double size);
void raven_round(cairo_t *cr, double x, double y, double w, double h, double radius);
