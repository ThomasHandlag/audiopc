#include "include/audiopc/audiopc_plugin_c_api.h"

#include <flutter/plugin_registrar_windows.h>

#include "audiopc_plugin.h"

void AudiopcPluginCApiRegisterWithRegistrar(
    FlutterDesktopPluginRegistrarRef registrar) {
  audiopc::AudiopcPlugin::RegisterWithRegistrar(
      flutter::PluginRegistrarManager::GetInstance()
          ->GetRegistrar<flutter::PluginRegistrarWindows>(registrar));
}
