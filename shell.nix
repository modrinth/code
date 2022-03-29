{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc cargo clippy openssl pkg-config
    gtk4 gdk-pixbuf atk webkitgtk
  ];
}
