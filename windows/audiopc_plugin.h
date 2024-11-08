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
#include <iostream>


namespace audiopc {
	class AudioPlayer;
	class AudiopcPlugin;

	// Helper macro for checking HRESULT results and throwing exceptions on failure.

	enum PlaybackState {
		Closed = 0,     // No session.
		Ready,          // Session was created, ready to open a file. 
		OpenPending,    // Session is opening a file.
		Started,        // Session is playing a file.
		Paused,         // Session is paused.
		Stopped,        // Session is stopped (ready to play). 
		Closing
	};

	const UINT WM_APP_PLAYER_EVENT = WM_APP + 1;

	template <class T> void SAFE_RELEASE(T** ppT)
	{
		if (*ppT)
		{
			(*ppT)->Release();
			*ppT = NULL;
		}
	}

#define CMD_PENDING      0x01
#define CMD_PENDING_SEEK 0x02
#define CMD_PENDING_RATE 0x04
#define MICRO_TO_SECOND  10000000.0

	class AudiopcPlugin : public flutter::Plugin {
	public:
		AudioPlayer* player;
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
	class AudioPlayer : public IMFAsyncCallback
	{
	protected:
		ULONG m_cRef;

		AudioPlayer();
		~AudioPlayer();

	public:
		static HRESULT CreateInstance(AudioPlayer** ppCB);

		// IUnknown methods
		STDMETHODIMP QueryInterface(REFIID riid, void** ppv) {
			static const QITAB qit[] =
			{
				QITABENT(AudioPlayer, IMFAsyncCallback),
				{ 0 }
			};
			return QISearch(this, qit, riid, ppv);
		}

		STDMETHODIMP_(ULONG) AddRef();
		STDMETHODIMP_(ULONG) Release();
		STDMETHODIMP Invoke(IMFAsyncResult* pResult);

		STDMETHODIMP GetParameters(DWORD* pdwFlags, DWORD* pdwQueue) {
			return E_NOTIMPL;
		}

		HRESULT SetSource(const WCHAR* path);

		HRESULT Play();
		HRESULT Stop();
		HRESULT Pause();
		HRESULT GetSecondDuration(double& duration);
		/// <summary>
		///  set the HWND for audioplayer event
		/// </summary>
		/// <param name="hwnd"></param>
		void SetHWND(HWND hwnd) { m_hwndEvent = hwnd; }

		/// <summary>
		/// a audio pool for playing audio
		/// </summary>
		/// <returns>HRESULT</returns>
		HRESULT StartAudioPool() {
			while (m_poolFlag) {
				this->EventHandle();
			}
			std::cout << "End of audio playback\n";

			return S_OK;
		};

		HRESULT CanSeek(BOOL* pbCanSeek);
		HRESULT GetCurrentPosition();
		HRESULT GetCDurationSecond(double& duration);
		HRESULT SetPosition(MFTIME hnsPosition);

		HRESULT CanScrub(BOOL* pbCanScrub);
		HRESULT Scrub(BOOL bScrub);

		HRESULT CanFastForward(BOOL* pbCanFF);
		HRESULT CanRewind(BOOL* pbCanRewind);
		HRESULT SetRate(float fRate);
		HRESULT FastForward();
		HRESULT Rewind();
		PlaybackState GetState();


	protected:
		HRESULT StartPlayer();
		HRESULT CreateMediaSession();
		HRESULT CloseSession();
		HRESULT StartPlayback();
		HRESULT CreatePlaybackTopology();
		HRESULT AddBranchToPartialTopology(DWORD index);
		HRESULT AddSourceNode();
		HRESULT AddOutputNode();
		HRESULT CreateMediaSinkActivate();
		HRESULT OnTopologyStatus();
		HRESULT OnPresentationEnded();
		HRESULT OnNewPresentation();
		HRESULT Shutdown();
		HRESULT EventListener();
		HRESULT EventHandle();

		HRESULT GetDuration();

		PlaybackState m_state;
		HANDLE m_hCloseEvent;
		IMFSourceResolver* m_pSourceResolver;
		IMFMediaBuffer* m_pBuffer;
		IMFMediaSource* m_pSource;
		IUnknown* m_pSourceUnk;
		IMFMediaSession* m_pSession;
		IMFStreamDescriptor* m_pStreamDescriptor;
		IMFTopology* m_pTopology;
		IMFPresentationDescriptor* m_pPD;
		IMFActivate* m_pSinkActivate;
		IMFTopologyNode* m_pSourceNode;
		IMFTopologyNode* m_pOutputNode;
		IMFMediaEvent* m_pEvent;
		MediaEventType m_eType;
		MFTIME m_duration;
		HWND m_hwndEvent;
		bool m_poolFlag = true;

		IMFPresentationClock* m_pClock;
		IMFRateControl* m_pRate;
		IMFRateSupport* m_pRateSupport;
		MFTIME m_cDuration = 0;


		class CritSec
		{
		private:
			CRITICAL_SECTION m_criticalSection;
		public:
			CritSec()
			{
				InitializeCriticalSection(&m_criticalSection);
			}
			~CritSec()
			{
				DeleteCriticalSection(&m_criticalSection);
			}
			void Lock()
			{
				EnterCriticalSection(&m_criticalSection);
			}
			void Unlock()
			{
				LeaveCriticalSection(&m_criticalSection);
			}
		};

		class AutoLock
		{
		private:
			CritSec* m_pCriticalSection;
		public:
			AutoLock(CritSec& crit)
			{
				m_pCriticalSection = &crit;
				m_pCriticalSection->Lock();
			}
			~AutoLock()
			{
				m_pCriticalSection->Unlock();
			}
		};

		enum Command
		{
			CmdNone = 0,
			CmdStop,
			CmdStart,
			CmdPause,
			CmdSeek,
		};

		HRESULT SetPositionInternal(const MFTIME& hnsPosition);
		HRESULT CommitRateChange(float fRate, BOOL bThin);
		float   GetNominalRate();

		HRESULT OnSessionStart();
		HRESULT OnSessionStop();
		HRESULT OnSessionPause();
		HRESULT OnSessionEnded();

		HRESULT UpdatePendingCommands(Command cmd);


		struct SeekState
		{
			Command command;
			float   fRate;      // Playback rate
			BOOL    bThin;      // Thinned playback?
			MFTIME  hnsStart;   // Start position
		};

		BOOL        m_bPending;     // Is a request pending?

		SeekState   m_sState;        // Current nominal state.
		SeekState   m_request;      // Pending request.

		CritSec     m_critsec;      // Protects the seeking and rate-change states.

		DWORD       m_caps;         // Session caps.
		BOOL        m_bCanScrub;    // Does the current session support rate = 0.

		float       m_fPrevRate;
	};

}  // namespace audiopc

#endif  // FLUTTER_PLUGIN_AUDIOPC_PLUGIN_H_
