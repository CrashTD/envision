if [[ ! -d build ]]; then
    meson setup build -Dprefix="$PWD/build/testdir"
fi

ninja -C build envision-pot
