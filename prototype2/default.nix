# TODO: Untangle this crazy business
with (import <nixpkgs> {});
let
  pkgs = import <nixpkgs> {};
  env = bundlerEnv {
    name = "your-package";
    inherit ruby;
    gemfile = ./Gemfile;
    lockfile = ./Gemfile.lock;
    gemset = ./gemset.nix;
  };
in pkgs.stdenv.mkDerivation rec {
  name = "space_coop";
  buildInputs = [ pkgs.mesa_glu pkgs.freetype pkgs.mesa pkgs.cmake pkgs.xlibs.libX11 pkgs.xlibs.libXxf86vm pkgs.xlibs.libXi pkgs.xlibs.libXrandr pkgs.xlibs.libXinerama pkgs.xlibs.libXcursor pkgs.xlibs.libXext pkgs.xlibs.libXfixes pkgs.ruby env];
  LD_LIBRARY_PATH = with pkgs.xlibs; "/run/opengl-driver/lib:/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${xlibsWrapper}/lib:${pkgs.freeglut}/lib:${libXext}/lib:${pkgs.glfw2}/lib:${pkgs.vtk}/lib:${libXrandr}/lib:${libXfixes}/lib:${libXinerama}/lib:${libXcursor}/lib";
}
