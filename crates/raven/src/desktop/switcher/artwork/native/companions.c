#include "artwork.h"

// Generated URI-handler entries can have an exact app ID but omit their Icon.
// Match declared metadata, not executable wrappers, guessed paths, or app lists.
static char *name_key(GAppInfo *app) {
    const char *name = g_app_info_get_name(app);
    char *folded = g_utf8_casefold(name ? name : "", -1);
    GString *key = g_string_new(NULL);
    for (const char *p = folded; *p; p = g_utf8_next_char(p)) {
        gunichar c = g_utf8_get_char(p);
        if (g_unichar_isalnum(c)) g_string_append_unichar(key, c);
    }
    g_free(folded);
    return g_string_free(key, FALSE);
}
static bool shared_scheme(GAppInfo *a, GAppInfo *b) {
    const char *const *first = g_app_info_get_supported_types(a);
    const char *const *second = g_app_info_get_supported_types(b);
    for (int i = 0; first && first[i]; i++) {
        if (!g_str_has_prefix(first[i], "x-scheme-handler/")) continue;
        for (int j = 0; second && second[j]; j++)
            if (g_str_equal(first[i], second[j])) return true;
    }
    return false;
}
GIcon *raven_companion_icon(GList *apps, GAppInfo *selected) {
    char *name = name_key(selected);
    GIcon *result = NULL;
    for (GList *item = apps; item && *name; item = item->next) {
        GAppInfo *candidate = item->data;
        GIcon *icon = g_app_info_get_icon(candidate);
        if (candidate == selected || !icon || !shared_scheme(selected, candidate)) continue;
        char *other = name_key(candidate);
        bool related = g_str_equal(name, other);
        g_free(other);
        if (!related) continue;
        // Multiple equivalent registrations are fine; conflicting icons are
        // ambiguous and must not silently choose an unrelated application.
        if (result && !g_icon_equal(result, icon)) { result = NULL; break; }
        result = icon;
    }
    g_free(name);
    return result; // borrowed from apps
}
