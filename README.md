# Building

1. Install Android NDK.
2. Install [dinghy](https://github.com/sonos/dinghy/blob/main/docs/android.md).
3. `rustup target install aarch64-linux-android`
4. `cargo dinghy -p auto-android-aarch64-api32 build -r`

The compiled `libetvr_openxr_layer.so` file will be located in `target/aarch64-linux-android/` in the debug or release folder.

# Patching the Steam Link APK

1. Get the Steam Link APK file using e.g. SideQuest to extract it from the headset.
2. Decompile the APK using apktool.
3. In the decompiled folder, put the `etvr-openxr-layer.json` file in the `assets/openxr/1/api_layers/implicit.d/` folder.
4. Put the `libetvr_openxr_layer.so` file in the `lib/arm64-v8a/` folder.
5. Build the APK with the changes using apktool.
6. Zipalign and sign the APK.

For now, a patched OpenXR loader is required as well, as the built-in one doesn't load implicit API layers properly.