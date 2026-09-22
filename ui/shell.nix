# SPDX-License-Identifier: MIT OR Apache-2.0
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ pkg-config wrapGAppsHook3 ];
  buildInputs = with pkgs; [ gtk3 webkitgtk_4_1 libayatana-appindicator librsvg openssl xdotool ];
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [ gtk3 webkitgtk_4_1 libayatana-appindicator librsvg openssl xdotool glib cairo gdk-pixbuf libsoup_3 pango libglvnd mesa ]);
  LIBGL_DRIVERS_PATH = "${pkgs.mesa}/lib/dri";
  __EGL_VENDOR_LIBRARY_FILENAMES = "${pkgs.mesa}/share/glvnd/egl_vendor.d/50_mesa.json";
  packages = with pkgs; [ xvfb-run ];
}
