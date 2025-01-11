#ifndef FLUTTER_PLUGIN_AUDIOPC_PLUGIN_H_
#define FLUTTER_PLUGIN_AUDIOPC_PLUGIN_H_

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>

// WMF headers
#include <windows.h>
#include <mfapi.h>
#include <mfidl.h>
#include <Shlwapi.h>
#include <stdio.h>
#include <new>
#include <vector>
#include <thread>
#include <iostream>
#include <stdexcept>
#include "audio_samples_grabber.h"
#include "audio_player.h"
#include <map>

namespace audiopc {

	using std::map;

	constexpr int MAX_PLAYERS = 128;

	class AudiopcPlugin : public flutter::Plugin {
	public:
		map<std::string, std::unique_ptr<AudioPlayer>> players;

		static std::unique_ptr<EventStreamHandler> eventHandler;

		static void RegisterWithRegistrar(flutter::PluginRegistrarWindows* registrar);
		static HWND hwnd;

		AudiopcPlugin();

		virtual ~AudiopcPlugin();

		// Disallow copy and assign.
		AudiopcPlugin(const AudiopcPlugin&) = delete;
		AudiopcPlugin& operator=(const AudiopcPlugin&) = delete;

		// Called when a method is called on this plugin's channel from Dart.
		void HandleMethodCall(
			const flutter::MethodCall<flutter::EncodableValue>& method_call,
			std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result);
	};

	// AudioPlayer class
	

}  // namespace audiopc

#endif  // FLUTTER_PLUGIN_AUDIOPC_PLUGIN_H_