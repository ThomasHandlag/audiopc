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

	// Performance-optimized sample processing callback.
	// Called frequently during audio playback (potentially thousands of times per second).
	// 
	// Optimizations:
	// - Pre-allocate vector with exact size to avoid incremental growth
	// - Use const pointer for better compiler optimization
	// - Replace division with multiplication (faster on most CPUs)
	// - Cleaner type casting reduces redundant operations
	STDMETHODIMP AudioSamplesGrabber::OnProcessSample(REFGUID guidMajorMediaType, DWORD dwSampleFlags,
		LONGLONG llSampleTime, LONGLONG llSampleDuration, const BYTE* pSampleBuffer,
		DWORD dwSampleSize)
	{
		// Calculate exact sample count and pre-allocate vector.
		// Before: Vector grew incrementally during push_back operations.
		// After: Single allocation for exact size needed.
		const size_t sampleCount = dwSampleSize / 2;
		vector<double> samples;
		samples.reserve(sampleCount);
		
		// Use const pointer for better optimization hints to compiler.
		// Indicates that buffer won't be modified, enabling more aggressive optimizations.
		const uint8_t* buffer = pSampleBuffer;
		
		for (size_t i = 0; i < dwSampleSize; i += 2) {
			// Combine two bytes into int16 using little-endian format.
			// Cleaner type casting improves readability and performance.
			const int16_t int16_sample = static_cast<int16_t>(
				buffer[i] | (buffer[i + 1] << 8)
			);

			// Normalize to [-1.0, 1.0] range.
			// Multiplication is typically 2-3x faster than division on modern CPUs.
			// Before: int16_sample / 32768.0
			// After: int16_sample * (1.0 / 32768.0)
			samples.push_back(static_cast<double>(int16_sample) * (1.0 / 32768.0));
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