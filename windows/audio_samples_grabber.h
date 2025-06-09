#pragma once
#include <windows.h>
#include <mfapi.h>
#include <iostream>
#include <vector>
#include <thread>
#include <mfidl.h>
#include "circular_buffer.h"
#include "event_stream_handler.h"

namespace audiopc {

	using std::cout, std::endl, std::vector, std::thread, std::nothrow;
	using std::shared_ptr;
	class AudioSamplesGrabber : public IMFSampleGrabberSinkCallback {
	private: 
		vector<double> m_samples;
		std::string id;
		shared_ptr<EventSink<flutter::EncodableValue>> handler;
		void emitValue(const std::map<std::string, EncodableValue> value) const;
	public:
		ULONG m_cRef;

		AudioSamplesGrabber(shared_ptr<EventSink<flutter::EncodableValue>> *handler, const std::string id);
		~AudioSamplesGrabber();
		// IUnknown methods
		static HRESULT CreateInstance(
			AudioSamplesGrabber** ppCB, 
			std::string id,
			shared_ptr<EventSink<flutter::EncodableValue>>* handler
		);

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
