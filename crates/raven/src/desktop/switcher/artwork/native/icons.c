#include "artwork.h"
#include <limits.h>
#include <stdlib.h>
#include <string.h>

static GdkPixbuf *load(const char *path, int size) {
    return gdk_pixbuf_new_from_file_at_scale(path, size, size, TRUE, NULL);
}
static GPtrArray *roots(void) {
    GPtrArray *paths = g_ptr_array_new_with_free_func(g_free);
    g_ptr_array_add(paths, g_build_filename(g_get_user_data_dir(), "icons", NULL));
    g_ptr_array_add(paths, g_build_filename(g_get_home_dir(), ".icons", NULL));
    const char *const *dirs = g_get_system_data_dirs();
    for (int i = 0; dirs[i]; i++) g_ptr_array_add(paths, g_build_filename(dirs[i], "icons", NULL));
    return paths;
}
static GdkPixbuf *theme_icon(GPtrArray *paths, const char *theme, const char *name, int size, GHashTable *visited) {
    if (!theme || !*theme || strchr(theme, '/') || g_hash_table_contains(visited, theme) || g_hash_table_size(visited) >= 32) return NULL;
    g_hash_table_add(visited, g_strdup(theme));
    char *best = NULL;
    int score = INT_MAX;
    GPtrArray *parents = g_ptr_array_new_with_free_func(g_free);
    for (guint root = 0; root < paths->len; root++) {
        const char *base = g_ptr_array_index(paths, root);
        char *index = g_build_filename(base, theme, "index.theme", NULL);
        GKeyFile *file = g_key_file_new();
        bool loaded = g_key_file_load_from_file(file, index, G_KEY_FILE_NONE, NULL);
        // A theme can be spread across XDG roots. A user icon directory need
        // not repeat the system theme’s index.theme.
        for (guint other = 0; !loaded && other < paths->len; other++) {
            char *fallback = g_build_filename(g_ptr_array_index(paths, other), theme, "index.theme", NULL);
            loaded = g_key_file_load_from_file(file, fallback, G_KEY_FILE_NONE, NULL);
            g_free(fallback);
        }
        if (!loaded) { g_free(index); g_key_file_unref(file); continue; }
        g_free(index);
        char *inherits = g_key_file_get_string(file, "Icon Theme", "Inherits", NULL);
        char **names = g_strsplit(inherits ? inherits : "", ",", -1);
        for (int i = 0; names[i]; i++) if (*g_strstrip(names[i])) g_ptr_array_add(parents, g_strdup(names[i]));
        g_strfreev(names); g_free(inherits);
        const char *keys[] = {"Directories", "ScaledDirectories"};
        for (int k = 0; k < 2; k++) {
            char *directories = g_key_file_get_string(file, "Icon Theme", keys[k], NULL);
            char **dirs = g_strsplit(directories ? directories : "", ",", -1);
            for (int d = 0; dirs[d]; d++) {
                const char *dir = g_strstrip(dirs[d]);
                if (!*dir) continue;
                int nominal = CLAMP(g_key_file_get_integer(file, dir, "Size", NULL), 1, 16384);
                int scale = g_key_file_get_integer(file, dir, "Scale", NULL);
                if (scale <= 0) scale = 1;
                char *type = g_key_file_get_string(file, dir, "Type", NULL);
                int low = nominal, high = nominal;
                if (g_strcmp0(type, "Scalable") == 0) {
                    low = g_key_file_get_integer(file, dir, "MinSize", NULL);
                    high = g_key_file_get_integer(file, dir, "MaxSize", NULL);
                    if (low <= 0) low = nominal;
                    if (high <= 0) high = nominal;
                } else if (!type || g_str_equal(type, "Threshold")) {
                    int threshold = g_key_file_has_key(file, dir, "Threshold", NULL) ? g_key_file_get_integer(file, dir, "Threshold", NULL) : 2;
                    threshold = CLAMP(threshold, 0, 16384);
                    low -= threshold; high += threshold;
                }
                g_free(type);
                double distance = size < (double)low * scale ? (double)low * scale - size : size > (double)high * scale ? size - (double)high * scale : 0;
                int current = (int)MIN(distance, INT_MAX);
                if (current >= score) continue;
                const char *extensions[] = {"png", "svg", "xpm"};
                for (int ext = 0; ext < 3; ext++) {
                    char *filename = g_strdup_printf("%s.%s", name, extensions[ext]);
                    char *path = g_build_filename(base, theme, dir, filename, NULL);
                    g_free(filename);
                    if (g_file_test(path, G_FILE_TEST_IS_REGULAR)) { g_free(best); best = path; score = current; break; }
                    g_free(path);
                }
            }
            g_strfreev(dirs); g_free(directories);
        }
        g_key_file_unref(file);
    }
    GdkPixbuf *image = best ? load(best, size) : NULL;
    g_free(best);
    for (guint i = 0; !image && i < parents->len; i++) image = theme_icon(paths, g_ptr_array_index(parents, i), name, size, visited);
    g_ptr_array_unref(parents);
    return image;
}
GdkPixbuf *raven_icon(RavenArt *art, GIcon *icon, int size) {
    char *serialized = g_icon_to_string(icon);
    char *key = g_strdup_printf("%s:%d", serialized ? serialized : "", size);
    g_free(serialized);
    GdkPixbuf *image = g_hash_table_lookup(art->images, key);
    if (image) { g_free(key); return image; }
    if (G_IS_FILE_ICON(icon)) {
        char *path = g_file_get_path(g_file_icon_get_file(G_FILE_ICON(icon)));
        if (path) image = load(path, size);
        g_free(path);
    } else if (G_IS_THEMED_ICON(icon)) {
        const char *const *names = g_themed_icon_get_names(G_THEMED_ICON(icon));
        GPtrArray *paths = roots();
        for (int n = 0; names[n] && !image; n++) {
            if (strchr(names[n], '/')) continue;
            GHashTable *visited = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, NULL);
            image = theme_icon(paths, art->theme, names[n], size, visited);
            if (!image) image = theme_icon(paths, "hicolor", names[n], size, visited);
            g_hash_table_unref(visited);
            if (!image) {
                const char *const *dirs = g_get_system_data_dirs();
                for (int d = 0; dirs[d] && !image; d++) {
                    const char *extensions[] = {"png", "svg", "xpm"};
                    for (int e = 0; e < 3 && !image; e++) {
                        char *file = g_strdup_printf("%s.%s", names[n], extensions[e]);
                        char *path = g_build_filename(dirs[d], "pixmaps", file, NULL);
                        image = load(path, size); g_free(path); g_free(file);
                    }
                }
            }
        }
        g_ptr_array_unref(paths);
    }
    if (image) {
        if (g_hash_table_size(art->images) >= 64) g_hash_table_remove_all(art->images);
        g_hash_table_insert(art->images, key, image);
    } else g_free(key);
    return image;
}
