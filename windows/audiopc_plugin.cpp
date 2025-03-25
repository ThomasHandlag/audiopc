#include "audiopc_plugin.h"

// This must be included before many other Windows headers.
#include <windows.h>

// For getPlatformVersion; remove unless needed for your plugin implementation.
#include <VersionHelpers.h>
#include <flutter/method_channel.h>
#include <flutter/plugin_registrar_windows.h>
#include <flutter/standard_method_codec.h>
#include <flutter/event_channel.h>
#include <iostream>
#include <string>
#include <thread>
#include <cassert>
#include <Mferror.h>
#include "event_stream_handler.h"
#include "metadata.h"

#pragma comment(lib, "mfplat")
#pragma comment(lib, "mf")
#pragma comment(lib, "mfreadwrite")
#pragma comment(lib, "mfuuid")
#pragma comment(lib, "Shlwapi")

namespace audiopc {
	using std::cout, std::endl, std::hex, std::make_unique, std::unique_ptr;
	using std::get, std::string, std::wstring, std::vector, std::move;

	HWND audiopc::AudiopcPlugin::hwnd = nullptr;
	shared_ptr<EventSink<EncodableValue>> audiopc::EventStreamHandler::_sink = nullptr;


	// static
	void AudiopcPlugin::RegisterWithRegistrar(
		flutter::PluginRegistrarWindows* registrar) {
		auto channel =
			make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
				registrar->messenger(), "audiopc/methodChannel",
				&flutter::StandardMethodCodec::GetInstance());
		auto plugin = make_unique<AudiopcPlugin>();

		channel->SetMethodCallHandler(
			[plugin_pointer = plugin.get()](const auto& call, auto result) {
				plugin_pointer->HandleMethodCall(call, move(result));
			});
		registrar->AddPlugin(move(plugin));
		AudiopcPlugin::hwnd = registrar->GetView()->GetNativeWindow();

		auto eventChannel = make_unique<flutter::EventChannel<flutter::EncodableValue>>(
			registrar->messenger(), "audiopc/eventChannel",
			&flutter::StandardMethodCodec::GetInstance());

		auto eventHandler = make_unique<EventStreamHandler>();
		eventChannel->SetStreamHandler(move(eventHandler));
	}

	void AudiopcPlugin::HandleMethodCall(
		const flutter::MethodCall<flutter::EncodableValue>& method_call,
		unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {
		flutter::EncodableMap map = get<flutter::EncodableMap>(*method_call.arguments());


		if (method_call.method_name().compare("init") == 0) {
			HRESULT hr = S_OK;
			auto value = map.find(flutter::EncodableValue("id"));
			if (value == map.end()) {
				result->Error("Error", "ID is required, found 0");
			}
			string id = get<string>(value->second);
			unique_ptr<AudioPlayer> player;
			hr = AudioPlayer::CreateInstance(&player, AudioPlayer::m_playerCount, id, &audiopc::EventStreamHandler::_sink);
			if (SUCCEEDED(hr)) {
				player->SetHWND(audiopc::AudiopcPlugin::hwnd);
				players.insert({ id, std::move(player) });
			}
			else {
				result->Error("Error", "Error creating player");
			}
		}
		else if (method_call.method_name().compare("getMetaData") == 0) {
			auto value = map.find(flutter::EncodableValue("path"));
			if (value == map.end()) {
				result->Error("Error", "Empty path");
			}
			string pathStr = get<string>(value->second);
			wstring pathWStr(pathStr.begin(), pathStr.end());
			AudioMetaData audioMetaData(pathWStr);
			flutter::EncodableMap rs;
			for (auto& [key, val] : audioMetaData.metaData) {
				rs[key] = val;
			}
			result->Success(flutter::EncodableValue(rs));
		}
		else
		{
			auto value = map.find(flutter::EncodableValue("id"));
			if (value == map.end()) {
				result->Error("Error", "ID is required, found 0");
			}
			string id = get<string>(value->second);
			AudioPlayer* player = players[id].get();

			if (player) {
				if (method_call.method_name().compare("setSource") == 0) {
					auto path = map.find(flutter::EncodableValue("path"));

					if (path == map.end()) {
						result->Error("Error", "Empty path");
					}

					string pathStr = get<string>(path->second);
					wstring pathWStr(pathStr.begin(), pathStr.end());

					if (player->m_path.empty()) {
						HRESULT hr = player->SetSource(pathWStr.c_str());
						if (SUCCEEDED(hr)) {
							player->m_path = pathWStr;
							result->Success();
						}
						else {
							result->Error("Error", "Error setting source");
						}
					}
					else {
						if (player->m_path.compare(pathWStr) != 0) {
							HRESULT hr = player->SetSource(pathWStr.c_str());
							if (SUCCEEDED(hr)) {
								player->m_path = pathWStr;
								result->Success();
							}
							else {
								result->Error("Error", "Error setting source");
							}
						}
						else {
							return;
						}
					}
				}
				else if (method_call.method_name().compare("close") == 0) {
					players[id].reset(nullptr);
					players.erase(id);
					result->Success();
				}
				else if (method_call.method_name().compare("play") == 0) {
					HRESULT hr = S_OK;
					hr = player->Play();
					thread poolThread(&AudioPlayer::StartAudioPool, player);
					// detach the audio pool thread from the main thread to prevent blocking UI thread
					poolThread.detach();
					result->Success();
				}
				else if (method_call.method_name().compare("stop") == 0) {
					player->Stop();
					result->Success();
				}
				else if (method_call.method_name().compare("pause") == 0) {
					player->Pause();
					result->Success();
				}
				else if (method_call.method_name().compare("getPosition") == 0) {
					double position = 0;
					HRESULT hr = player->GetPositionSecond(position);
					if (SUCCEEDED(hr)) {
						result->Success(flutter::EncodableValue(position));
					}
				}
				else if (method_call.method_name().compare("seek") == 0) {
					auto position = map.find(flutter::EncodableValue("position"));

					if (position == map.end()) {
						result->Error("Error", "Position is not provided");
					}

					double positionValue = get<double>(position->second);

					MFTIME time = static_cast<MFTIME>(positionValue * MICRO_TO_SECOND);
					HRESULT hr = player->SetPosition(time);
					if (SUCCEEDED(hr)) {
						result->Success(flutter::EncodableValue(1.0));
					}
					else {
						result->Error("Error", "Error setting position");
					}
				}
				else if (method_call.method_name().compare("setRate") == 0) {
					auto rate = map.find(flutter::EncodableValue("rate"));
					if (rate == map.end()) {
						result->Error("Error", "Rate is not provided");
					}
					double rateValue = get<double>(rate->second);
					player->SetRate(static_cast<float>(rateValue));

				}
				else if (method_call.method_name().compare("getSamples") == 0) {
					vector<double> samples = vector<double>(0, 0);
					player->GetSamples(samples);
					result->Success(flutter::EncodableValue(samples));
				}
				else {
					result->NotImplemented();
				}
			}
			else {
				result->Error("Error", "Player is not available");
			}
		}
	}

	AudiopcPlugin::AudiopcPlugin() {
	}

	AudiopcPlugin::~AudiopcPlugin() {
		players.clear();
	}
}  // namespace audiopc