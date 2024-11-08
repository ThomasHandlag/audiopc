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
#include <chrono>

#include <cassert>
#include <Mferror.h>

#pragma comment(lib, "mfplat")
#pragma comment(lib, "mf")
#pragma comment(lib, "mfreadwrite")
#pragma comment(lib, "mfuuid")
#pragma comment(lib, "Shlwapi")


namespace audiopc {
	using std::cout, std::endl;
	HWND audiopc::AudiopcPlugin::hwnd = nullptr;

	template <class Q>
	HRESULT GetEventObject(IMFMediaEvent* pEvent, Q** ppObject)
	{
		*ppObject = NULL;

		PROPVARIANT var;
		HRESULT hr = pEvent->GetValue(&var);
		if (SUCCEEDED(hr))
		{
			if (var.vt == VT_UNKNOWN)
			{
				hr = var.punkVal->QueryInterface(ppObject);
			}
			else
			{
				hr = MF_E_INVALIDTYPE;
			}
			PropVariantClear(&var);
		}
		return hr;
	}

	// static
	void AudiopcPlugin::RegisterWithRegistrar(
		flutter::PluginRegistrarWindows* registrar) {
		auto channel =
			std::make_unique<flutter::MethodChannel<flutter::EncodableValue>>(
				registrar->messenger(), "audiopc",
				&flutter::StandardMethodCodec::GetInstance());
		auto plugin = std::make_unique<AudiopcPlugin>();

		channel->SetMethodCallHandler(
			[plugin_pointer = plugin.get()](const auto& call, auto result) {
				plugin_pointer->HandleMethodCall(call, std::move(result));
			});
		registrar->AddPlugin(std::move(plugin));

		AudiopcPlugin::hwnd = registrar->GetView()->GetNativeWindow();
	}


