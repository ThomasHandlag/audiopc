#include "audiopc_plugin.h"

// This must be included before many other Windows headers.
#include <windows.h>

// For getPlatformVersion; remove unless needed for your plugin implementation.
#include <VersionHelpers.h>

#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>

#include <iostream>
#include <string>
#include <thread>

#include <cassert>
#include <Mferror.h>

#pragma comment(lib, "mfplat")
#pragma comment(lib, "mf")
#pragma comment(lib, "mfreadwrite")
#pragma comment(lib, "mfuuid")
#pragma comment(lib, "Shlwapi")


namespace audiopc {
	using std::cout, std::endl, std::hex, std::make_unique, std::unique_ptr;
	using std::get, std::string, std::wstring, std::vector;

	HWND audiopc::AudiopcPlugin::hwnd = nullptr;

	// static
	void AudiopcPlugin::RegisterWithRegistrar(
		flutter::PluginRegistrarWindows* registrar) {
		auto channel =
			make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
				registrar->messenger(), "audiopc",
				&flutter::StandardMethodCodec::GetInstance());
		auto plugin = make_unique<AudiopcPlugin>();

		channel->SetMethodCallHandler(
			[plugin_pointer = plugin.get()](const auto& call, auto result) {
				plugin_pointer->HandleMethodCall(call, move(result));
			});
		registrar->AddPlugin(move(plugin));
		AudiopcPlugin::hwnd = registrar->GetView()->GetNativeWindow();
	}

	void AudiopcPlugin::HandleMethodCall(
		const flutter::MethodCall<flutter::EncodableValue>& method_call,
		unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
		flutter::EncodableMap map = get<flutter::EncodableMap>(*method_call.arguments());

		if (player) {
			player->SetHWND(AudiopcPlugin::hwnd);
		}

		if (method_call.method_name().compare("setSource") == 0) {
			auto path = map.find(flutter::EncodableValue("path"));

			if (path == map.end()) {
				result->Error("Error", "Invalid path");
			}

			string pathStr = get<string>(path->second);
			wstring pathWStr(pathStr.begin(), pathStr.end());

			if (player) {
				HRESULT hr = player->SetSource(pathWStr.c_str());
				if (SUCCEEDED(hr)) {
					result->Success();
				}
				else {
					cout << "Error" << __FILE__ << ":" << static_cast<double>(__LINE__) << "-" << hex << hr << endl;
					result->Error("Error", "Error setting source");
				}
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("play") == 0) {
			if (player) {
				HRESULT hr = S_OK;
				hr = player->Play();
				thread poolThread(&AudioPlayer::StartAudioPool, player);
				// detach the audio pool thread from the main thread to prevent blocking UI thread
				poolThread.detach();
				result->Success(flutter::EncodableValue(true));
			}
			else {
				result->Error("Error", "Player is not initialized");
			}

		}
		else if (method_call.method_name().compare("stop") == 0) {
			if (player) {
				player->Stop();
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("pause") == 0) {
			if (player) {
				player->Pause();
				result->Success(flutter::EncodableValue(true));
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("getDuration") == 0) {
			if (player) {
				double duration = 0;
				HRESULT hr = player->GetSecondDuration(duration);

				if (SUCCEEDED(hr)) {
					result->Success(flutter::EncodableValue(duration));
				}
				else {
					result->Error("Error", "Error getting duration");
				}
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("getPostion") == 0) {
			if (player) {
				double position = 0;
				HRESULT hr = player->GetCDurationSecond(position);
				if (SUCCEEDED(hr)) {
					result->Success(flutter::EncodableValue(position));
				}
				else {
					result->Error("Error", "Error getting position");
				}
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("seek") == 0) {
			flutter::EncodableMap map = get<flutter::EncodableMap>(*method_call.arguments());
			auto position = map.find(flutter::EncodableValue("position"));

			if (position == map.end()) {
				result->Error("Error", "Position is not provided");
			}

			double positionValue = get<double>(position->second);

			if (player) {
				MFTIME time = static_cast<MFTIME>(positionValue * MICRO_TO_SECOND);
				HRESULT hr = player->SetPosition(time);
				if (SUCCEEDED(hr)) {
					result->Success(flutter::EncodableValue(1.0));
				}
				else {
					result->Error("Error", "Error setting position");
				}
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("getState") == 0) {
			if (player) {
				result->Success(flutter::EncodableValue((double)player->GetState()));
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("getSamples") == 0) {
			if (player) {
				vector<double> samples = vector<double>(44100, 0);
				player->GetSamples(samples);
				result->Success(flutter::EncodableValue(samples));
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}
		else if (method_call.method_name().compare("getPosition") == 0) {
			if (player) {
				double position = 0;
				HRESULT hr = player->GetCDurationSecond(position);
				if (SUCCEEDED(hr)) {
					result->Success(flutter::EncodableValue(position));
				}
				else {
					result->Error("Error", "Error getting position");
				}
			}
			else {
				result->Error("Error", "Player is not initialized");
			}
		}

		else {
			result->NotImplemented();
		}
	}

	AudiopcPlugin::AudiopcPlugin() {
		AudioPlayer::CreateInstance(&player);
	}

	AudiopcPlugin::~AudiopcPlugin() {
	}


	

	

	
}  // namespace audiopc