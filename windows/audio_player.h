#pragma once
#include <string>
#include <thread>
#include <mfapi.h>
#include <mfidl.h>
#include "audio_samples_grabber.h"
#include <Shlwapi.h>
#include <chrono>
#include <iostream>

#include <cassert>
#include <Mferror.h>
#include "helper.h"

namespace audiopc {

	constexpr int CMD_PENDING = 0x01;
	constexpr int CMD_PENDING_SEEK = 0x02;
	constexpr int CMD_PENDING_RATE = 0x04;
	constexpr double MICRO_TO_SECOND = 10000000.0;

	constexpr char* DB_RED = "\033[1;31m";
	constexpr char* DB_GREEN = "\033[1;32m";
	constexpr char* DB_YELLOW = "\033[1;33m";
	constexpr char* DB_BLUE = "\033[1;34m";
	constexpr char* DB_MAGENTA = "\033[1;35m";
	constexpr char* DB_CYAN = "\033[1;36m";
	constexpr char* DB_RESET = "\033[0m";
	constexpr char* DB_BOLD = "\033[1m";

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

	template <class T> void SAFE_RELEASE(T** ppT)
	{
		if (*ppT)
		{
			(*ppT)->Release();
			*ppT = NULL;
		}
	}

	enum PlaybackState {
		Closed = 0,     // No session.
		Ready,          // Session was created, ready to open a file. 
		OpenPending,    // Session is opening a file.
		Started,        // Session is playing a file.
		Paused,         // Session is paused.
		Stopped,        // Session is stopped (ready to play). 
		Closing
	};

	class AudioPlayer : public IMFAsyncCallback
	{
	protected:
		ULONG m_cRef;
		AudioSamplesGrabber* m_pGrabber;
		UINT WM_APP_PLAYER_EVENT;


		AudioPlayer(AudioSamplesGrabber** grabber);
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
		void GetSamples(vector<double>& out) const;
		PlaybackState GetState() const;


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
		HRESULT EventHandle();
		HRESULT AddGrabberOutputNode();

		HRESULT GetDuration();

		PlaybackState m_state;
		HANDLE m_hCloseEvent;
		IMFSourceResolver* m_pSourceResolver;
		IMFMediaSource* m_pSource;
		IUnknown* m_pSourceUnk;
		IMFMediaSession* m_pSession;
		IMFStreamDescriptor* m_pStreamDescriptor;
		IMFTopology* m_pTopology;
		IMFPresentationDescriptor* m_pPD;

		IMFTopologyNode* m_pSourceNode;
		IMFTopologyNode* m_pOutputNode;
		IMFTopologyNode* m_pGrabberNode;

		IMFMediaEvent* m_pEvent;
		MediaEventType m_eType;

		IMFActivate* m_pSinkActivate;
		IMFActivate* m_pSinkActivateGrabber;

		IMFMediaType* m_pMediaType;
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
}