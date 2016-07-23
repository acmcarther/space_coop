# TODO: Untangle this crazy business
let
  pkgs = import <nixpkgs> {};
in pkgs.stdenv.mkDerivation rec {
  name = "space_coop";
  buildInputs = [ pkgs.freetype pkgs.mesa pkgs.cmake pkgs.xlibs.libX11 pkgs.xlibs.libXxf86vm pkgs.xlibs.libXi pkgs.xlibs.libXrandr pkgs.xlibs.libXinerama pkgs.xlibs.libXcursor pkgs.xlibs.libXext pkgs.xlibs.libXfixes ];
  C_INCLUDE_PATH = with pkgs.xlibs; "${pkgs.mesa}/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${pkgs.mesa_glu}/lib:${xlibsWrapper}/lib:${pkgs.freeglut}/lib:${libXext}/lib:${pkgs.glfw2}/lib:${pkgs.vtk}/lib:${libXrandr}/lib:${libXfixes}/lib:${libXinerama}/lib:${libXcursor}/lib:{${pkgs.mesa_drivers}/lib";
  CPLUS_INCLUDE_PATH = with pkgs.xlibs; "${pkgs.mesa}/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${pkgs.mesa_glu}/lib:${xlibsWrapper}/lib:${pkgs.freeglut}/lib:${libXext}/lib:${pkgs.glfw2}/lib:${pkgs.vtk}/lib:${libXrandr}/lib:${libXfixes}/lib:${libXinerama}/lib:${libXcursor}/lib:{${pkgs.mesa_drivers}/lib";
  CMAKE_LIBRARY_PATH = with pkgs.xlibs; "${pkgs.mesa}/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${pkgs.mesa_glu}/lib:${xlibsWrapper}/lib:${pkgs.freeglut}/lib:${libXext}/lib:${pkgs.glfw2}/lib:${pkgs.vtk}/lib:${libXrandr}/lib:${libXfixes}/lib:${libXinerama}/lib:${libXcursor}/lib:{${pkgs.mesa_drivers}/lib";
  LD_LIBRARY_PATH = with pkgs.xlibs; "${pkgs.mesa}/lib:${libX11}/lib:${libXcursor}/lib:${libXxf86vm}/lib:${libXi}/lib:${pkgs.mesa_glu}/lib:${xlibsWrapper}/lib:${pkgs.freeglut}/lib:${libXext}/lib:${pkgs.glfw2}/lib:${pkgs.vtk}/lib:${libXrandr}/lib:${libXfixes}/lib:${libXinerama}/lib:${libXcursor}/lib:{${pkgs.mesa_drivers}/lib";
}
