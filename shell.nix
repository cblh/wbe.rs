{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  shellHook = ''
    # INFO winit::platform_impl::platform::x11::window: Guessed window scale factor: 1.1666666666666667
    # export WINIT_X11_SCALE_FACTOR=1

    # Times New Roman
    export WBE_FONT_PATH=${pkgs.corefonts.outPath}/share/fonts/truetype/Times_New_Roman.ttf
    export WBE_FONT_PATH_B=${pkgs.corefonts.outPath}/share/fonts/truetype/Times_New_Roman_Bold.ttf
    export WBE_FONT_PATH_I=${pkgs.corefonts.outPath}/share/fonts/truetype/Times_New_Roman_Italic.ttf
    export WBE_FONT_PATH_BI=${pkgs.corefonts.outPath}/share/fonts/truetype/Times_New_Roman_Bold_Italic.ttf

    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libglvnd
    ]}
  '';

  buildInputs = [
      pkgs.xorg.libX11
      pkgs.xorg.libXcursor
      pkgs.xorg.libXrandr
      pkgs.xorg.libXi
      pkgs.libglvnd
  ] ++ (if pkgs.stdenv.isDarwin then [
    pkgs.darwin.apple_sdk.frameworks.Security
    pkgs.libiconv
    pkgs.darwin.apple_sdk.frameworks.OpenGL
    pkgs.darwin.apple_sdk.frameworks.CoreServices
    pkgs.darwin.apple_sdk.frameworks.AppKit
  ] else []);
}
