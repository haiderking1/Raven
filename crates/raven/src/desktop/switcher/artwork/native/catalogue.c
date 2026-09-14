#include "artwork.h"
#include <string.h>
static void app_free(void *data) {
    RavenApp *app = data;
    g_free(app->name); g_clear_object(&app->icon); g_free(app);
}
void *raven_switcher_art_new(void) {
    RavenArt *art = g_new0(RavenArt, 1);
    art->apps = g_app_info_get_all();
    art->metadata = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, app_free);
    art->images = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_object_unref);
    art->fonts = pango_cairo_font_map_new();
    GSettingsSchemaSource *source = g_settings_schema_source_get_default();
    GSettingsSchema *schema = source ? g_settings_schema_source_lookup(source, "org.gnome.desktop.interface", TRUE) : NULL;
    if (schema && g_settings_schema_has_key(schema, "icon-theme")) {
        GSettings *settings = g_settings_new_full(schema, NULL, NULL);
        art->theme = g_settings_get_string(settings, "icon-theme");
        g_object_unref(settings);
    }
    if (schema) g_settings_schema_unref(schema);
    if (!art->theme || !*art->theme) { g_free(art->theme); art->theme = g_strdup("Adwaita"); }
    return art;
}
void raven_switcher_art_free(void *data) {
    RavenArt *art = data;
    if (!art) return;
    g_list_free_full(art->apps, g_object_unref);
    g_hash_table_unref(art->metadata); g_hash_table_unref(art->images);
    g_object_unref(art->fonts); g_free(art->theme); g_free(art);
}
static bool matches(const char *candidate, const char *id) {
    if (!candidate) return false;
    char *normalized = g_ascii_strdown(candidate, -1);
    if (g_str_has_suffix(normalized, ".desktop")) normalized[strlen(normalized) - 8] = 0;
    bool equal = g_str_equal(normalized, id);
    g_free(normalized); return equal;
}
RavenApp *raven_app(RavenArt *art, const char *id, const char *title) {
    RavenApp *cached = g_hash_table_lookup(art->metadata, id);
    if (cached) return cached;
    if (g_hash_table_size(art->metadata) >= 512) g_hash_table_remove_all(art->metadata);
    RavenApp *app = g_new0(RavenApp, 1);
    GAppInfo *best = NULL;
    int best_score = 0;
    for (GList *item = art->apps; item; item = item->next) {
        GAppInfo *info = item->data;
        const char *wmclass = G_IS_DESKTOP_APP_INFO(info) ? g_desktop_app_info_get_startup_wm_class(G_DESKTOP_APP_INFO(info)) : NULL;
        int score = matches(g_app_info_get_id(info), id) ? 3 : matches(wmclass, id) ? (g_app_info_should_show(info) ? 2 : 1) : 0;
        if (score > best_score) { best = info; best_score = score; }
    }
    if (best) {
        app->name = g_strdup(g_app_info_get_display_name(best));
        GIcon *icon = g_app_info_get_icon(best);
        if (!icon) icon = raven_companion_icon(art->apps, best);
        if (icon) app->icon = g_object_ref(icon);
    }
    if (!app->name) app->name = g_utf8_make_valid(*title ? title : id, -1);
    if (!app->icon) app->icon = g_themed_icon_new("application-x-executable");
    g_hash_table_insert(art->metadata, g_strdup(id), app);
    return app;
}
