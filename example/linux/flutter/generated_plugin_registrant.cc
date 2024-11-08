//
//  Generated file. Do not edit.
//

// clang-format off

#include "generated_plugin_registrant.h"

#include <audiopc/audiopc_plugin.h>

void fl_register_plugins(FlPluginRegistry* registry) {
  g_autoptr(FlPluginRegistrar) audiopc_registrar =
      fl_plugin_registry_get_registrar_for_plugin(registry, "AudiopcPlugin");
  audiopc_plugin_register_with_registrar(audiopc_registrar);
}
