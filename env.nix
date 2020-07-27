with import <nixpkgs> {};

stdenv.mkDerivation rec {
    name = "rust";
    buildInputs = [
	    zlib
        sqlite
        openssl
        pkg-config
    ];

    LD_LIBRARY_PATH=lib.makeLibraryPath buildInputs;
}
