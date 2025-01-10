#include "audio_samples_grabber.h"
#include <Shlwapi.h>

namespace audiopc {

	AudioSamplesGrabber::AudioSamplesGrabber() : m_cRef(1), samplesBuffer(44100) {
	}

	AudioSamplesGrabber::~AudioSamplesGrabber() {
	}

	HRESULT AudioSamplesGrabber::CreateInstance(AudioSamplesGrabber** ppCB)
	{
		*ppCB = new (nothrow) AudioSamplesGrabber();

		if (ppCB == NULL)
		{
			return E_OUTOFMEMORY;
		}
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::QueryInterface(REFIID riid, void** ppv)
	{
		static const QITAB qit[] =
		{
			QITABENT(AudioSamplesGrabber, IMFSampleGrabberSinkCallback),
			QITABENT(AudioSamplesGrabber, IMFClockStateSink),
			{ 0 }
		};
		return QISearch(this, qit, riid, ppv);
	}

	STDMETHODIMP_(ULONG) AudioSamplesGrabber::AddRef()
	{
		return InterlockedIncrement(&m_cRef);
	}

	STDMETHODIMP_(ULONG) AudioSamplesGrabber::Release()
	{
		ULONG cRef = InterlockedDecrement(&m_cRef);
		if (cRef == 0)
		{
			delete this;
		}
		return cRef;

	}

	// IMFClockStateSink methods.

	// In these example, the IMFClockStateSink methods do not perform any actions. 
	// You can use these methods to track the state of the sample grabber sink.

	STDMETHODIMP AudioSamplesGrabber::OnClockStart(MFTIME hnsSystemTime, LONGLONG llClockStartOffset)
	{
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnClockStop(MFTIME hnsSystemTime)
	{
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnClockPause(MFTIME hnsSystemTime)
	{
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnClockRestart(MFTIME hnsSystemTime)
	{
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnClockSetRate(MFTIME hnsSystemTime, float flRate)
	{
		return S_OK;
	}

	// IMFSampleGrabberSink methods.

	STDMETHODIMP AudioSamplesGrabber::OnSetPresentationClock(IMFPresentationClock* pClock)
	{
		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnProcessSample(REFGUID guidMajorMediaType, DWORD dwSampleFlags,
		LONGLONG llSampleTime, LONGLONG llSampleDuration, const BYTE* pSampleBuffer,
		DWORD dwSampleSize)
	{
		vector<double> samples;
		for (DWORD i = 0; i < dwSampleSize; i += 2) {
			// Combine two bytes into one int16_t
			int16_t int16_sample = static_cast<int16_t>(pSampleBuffer[i] | (pSampleBuffer[i + 1] << 8));
			// Normalize to double in range [-1.0, 1.0]
			double sample = int16_sample / 32768.0; // 32768 = 2^15
			samples.push_back(sample);
		}
		samplesBuffer.Write(samples);

		return S_OK;
	}

	STDMETHODIMP AudioSamplesGrabber::OnShutdown()
	{
		return S_OK;
	}
}