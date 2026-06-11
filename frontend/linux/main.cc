<<<<<<< HEAD
#include <flutter_linux/flutter_linux.h>
#include <gtk/gtk.h>

static void activate(GtkApplication* app, gpointer user_data) {
  GtkWidget* window = gtk_application_window_new(app);
  gtk_window_set_default_size(GTK_WINDOW(window), 1280, 800);
  gtk_window_set_title(GTK_WINDOW(window), "NeQST Office 1-2-3");

  FlDartProject* project = fl_dart_project_new();
  FlView* view = fl_view_new(project);

  gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));
  gtk_widget_show_all(window);
}

int main(int argc, char** argv) {
  GtkApplication* app = gtk_application_new("org.neqst.neqst_office_123",
                                            G_APPLICATION_FLAGS_NONE);
  g_signal_connect(app, "activate", G_CALLBACK(activate), nullptr);

  int status = g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);
  return status;
}

=======
#include <flutter_linux/flutter_linux.h>
#include <gtk/gtk.h>

static void activate(GtkApplication* app, gpointer user_data) {
  GtkWidget* window = gtk_application_window_new(app);
  gtk_window_set_default_size(GTK_WINDOW(window), 1280, 800);
  gtk_window_set_title(GTK_WINDOW(window), "NeQST Office 1-2-3");

  FlDartProject* project = fl_dart_project_new();
  FlView* view = fl_view_new(project);

  gtk_container_add(GTK_CONTAINER(window), GTK_WIDGET(view));
  gtk_widget_show_all(window);
}

int main(int argc, char** argv) {
  GtkApplication* app = gtk_application_new("org.neqst.neqst_office_123",
                                            G_APPLICATION_FLAGS_NONE);
  g_signal_connect(app, "activate", G_CALLBACK(activate), nullptr);

  int status = g_application_run(G_APPLICATION(app), argc, argv);
  g_object_unref(app);
  return status;
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
