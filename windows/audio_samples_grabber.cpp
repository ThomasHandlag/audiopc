#include "audio_samples_grabber.h"
#include <Shlwapi.h>

namespace audiopc {

	AudioSamplesGrabber::AudioSamplesGrabber(
		shared_ptr<EventSink<flutter::EncodableValue>>* handler,
		const std::string id) : m_cRef(1), handler(*handler) {
		this->id = id;
	}

	AudioSamplesGrabber::~AudioSamplesGrabber() {
		m_samples.clear();
	}

	HRESULT AudioSamplesGrabber::CreateInstance(
		AudioSamplesGrabber** ppCB, std::string id,
		shared_ptr<EventSink<flutter::EncodableValue>>* handler)
	{
		*ppCB = new (nothrow) AudioSamplesGrabber(handler, id);

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
		samples.reserve(dwSampleSize / 2);
		for (DWORD i = 0; i < dwSampleSize; i += 2) {
			int16_t int16_sample = static_cast<int16_t>(
				static_cast<uint8_t>(pSampleBuffer[i]) |
				(static_cast<uint8_t>(pSampleBuffer[i + 1]) << 8)
				);

			// Normalize the sample to a double in the range [-1.0, 1.0]
			double sample = static_cast<double>(int16_sample) / 32768.0;

			samples.push_back(sample);
		}
		if (handler) {
			// Emit the samples to the event handler
			emitValue({ {"id", id},
				{"event", "samples"},
				{"value", flutter::EncodableValue(samples)}
				});
		}

		return S_OK;
	}

	void AudioSamplesGrabber::emitValue(const std::map<std::string, EncodableValue> value) const {
		if (handler) {
			flutter::EncodableMap map;
			for (auto& [key, data] : value) {
				map[EncodableValue(key)] = data;
			}
			handler->Success(EncodableValue(map));
		}
	}

	STDMETHODIMP AudioSamplesGrabber::OnShutdown()
	{
		return S_OK;
	}
}