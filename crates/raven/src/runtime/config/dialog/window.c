#include <gtk/gtk.h>

static void copy_error(GtkButton *button, gpointer message) {
    GdkClipboard *clipboard = gtk_widget_get_clipboard(GTK_WIDGET(button));
    gdk_clipboard_set_text(clipboard, message);
    gtk_button_set_label(button, "Copied");
}

static gboolean key_pressed(GtkEventControllerKey *controller, guint key,
        guint code, GdkModifierType modifiers, gpointer window) {
    (void)controller; (void)code; (void)modifiers;
    if (key != GDK_KEY_Escape) return FALSE;
    gtk_window_close(GTK_WINDOW(window));
    return TRUE;
}

static void activate(GtkApplication *app, gpointer message) {
    GtkWidget *window = gtk_application_window_new(app);
    /* Raven owns the frame; do not add GTK titlebars or CSD shadows. */
    gtk_window_set_decorated(GTK_WINDOW(window), FALSE);
    gtk_window_set_title(GTK_WINDOW(window), "Raven configuration error");
    gtk_window_set_default_size(GTK_WINDOW(window), 720, 440);
    /* Fixed-size XDG hints use Raven's ordinary floating-dialog admission. */
    gtk_window_set_resizable(GTK_WINDOW(window), FALSE);
    GtkWidget *layout = gtk_box_new(GTK_ORIENTATION_VERTICAL, 16);
    gtk_widget_set_margin_top(layout, 24);
    gtk_widget_set_margin_bottom(layout, 24);
    gtk_widget_set_margin_start(layout, 24);
    gtk_widget_set_margin_end(layout, 24);
    gtk_window_set_child(GTK_WINDOW(window), layout);
    GtkWidget *heading = gtk_label_new("Configuration was not applied");
    gtk_widget_add_css_class(heading, "title-2");
    gtk_label_set_xalign(GTK_LABEL(heading), 0);
    gtk_box_append(GTK_BOX(layout), heading);
    GtkWidget *hint = gtk_label_new("Fix the error and save raven.lua to retry automatically.");
    gtk_label_set_xalign(GTK_LABEL(hint), 0);
    gtk_label_set_wrap(GTK_LABEL(hint), TRUE);
    gtk_box_append(GTK_BOX(layout), hint);
    GtkWidget *scroll = gtk_scrolled_window_new();
    gtk_widget_set_vexpand(scroll, TRUE);
    gtk_scrolled_window_set_has_frame(GTK_SCROLLED_WINDOW(scroll), TRUE);
    GtkWidget *text = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(text), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(text), FALSE);
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(text), TRUE);
    gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(text), GTK_WRAP_WORD_CHAR);
    gtk_text_view_set_left_margin(GTK_TEXT_VIEW(text), 12);
    gtk_text_view_set_right_margin(GTK_TEXT_VIEW(text), 12);
    gtk_text_view_set_top_margin(GTK_TEXT_VIEW(text), 12);
    gtk_text_view_set_bottom_margin(GTK_TEXT_VIEW(text), 12);
    gtk_text_buffer_set_text(gtk_text_view_get_buffer(GTK_TEXT_VIEW(text)), message, -1);
    gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scroll), text);
    gtk_box_append(GTK_BOX(layout), scroll);
    GtkWidget *buttons = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 12);
    gtk_widget_set_halign(buttons, GTK_ALIGN_END);
    GtkWidget *close = gtk_button_new_with_label("Close");
    g_signal_connect_swapped(close, "clicked", G_CALLBACK(gtk_window_close), window);
    GtkWidget *copy = gtk_button_new_with_label("Copy error");
    gtk_widget_add_css_class(copy, "suggested-action");
    g_signal_connect(copy, "clicked", G_CALLBACK(copy_error), message);
    gtk_box_append(GTK_BOX(buttons), close);
    gtk_box_append(GTK_BOX(buttons), copy);
    gtk_box_append(GTK_BOX(layout), buttons);
    GtkEventController *keys = gtk_event_controller_key_new();
    g_signal_connect(keys, "key-pressed", G_CALLBACK(key_pressed), window);
    gtk_widget_add_controller(window, keys);
    gtk_window_present(GTK_WINDOW(window));
}

int raven_config_dialog(const char *message) {
    GtkApplication *app = gtk_application_new("org.raven.ConfigError", G_APPLICATION_NON_UNIQUE);
    g_signal_connect(app, "activate", G_CALLBACK(activate), (gpointer)message);
    int result = g_application_run(G_APPLICATION(app), 0, NULL);
    g_object_unref(app);
    return result;
}
