#pragma once
#include <windows.h>
#include <mfapi.h>
#include <iostream>
#include <vector>
#include <thread>
#include <mfidl.h>
#include "circular_buffer.h"

namespace audiopc {

	using std::cout, std::endl, std::vector, std::thread, std::nothrow;
	class AudioSamplesGrabber : public IMFSampleGrabberSinkCallback {
	public:
		ULONG m_cRef;
		CircularBuffer samplesBuffer;
		vector<double> m_samples;

		AudioSamplesGrabber();
		~AudioSamplesGrabber();
		// IUnknown methods
		static HRESULT CreateInstance(AudioSamplesGrabber** ppCB);

		// IUnknown methods
		STDMETHODIMP QueryInterface(REFIID iid, void** ppv);
		STDMETHODIMP_(ULONG) AddRef();
		STDMETHODIMP_(ULONG) Release();

		// IMFClockStateSink methods
		STDMETHODIMP OnClockStart(MFTIME hnsSystemTime, LONGLONG llClockStartOffset);
		STDMETHODIMP OnClockStop(MFTIME hnsSystemTime);
		STDMETHODIMP OnClockPause(MFTIME hnsSystemTime);
		STDMETHODIMP OnClockRestart(MFTIME hnsSystemTime);
		STDMETHODIMP OnClockSetRate(MFTIME hnsSystemTime, float flRate);

		// IMFSampleGrabberSinkCallback methods
		STDMETHODIMP OnSetPresentationClock(IMFPresentationClock* pClock);
		STDMETHODIMP OnProcessSample(REFGUID guidMajorMediaType, DWORD dwSampleFlags,
			LONGLONG llSampleTime, LONGLONG llSampleDuration, const BYTE* pSampleBuffer,
			DWORD dwSampleSize);
		STDMETHODIMP OnShutdown();
	};

} // namespace audiopc