	void AudiopcPlugin::HandleMethodCall(
		const flutter::MethodCall<flutter::EncodableValue>& method_call,
		std::unique_ptr<flutter::MethodResult<flutter::EncodableValue>> result) {

		if (player) {
			player->SetHWND(AudiopcPlugin::hwnd);
		}

		if (method_call.method_name().compare("setSource") == 0) {
			flutter::EncodableMap map = std::get<flutter::EncodableMap>(*method_call.arguments());
			auto path = map.find(flutter::EncodableValue("path"));

			if (path == map.end()) {
				result->Error("Error", "Path is not provided");
			}

			std::string pathStr = std::get<std::string>(path->second);
			std::wstring pathWStr(pathStr.begin(), pathStr.end());

			if (player) {
				//HRESULT hr = player->SetSource(L"D:/Downloads/thehype.mp3");
				HRESULT hr = player->SetSource(pathWStr.c_str());
				if (SUCCEEDED(hr)) {
					result->Success();
				}
				else {
					std::cout << "Error" << __FILE__ << ":" << __LINE__ << "-" << std::hex << hr << std::endl;
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

				std::thread poolThread(&AudioPlayer::StartAudioPool, player);
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
				cout << duration << endl;

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

		else if (method_call.method_name().compare("getPostion")) {
			if (player) {
				double position = 0;
				HRESULT hr = player->GetCDurationSecond(position);
				//cout << "Current position: " << position << endl;

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
		else if (method_call.method_name().compare("seek")) {
			flutter::EncodableMap map = std::get<flutter::EncodableMap>(*method_call.arguments());
			auto position = map.find(flutter::EncodableValue("position"));

			if (position == map.end()) {
				result->Error("Error", "Position is not provided");
			}

			double positionValue = std::get<double>(position->second);

			if (player) {
				MFTIME time = static_cast<MFTIME>(positionValue * MICRO_TO_SECOND);
				HRESULT hr = player->SetPosition(time);
				if (SUCCEEDED(hr)) {
					result->Success(flutter::EncodableValue(1));
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
				result->Success(flutter::EncodableValue(player->GetState()));
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
		cout << "Create Player instance" << endl;
		AudioPlayer::CreateInstance(&player);
	}

	AudiopcPlugin::~AudiopcPlugin() {
		cout << "Plugin end here" << endl;
	}


	HRESULT AudioPlayer::CreateInstance(AudioPlayer** ppCB)
	{
		HRESULT hr = S_OK;
		if (ppCB == NULL)
		{
			return E_POINTER;
		}

		AudioPlayer* pPlayer = new (std::nothrow) AudioPlayer();
		if (pPlayer == NULL)
		{
			return E_OUTOFMEMORY;
		}

		hr = pPlayer->StartPlayer();
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		*ppCB = pPlayer;
		(*ppCB)->AddRef();

	done:
		return hr;
	}

	HRESULT AudioPlayer::StartPlayer() {
		cout << "StartPlayer" << endl;
		HRESULT hr = S_OK;
		hr = MFStartup(MF_VERSION);
		if (SUCCEEDED(hr))
		{
			m_hCloseEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
			if (m_hCloseEvent == NULL)
			{
				hr = HRESULT_FROM_WIN32(GetLastError());
			}
		}
		return hr;
	}

	HRESULT AudioPlayer::SetSource(const WCHAR* path) {
		cout << "SetSoucre: " << path << endl;
		MF_OBJECT_TYPE ObjectType = MF_OBJECT_INVALID;
		HRESULT hr = S_OK;
		HRESULT hr_Tmp = S_OK;

		hr = CreateMediaSession();

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = MFCreateSourceResolver(&m_pSourceResolver);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSourceResolver->CreateObjectFromURL(
			path,
			MF_RESOLUTION_MEDIASOURCE,
			NULL,
			&ObjectType,
			&m_pSourceUnk
		);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSourceUnk->QueryInterface(IID_PPV_ARGS(&m_pSource));

		SAFE_RELEASE(&m_pSourceResolver);
		SAFE_RELEASE(&m_pSourceUnk);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSource->CreatePresentationDescriptor(&m_pPD);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = CreatePlaybackTopology();

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSession->SetTopology(0, m_pTopology);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSession->GetSessionCapabilities(&m_caps);
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		IMFClock* pClock = NULL;

		hr = m_pSession->GetClock(&pClock);
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		hr = pClock->QueryInterface(IID_PPV_ARGS(&m_pClock));
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr_Tmp = MFGetService(m_pSession, MF_RATE_CONTROL_SERVICE, IID_PPV_ARGS(&m_pRate));
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
		}
		hr_Tmp = MFGetService(m_pSession, MF_RATE_CONTROL_SERVICE, IID_PPV_ARGS(&m_pRateSupport));
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
		}
		hr_Tmp = m_pRateSupport->IsRateSupported(TRUE, 0, NULL);
		if (SUCCEEDED(hr)) {
			m_bCanScrub = TRUE;
		}

		assert(m_pRate || !m_bCanScrub);

		m_state = OpenPending;
	done:
		if (FAILED(hr))
		{
			m_state = Closed;
		}
		SAFE_RELEASE(&pClock);
		return hr;
	}

	STDMETHODIMP AudioPlayer::Invoke(IMFAsyncResult* pResult) {
		std::cout << "Invoke is called" << std::endl;
		HRESULT hr = S_OK;
		// Get the event from the event queue.
		hr = m_pSession->EndGetEvent(pResult, &m_pEvent);
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pEvent->GetType(&m_eType);
		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		if (m_eType == MESessionClosed)
		{
			// The session was closed. 
			// The application is waiting on the m_hCloseEvent event handle. 
			SetEvent(m_hCloseEvent);
		}
		else
		{
			// For all other events, get the next event in the queue.
			hr = m_pSession->BeginGetEvent(this, NULL);
			if (FAILED(hr)) {
				std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}
		}

		if (m_state != Closing)
		{
			m_pEvent->AddRef();
			PostMessage(m_hwndEvent, WM_APP_PLAYER_EVENT,
				(WPARAM)m_pEvent, (LPARAM)m_eType);
		}
	done:
		return hr;
	}

	STDMETHODIMP_(ULONG) AudioPlayer::AddRef() {
		return InterlockedIncrement(&m_cRef);
	}


	STDMETHODIMP_(ULONG) AudioPlayer::Release() {
		ULONG uCount = InterlockedDecrement(&m_cRef);
		if (uCount == 0) {
			delete this;
		}
		return uCount;
	}

	HRESULT AudioPlayer::CreateMediaSession() {
		cout << "CreateMediaSession" << endl;
		HRESULT hr = S_OK;

		// Close the old session, if any.
		hr = CloseSession();
		if (FAILED(hr)) {
			////std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		assert(m_state == Closed);

		hr = MFCreateMediaSession(NULL, &m_pSession);
		if (FAILED(hr)) {
			////std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSession->BeginGetEvent((IMFAsyncCallback*)this, NULL);
		if (FAILED(hr)) {
			////std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		m_state = Ready;
	done:
		return hr;
	}

	HRESULT AudioPlayer::CloseSession() {
		cout << "CloseSession" << endl;
		HRESULT hr = S_OK;

		if (m_pSession) {
			m_state = Closing;
			hr = m_pSession->Close();
			if (SUCCEEDED(hr))
			{
				// Wait for the close operation to complete
				hr = WaitForSingleObject(m_hCloseEvent, 5000);
				if (FAILED(hr)) {
					assert(FALSE);
				}
			}
		}

		if (SUCCEEDED(hr))
		{
			// Shut down the media source. (Synchronous operation, no events.)
			if (m_pSource)
			{
				(void)m_pSource->Shutdown();
			}
			// Shut down the media session. (Synchronous operation, no events.)
			if (m_pSession)
			{
				(void)m_pSession->Shutdown();
			}
		}
		SAFE_RELEASE(&m_pSource);
		SAFE_RELEASE(&m_pSession);
		m_state = Closed;
		return hr;
	}

	AudioPlayer::AudioPlayer() : m_cRef(1), m_pSourceResolver(NULL), m_pEvent(NULL), m_pTopology(NULL),
		m_pBuffer(NULL), m_pSource(NULL), m_hCloseEvent(NULL), m_state(Closed), m_eType(MEUnknown),
		m_pSourceUnk(NULL), m_pSession(NULL), m_pStreamDescriptor(NULL), m_poolFlag(true),
		m_duration(NULL), m_pClock(NULL), m_pRate(NULL), m_pRateSupport(NULL),
		m_pOutputNode(NULL), m_pSinkActivate(NULL), m_pPD(NULL), m_pSourceNode(NULL) {
		std::cout << "Contructor" << std::endl;
	}

	AudioPlayer::~AudioPlayer() {
		assert(m_pSession == 0);

		Shutdown();
	}

	HRESULT AudioPlayer::StartPlayback()
	{
		assert(m_pSession != NULL);
		HRESULT hr = S_OK;


		PROPVARIANT varStart;
		PropVariantInit(&varStart);

		hr = m_pSession->Start(&GUID_NULL, &varStart);
		if (SUCCEEDED(hr)) {
			m_state = Started;
		}

		m_sState.command = CmdStart;
		m_bPending = CMD_PENDING;

		PropVariantClear(&varStart);
		if (m_bPending) {
			m_request.command = CmdStart;
		}
		return hr;
	}

	//  Start playback from paused or stopped.
	HRESULT AudioPlayer::Play()
	{
		std::cout << "Play" << std::endl;
		AutoLock lock(m_critsec);

		if (m_state != Paused && m_state != Stopped)
		{
			return MF_E_INVALIDREQUEST;
		}
		if (m_pSession == NULL || m_pSource == NULL)
		{
			return E_UNEXPECTED;
		}
		return StartPlayback();
	}

	HRESULT AudioPlayer::Pause()
	{
		cout << "Pause" << endl;
		AutoLock lock(m_critsec);
		HRESULT hr = S_OK;

		if (m_state != Started)
		{
			return MF_E_INVALIDREQUEST;
		}
		if (m_pSession == NULL || m_pSource == NULL)
		{
			return E_UNEXPECTED;
		}

		hr = m_pSession->Pause();
		if (SUCCEEDED(hr))
		{
			m_state = Paused;
		}
		if (m_bPending) {
			m_request.command = CmdPause;
			m_bPending = CMD_PENDING;
		}

		return hr;
	}

	// Stop playback.
	HRESULT AudioPlayer::Stop()
	{
		cout << "Stop" << endl;
		HRESULT hr = S_OK;
		if (m_state != Started && m_state != Paused)
		{
			return MF_E_INVALIDREQUEST;
		}
		if (m_pSession == NULL)
		{
			return E_UNEXPECTED;
		}

		hr = m_pSession->Stop();
		if (SUCCEEDED(hr))
		{
			m_state = Stopped;
		}
		if (m_bPending) {
			m_request.command = CmdStop;
			m_bPending = CMD_PENDING;
		}
		return hr;
	}

	HRESULT AudioPlayer::GetDuration() {
		HRESULT hr = S_OK;
		UINT64 duration = 0;
		hr = m_pPD->GetUINT64(MF_PD_DURATION, &duration);

		if (FAILED(hr)) {
			std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			hr = MF_E_NO_DURATION;
			goto done;
		}

		m_duration = duration;

	done:
		return hr;
	}

	HRESULT AudioPlayer::CreateMediaSinkActivate() {
		cout << "CreateMediaSinkActivate" << endl;
		IMFMediaTypeHandler* pHandler = NULL;

		HRESULT hr = S_OK;

		hr = m_pStreamDescriptor->GetMediaTypeHandler(&pHandler);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Get the major media type.
		GUID guidMajorType;
		hr = pHandler->GetMajorType(&guidMajorType);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Create an IMFActivate object for the renderer, based on the media type.
		if (MFMediaType_Audio == guidMajorType)
		{
			// Create the audio renderer.
			hr = MFCreateAudioRendererActivate(&m_pSinkActivate);
		}
		else
		{
			// Unknown stream type. 
			hr = E_FAIL;
			// Optionally, you could deselect this stream instead of failing.
		}

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		// Return IMFActivate pointer to caller.
		m_pSinkActivate->AddRef();

	done:
		SAFE_RELEASE(&pHandler);
		return hr;
	}

	// Add a source node to a topology.
	HRESULT AudioPlayer::AddSourceNode() {
		cout << "AddSourceNode" << endl;
		// Create the node.
		HRESULT hr = S_OK;

		hr = MFCreateTopologyNode(MF_TOPOLOGY_SOURCESTREAM_NODE, &m_pSourceNode);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSourceNode->SetUnknown(MF_TOPONODE_SOURCE, m_pSource);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSourceNode->SetUnknown(MF_TOPONODE_PRESENTATION_DESCRIPTOR, m_pPD);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pSourceNode->SetUnknown(MF_TOPONODE_STREAM_DESCRIPTOR, m_pStreamDescriptor);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pTopology->AddNode(m_pSourceNode);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		m_pSourceNode->AddRef();

	done:
		return hr;
	}

	// Add an output node to a topology.
	HRESULT AudioPlayer::AddOutputNode() {
		cout << "AddOutputNode" << endl;
		HRESULT hr = S_OK;
		// Create the node.
		hr = MFCreateTopologyNode(MF_TOPOLOGY_OUTPUT_NODE, &m_pOutputNode);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Set the object pointer.
		hr = m_pOutputNode->SetObject(m_pSinkActivate);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Set the stream sink ID attribute.
		hr = m_pOutputNode->SetUINT32(MF_TOPONODE_STREAMID, 0);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		hr = m_pOutputNode->SetUINT32(MF_TOPONODE_NOSHUTDOWN_ON_REMOVE, FALSE);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Add the node to the topology.
		hr = m_pTopology->AddNode(m_pOutputNode);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Return the pointer to the caller.
		m_pOutputNode->AddRef();

	done:
		return hr;
	}
	//</SnippetPlayer.cpp>

	//  Add a topology branch for one stream.
	//
	//  For each stream, this function does the following:
	//
	//    1. Creates a source node associated with the stream. 
	//    2. Creates an output node for the renderer. 
	//    3. Connects the two nodes.
	//
	//  The media session will add any decoders that are needed.

	HRESULT AudioPlayer::AddBranchToPartialTopology(DWORD index) {
		cout << "AddBranchToPartialTopology" << endl;
		BOOL fSelected = FALSE;

		HRESULT hr = S_OK;

		hr = m_pPD->GetStreamDescriptorByIndex(index, &fSelected, &m_pStreamDescriptor);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		if (fSelected)
		{
			// Create the media sink activation object.
			hr = CreateMediaSinkActivate();
			if (FAILED(hr)) {
				//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}

			// Add a source node for this stream.
			hr = AddSourceNode();
			if (FAILED(hr)) {
				//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}
			hr = AddOutputNode();

			if (FAILED(hr)) {
				//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}
			hr = m_pSourceNode->ConnectOutput(0, m_pOutputNode, 0);
		}
	done:
		SAFE_RELEASE(&m_pStreamDescriptor);
		SAFE_RELEASE(&m_pSinkActivate);
		SAFE_RELEASE(&m_pOutputNode);
		SAFE_RELEASE(&m_pSourceNode);
		return hr;
	}

	HRESULT AudioPlayer::CreatePlaybackTopology() {
		cout << "CreatePlaybackTopology" << endl;
		DWORD cSourceStreams = 0;

		HRESULT hr = S_OK;

		hr = MFCreateTopology(&m_pTopology);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		// Get the number of streams in the media source.
		hr = m_pPD->GetStreamDescriptorCount(&cSourceStreams);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// For each stream, create the topology nodes and add them to the topology.
		for (DWORD i = 0; i < cSourceStreams; i++)
		{
			hr = AddBranchToPartialTopology(i);
			if (FAILED(hr)) {
				//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}
		}

		m_pTopology->AddRef();

	done:
		return hr;
	}

	HRESULT AudioPlayer::Shutdown() {
		cout << "Shutdown" << endl;
		// Close the session
		HRESULT hr = S_OK;

		hr = CloseSession();
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Shutdown the Media Foundation platform
		MFShutdown();

		if (m_hCloseEvent)
		{
			CloseHandle(m_hCloseEvent);
			m_hCloseEvent = NULL;
		}
	done:
		return hr;
	}

	/// Protected methods

	HRESULT AudioPlayer::OnTopologyStatus() {
		std::cout << "OnTopologyStatus" << std::endl;
		UINT32 status;
		HRESULT hr = S_OK;
		hr = m_pEvent->GetUINT32(MF_EVENT_TOPOLOGY_STATUS, &status);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		if (status == MF_TOPOSTATUS_READY)
		{

			hr = StartPlayback();
			if (FAILED(hr)) {
				//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}
		}
	done:
		return hr;
	}


	//  Handler for MEEndOfPresentation event.
	HRESULT AudioPlayer::OnPresentationEnded() {
		std::cout << "OnPresentationEnded" << std::endl;
		// The session puts itself into the stopped state automatically.
		m_state = Stopped;
		return S_OK;
	}

	//  Handler for MENewPresentation event.
	//
	//  This event is sent if the media source has a new presentation, which 
	//  requires a new topology. 

	HRESULT AudioPlayer::OnNewPresentation() {
		std::cout << "OnNewPresentation" << std::endl;
		HRESULT hr = S_OK;
		// Get the presentation descriptor from the event.
		hr = GetEventObject(m_pEvent, &m_pPD);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Create a partial topology.
		hr = CreatePlaybackTopology();
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Set the topology on the media session.
		hr = m_pSession->SetTopology(0, m_pTopology);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		m_state = OpenPending;

	done:
		SAFE_RELEASE(&m_pPD);
		SAFE_RELEASE(&m_pTopology);
		return S_OK;
	}

	HRESULT AudioPlayer::EventListener() {
		HRESULT hr = S_OK;
		// Make sure the media session is valid
		if (!m_pSession || !m_pEvent) {
			return E_POINTER; // Return error if the session or event pointer is invalid
		}

		// Wait for the next event
		hr = m_pSession->GetEvent(0, &m_pEvent);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Check if the event is valid
		if (!m_pEvent) {
			std::cout << "Null event received from media session." << std::endl;
			return E_POINTER;
		}
	done:
		return hr;
	}

	HRESULT AudioPlayer::EventHandle() {
		HRESULT hrStatus = S_OK;
		HRESULT hr = S_OK;
		PROPVARIANT var;

		if (m_pEvent == NULL)
		{
			return E_POINTER;
		}

		// Get the event type.
		hr = m_pEvent->GetType(&m_eType);
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}
		// Get the event status. If the operation that triggered the event 
		// did not succeed, the status is a failure code.
		hr = m_pEvent->GetStatus(&hrStatus);

		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		// Check if the async operation succeeded.
		if (SUCCEEDED(hr) && FAILED(hrStatus))
		{
			hr = hrStatus;
		}
		if (FAILED(hr)) {
			//std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			goto done;
		}

		switch (m_eType)
		{
		case MESessionTopologyStatus:
			hr = OnTopologyStatus();
			break;

		case MEEndOfPresentation:
			hr = OnPresentationEnded();
			break;

		case MENewPresentation:
			hr = OnNewPresentation();
			break;

		case MESessionRateChanged:
			// If the rate change succeeded, we've already got the rate
			// cached. If it failed, try to get the actual rate.
			if (FAILED(hrStatus))
			{
				PropVariantInit(&var);

				hr = m_pEvent->GetValue(&var);

				if (SUCCEEDED(hr) && (var.vt == VT_R4))
				{
					m_sState.fRate = var.fltVal;
				}
			}
			break;
		case MESessionCapabilitiesChanged:
			// The session capabilities changed. Get the updated capabilities.
			m_caps = MFGetAttributeUINT32(m_pEvent, MF_EVENT_SESSIONCAPS, m_caps);
			break;

			/*case MESessionStarted:
				OnSessionStart();
				break;

			case MESessionStopped:
				OnSessionStop();
				break;

			case MESessionPaused:
				OnSessionPause();
				break;
			case MESessionEnded:
				OnSessionEnded();
				break;*/
		default:
			hr = S_OK;
			break;
		}

	done:
		return hr;
	}

	HRESULT AudioPlayer::GetSecondDuration(double& ref) {
		HRESULT hr = S_OK;
		if (m_pSource && m_pPD) {
			hr = GetDuration();
			if (FAILED(hr)) {
				std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
				goto done;
			}

			std::cout << "Duration: " << m_duration << std::endl;

			ref = static_cast<double>(m_duration) / MICRO_TO_SECOND;

		}
	done:
		return hr;
	}


	HRESULT AudioPlayer::CanSeek(BOOL* pbCanSeek) {
		if (pbCanSeek == NULL)
		{
			return E_POINTER;
		}

		// Note: The MFSESSIONCAP_SEEK flag is sufficient for seeking. However, to
		// implement a seek bar, an application also needs the duration (to get 
		// the valid range) and a presentation clock (to get the current position).
		// and make sure the duration is set

		*pbCanSeek = (
			((m_caps & MFSESSIONCAP_SEEK) == MFSESSIONCAP_SEEK) &&
			(m_duration > 0) &&
			(m_pClock != NULL)
			);

		return S_OK;
	}

	HRESULT AudioPlayer::GetCurrentPosition() {
		HRESULT hr = S_OK;
		if (m_pClock == NULL)
		{
			return MF_E_NO_CLOCK;
		}

		AutoLock lock(m_critsec);

		if (m_request.command == CmdSeek)
		{
			m_cDuration = m_request.hnsStart;
		}
		else if (m_bPending & CMD_PENDING_SEEK)
		{
			m_cDuration = m_sState.hnsStart;
		}
		else
		{
			hr = m_pClock->GetTime(&m_cDuration);
		}
		return hr;
	}

	HRESULT AudioPlayer::GetCDurationSecond(double& ref) {
		if (m_pClock) {
			HRESULT hr = S_OK;
			hr = GetCurrentPosition();
			if (FAILED(hr)) {
				std::cout << "Error at " << __FILE__ << ":" << __LINE__ << " - " << std::hex << hr << std::endl;
			}
			ref = static_cast<double>(m_cDuration) / MICRO_TO_SECOND;
		}
		return S_OK;
	}


	HRESULT AudioPlayer::SetPosition(MFTIME hnsPosition) {
		AutoLock lock(m_critsec);

		HRESULT hr = S_OK;

		if (m_bPending)
		{
			// Currently seeking or changing rates, so cache this request.
			m_request.command = CmdSeek;
			m_request.hnsStart = hnsPosition;
		}
		else
		{
			hr = SetPositionInternal(hnsPosition);
		}

		return hr;
	}

	HRESULT AudioPlayer::CanScrub(BOOL* pbCanScrub)
	{
		if (pbCanScrub == NULL)
		{
			return E_POINTER;
		}

		*pbCanScrub = m_bCanScrub;
		return S_OK;
	}


	HRESULT AudioPlayer::Scrub(BOOL bScrub)
	{
		// Scrubbing is implemented as rate = 0.

		AutoLock lock(m_critsec);

		if (!m_pRate)
		{
			return MF_E_INVALIDREQUEST;
		}
		if (!m_bCanScrub)
		{
			return MF_E_INVALIDREQUEST;
		}

		HRESULT hr = S_OK;

		if (bScrub)
		{
			// Enter scrubbing mode. Cache the current rate.

			if (GetNominalRate() != 0)
			{
				m_fPrevRate = m_sState.fRate;
			}

			hr = SetRate(0.0f);
		}
		else
		{
			// Leaving scrubbing mode. Restore the old rate.

			if (GetNominalRate() == 0)
			{
				hr = SetRate(m_fPrevRate);
			}
		}

		return hr;
	}

	HRESULT AudioPlayer::CanFastForward(BOOL* pbCanFF)
	{
		if (pbCanFF == NULL)
		{
			return E_POINTER;
		}

		*pbCanFF =
			((m_caps & MFSESSIONCAP_RATE_FORWARD) == MFSESSIONCAP_RATE_FORWARD);
		return S_OK;
	}


	// Queries whether the current session supports rewind (reverse play).

	HRESULT AudioPlayer::CanRewind(BOOL* pbCanRewind)
	{
		if (pbCanRewind == NULL)
		{
			return E_POINTER;
		}

		*pbCanRewind =
			((m_caps & MFSESSIONCAP_RATE_REVERSE) == MFSESSIONCAP_RATE_REVERSE);
		return S_OK;
	}


	// Switches to fast-forward playback, as follows:
	// - If the current rate is < 0 (reverse play), switch to 1x speed.
	// - Otherwise, double the current playback rate.
	//
	// Note: This method is for convenience; the application can also call SetRate.

	HRESULT AudioPlayer::FastForward()
	{
		if (!m_pRate)
		{
			return MF_E_INVALIDREQUEST;
		}

		HRESULT hr = S_OK;
		float   fTarget = GetNominalRate() * 2;

		if (fTarget <= 0.0f)
		{
			fTarget = 1.0f;
		}

		hr = SetRate(fTarget);

		return hr;
	}


	// Switches to reverse playback, as follows:
	// - If the current rate is > 0 (forward playback), switch to -1x speed.
	// - Otherwise, double the current (reverse) playback rate.
	//
	// Note: This method is for convenience; the application can also call SetRate.

	HRESULT AudioPlayer::Rewind()
	{
		if (!m_pRate)
		{
			return MF_E_INVALIDREQUEST;
		}

		HRESULT hr = S_OK;
		float   fTarget = GetNominalRate() * 2;

		if (fTarget >= 0.0f)
		{
			fTarget = -1.0f;
		}

		hr = SetRate(fTarget);

		return hr;
	}


	// Sets the playback rate.

	HRESULT AudioPlayer::SetRate(float fRate)
	{
		HRESULT hr = S_OK;
		BOOL bThin = FALSE;

		AutoLock lock(m_critsec);

		if (fRate == GetNominalRate())
		{
			return S_OK; // no-op
		}

		if (m_pRateSupport == NULL)
		{
			return MF_E_INVALIDREQUEST;
		}

		// Check if this rate is supported. Try non-thinned playback first,
		// then fall back to thinned playback.

		hr = m_pRateSupport->IsRateSupported(FALSE, fRate, NULL);

		if (FAILED(hr))
		{
			bThin = TRUE;
			hr = m_pRateSupport->IsRateSupported(TRUE, fRate, NULL);
		}

		if (FAILED(hr))
		{
			// Unsupported rate.
			return hr;
		}

		// If there is an operation pending, cache the request.
		if (m_bPending)
		{
			m_request.fRate = fRate;
			m_request.bThin = bThin;

			// Remember the current transport state (play, paused, etc), so that we
			// can restore it after the rate change, if necessary. However, if 
			// anothercommand is already pending, that one takes precedent.

			if (m_request.command == CmdNone)
			{
				m_request.command = m_sState.command;
			}

		}
		else
		{
			// No pending operation. Commit the new rate.
			hr = CommitRateChange(fRate, bThin);
		}

		return hr;

	}

	// Sets the playback position.

	HRESULT AudioPlayer::SetPositionInternal(const MFTIME& hnsPosition)
	{
		assert(!m_bPending);

		if (m_pSession == NULL)
		{
			return MF_E_INVALIDREQUEST;
		}

		HRESULT hr = S_OK;

		PROPVARIANT varStart;
		varStart.vt = VT_I8;
		varStart.hVal.QuadPart = hnsPosition;

		hr = m_pSession->Start(NULL, &varStart);

		if (SUCCEEDED(hr))
		{
			// Store the pending state.
			m_sState.command = CmdStart;
			m_sState.hnsStart = hnsPosition;
			m_bPending = CMD_PENDING_SEEK;
		}
		return hr;
	}




	// Sets the playback rate.

	HRESULT AudioPlayer::CommitRateChange(float fRate, BOOL bThin)
	{
		assert(!m_bPending);

		// Caller holds the lock.

		HRESULT hr = S_OK;
		MFTIME  hnsSystemTime = 0;
		MFTIME  hnsClockTime = 0;

		Command cmdNow = m_sState.command;

		IMFClock* pClock = NULL;

		// Allowed rate transitions:

		// Positive <-> negative:   Stopped
		// Negative <-> zero:       Stopped
		// Postive <-> zero:        Paused or stopped

		if ((fRate > 0 && m_sState.fRate <= 0) || (fRate < 0 && m_sState.fRate >= 0))
		{
			// Transition to stopped.
			if (cmdNow == CmdStart)
			{
				// Get the current clock position. This will be the restart time.
				hr = m_pSession->GetClock(&pClock);
				if (FAILED(hr))
				{
					goto done;
				}

				(void)pClock->GetCorrelatedTime(0, &hnsClockTime, &hnsSystemTime);

				assert(hnsSystemTime != 0);

				// Stop and set the rate
				hr = Stop();
				if (FAILED(hr))
				{
					goto done;
				}

				// Cache Request: Restart from stop.
				m_request.command = CmdSeek;
				m_request.hnsStart = hnsClockTime;
			}
			else if (cmdNow == CmdPause)
			{
				// The current state is paused.

				// For this rate change, the session must be stopped. However, the 
				// session cannot transition back from stopped to paused. 
				// Therefore, this rate transition is not supported while paused.

				hr = MF_E_UNSUPPORTED_STATE_TRANSITION;
				goto done;
			}
		}
		else if (fRate == 0 && m_sState.fRate != 0)
		{
			if (cmdNow != CmdPause)
			{
				// Transition to paused.

				 // This transition requires the paused state.

				 // Pause and set the rate.
				hr = Pause();
				if (FAILED(hr))
				{
					goto done;
				}

				// Request: Switch back to current state.
				m_request.command = cmdNow;
			}
		}

		// Set the rate.
		hr = m_pRate->SetRate(bThin, fRate);
		if (FAILED(hr))
		{
			goto done;
		}

		// Adjust our current rate and requested rate.
		m_request.fRate = m_sState.fRate = fRate;

	done:
		SAFE_RELEASE(&pClock);
		return hr;
	}


	float AudioPlayer::GetNominalRate()
	{
		return m_request.fRate;
	}

	HRESULT AudioPlayer::OnSessionStart()
	{
		HRESULT hr = S_OK;
		HRESULT hrStatus = S_OK;
		if (FAILED(hrStatus))
		{
			return hrStatus;
		}

		// The Media Session completed a start/seek operation. Check if there
		// is another seek request pending.
		UpdatePendingCommands(CmdStart);

		return hr;
	}


	// Called when playback stops.

	HRESULT AudioPlayer::OnSessionStop()
	{
		HRESULT hr = S_OK;
		HRESULT hrStatus = S_OK;
		if (FAILED(hrStatus))
		{
			return hrStatus;
		}

		// The Media Session completed a transition to stopped. This might occur
		// because we are changing playback direction (forward/rewind). Check if
		// there is a pending rate-change request.

		UpdatePendingCommands(CmdStop);

		return hr;
	}


	// Called when playback pauses.

	HRESULT AudioPlayer::OnSessionPause()
	{
		HRESULT hr = S_OK;
		HRESULT hrStatus = S_OK;

		if (FAILED(hrStatus))
		{
			return hrStatus;
		}

		hr = UpdatePendingCommands(CmdPause);

		return hr;
	}



	// Called when the session ends.

	HRESULT AudioPlayer::OnSessionEnded()
	{
		// After the session ends, playback starts from position zero. But if the
		// current playback rate is reversed, playback would end immediately 
		// (reversing from position 0). Therefore, reset the rate to 1x.
		HRESULT hr = S_OK;
		if (GetNominalRate() < 0.0f)
		{
			m_sState.command = CmdStop;

			hr = CommitRateChange(1.0f, FALSE);
		}

		return hr;
	}


	// Called after an operation completes.
	// This method executes any cached requests.

	HRESULT AudioPlayer::UpdatePendingCommands(Command cmd)
	{
		HRESULT hr = S_OK;

		PROPVARIANT varStart;
		PropVariantInit(&varStart);

		AutoLock lock(m_critsec);

		if (m_bPending && m_sState.command == cmd)
		{
			m_bPending = FALSE;

			// The current pending command has completed.

			// First look for rate changes.
			if (m_request.fRate != m_sState.fRate)
			{
				hr = CommitRateChange(m_request.fRate, m_request.bThin);
				if (FAILED(hr))
				{
					goto done;
				}
			}

			// Now look for seek requests.
			if (!m_bPending)
			{
				switch (m_request.command)
				{
				case CmdNone:
					// Nothing to do.
					break;

				case CmdStart:
					Play();
					break;

				case CmdPause:
					Pause();
					break;

				case CmdStop:
					Stop();
					break;

				case CmdSeek:
					SetPositionInternal(m_request.hnsStart);
					break;
				}
				m_request.command = CmdNone;
			}
		}

	done:
		return hr;
	}

	PlaybackState AudioPlayer::GetState() {
		return m_state;
	}

}  // namespace audiopc
